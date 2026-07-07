import shutil
import json
import os
import ssl
import threading
import time
import traceback
import uuid
from queue import Queue
import socket

from config import AgentConfig
from config_gen.runner import run_command
from fileclient.sync import sync
from localdb.database import Database
from messaging.log import debug, info
from messaging.report_error import log_error, report_error
from runner.service_manager import ServiceManager
from backup.backup_manager import BackupManager, FileQueueEntry


class Client:
    def __init__(
        self,
    ):
        if not AgentConfig.instance:
            raise RuntimeError("AgentConfig not initialized")
        self.config = AgentConfig.instance
        self.message_queue = Queue()
        self.file_queue: Queue[FileQueueEntry] = Queue()
        self.init_services()
        self.last_keepalive = time.time()
        self.keepalive_thread = None
        self.thread_pool = []
        self.database = Database(self.config.server["db_path"])
        self.backup_manager = BackupManager()
        self.watchfile_threads: dict[str,threading.Thread] = {}
        self.client_send_lock = threading.Lock()

    def keepalive_check(self):
        stop = False
        while not stop:
            time.sleep(5)
            if (
                time.time() - self.last_keepalive
                > self.config.server["keepalive_interval"]
            ):
                stop = True
        info("Keepalive check stopped")
        report_error(
            "KEEPALIVE CHECK FAILED",
            "The manager has stopped responding. Trying restarting/exiting the agent...",
            level=2,
        )
        exit(1)

    def connect(self):
        info(
            f"Connecting to server {self.config.server['host']}:{self.config.server['port']}..."
        )
        host = self.config.server["host"]
        port = self.config.server["port"]
        api_key = self.config.server["api_key"]
        self.ssl_context = ssl.create_default_context(ssl.Purpose.SERVER_AUTH)
        try:
            self.ssl_context.load_verify_locations(self.config.cert_path)
        except Exception as e:
            print(f"Invalid certificate at {self.config.cert_path}")
            print(f"Error: {e}")
            exit(1)

        self.client = self.ssl_context.wrap_socket(socket.socket(), server_hostname=host)
        try:
            self.client.connect((host, port))
        except ssl.SSLCertVerificationError:
            report_error(
                "SSL CERTIFICATE VERIFICATION FAILED",
                "The server's SSL certificate could not be verified. All future requests will fail.",
                3,
            )
            raise ConnectionError(f"Failed to connect: SSL certificate verification failed. Please ensure the file {self.config.cert_path} is up to date")
        except Exception as e:
            report_error(
                "HOMELAB-MGR SERVER DOWN/CAN'T CONNECT",
                f"ERROR: {str(e)}. All future requests will fail.",
                3,
            )
            raise ConnectionError(f"Failed to connect: {str(e)}")
        self.client.send(api_key.encode())
        data = self.client.recv(1024).decode()
        if len(data) != 2+1+36 or data[:3] != "OK:": # 2: OK, 1: ":", 36: reverse_api_key
            report_error(
                f"Homelab-MGR: Client {self.config.id} ({self.config.name}): connection issue",
                f"Failed to connect: {data}. Probably due to wrong API key. This instance will not receive any commands.",
                2,
            )
            raise ConnectionError(f"Failed to connect: {data}")
        local_api_key = self.database.get_local_setting("reverse_api_key")
        if not local_api_key:
            # First time connection: set the reverse API key
            self.database.set_local_setting("reverse_api_key", data[3:])
            local_api_key = data[3:]
        if local_api_key != data[3:]:
            report_error(
                f"Homelab-MGR: Client {self.config.id} ({self.config.name}): API key mismatch (A: Server got reset, B: You got hacked, C: Other reason)",
                "2",
            )
            raise ConnectionError(f"Server-side API key mismatch, cannot determine if the server is the same as before. Please check your server and agent configuration. (easy fix: delete agent's db file)")

        self.stop = False
        info("Connected to server.")
        info(
            "Syncronizing services folder (this might take a while if there are many services)..."
        )
        self.keepalive_thread = threading.Thread(target=self.keepalive_check)
        self.keepalive_thread.start()
        self.sync_services()
        self.connect_transfer_sockets()
        self.file_queue_thread = threading.Thread(target=self.file_queue_processor)
        self.file_queue_thread.start()
        services = self.service_manager.list_services()
        if services is not None:
            for service in services:
                self.database.ensure_service(service.name)
        threading.Thread(target=self.queue_processor).start()
        with open(".agent_state", "w") as f:
            f.write("RUNNING")
        while not self.stop:
            try:
                data = self.client.recv(1024).decode()
                if data:
                    for line in data.split("\n"):
                        if line:
                            debug(f"Received message: {line}... Handling...")
                            self.message_queue.put(json.loads(line))
                else:
                    self.stop = True
            except Exception as e:
                log_error(f"Error: {str(e)}")
                self.stop = True

    def sync_services(self):
        if AgentConfig.instance:
            sync(
                f"{self.config.server['host']}:{self.config.server['fsport']}",
                self.config.services_folder,
                self.config.fileserver_auth,
                AgentConfig.instance.cert_path
            )

    def file_queue_processor(self):
        while not self.stop:
            entry = self.file_queue.get()
            if entry:
                self.backup_manager.sync_file_to_storage(entry.service, entry.file_path, entry.file_name, self.transfer_socket_send)
                self.file_queue.task_done()
            time.sleep(0.1)

    def connect_transfer_socket(self, role: str):
        if AgentConfig.instance:
            transfer_socket = self.ssl_context.wrap_socket(socket.socket(socket.AF_INET, socket.SOCK_STREAM), server_hostname=AgentConfig.instance.server["host"])
            transfer_socket.connect((AgentConfig.instance.server["host"], AgentConfig.instance.backup_relay_port))
            payload = f"full:{role}:{self.config.server['api_key']}"
            transfer_socket.send(payload.encode())
            if transfer_socket.recv(1024).decode() != "OK":
                raise ConnectionError("Failed to connect to transfer socket")
            return transfer_socket
        else:
            raise RuntimeError("AgentConfig not initialized")

    def connect_transfer_sockets(self):
        self.transfer_socket_recv = self.connect_transfer_socket(role="recv")
        self.transfer_socket_send = self.connect_transfer_socket(role="send")
        self.transfer_socket_thread = threading.Thread(target=self.backup_manager.handle_as_storage, args=(self.transfer_socket_recv,))
        self.transfer_socket_thread.start()

    def queue_processor(self):
        while not self.stop:
            message = self.message_queue.get()
            if message:
                def handle_response(uuid):
                    response = self.handle_message(message)
                    if response:
                        if not response.get("r_uuid"):
                            response["r_uuid"] = uuid
                            with self.client_send_lock:
                                self.client.sendall((json.dumps(response) + "\n").encode())
                    self.thread_pool.remove(threading.current_thread())

                self.thread_pool.append(
                    threading.Thread(
                        target=handle_response,
                        args=(message.get("r_uuid", uuid.uuid4()),),
                        daemon=True
                    )
                )
                self.thread_pool[-1].start()

                self.message_queue.task_done()
            time.sleep(0.1)
    def add_file_to_queue(self, file_path: str, file_name: str, service: str):
        self.file_queue.put(FileQueueEntry(service, file_name, file_path))

    def send_message(self, message: dict):
        with self.client_send_lock:
            self.client.sendall((json.dumps(message) + "\n").encode())

    def init_services(self):
        self.service_manager = ServiceManager(self.config.services_folder, self.add_file_to_queue, self.send_message)

    def handle_message(self, message: dict) -> dict | None:
        try:
            if message.get("type") == "list_services":
                services = self.service_manager.list_services()
                return_datas = []
                for service in services:
                    return_datas.append(
                        {
                            "name": service.name,
                            "is_running": service.running,
                            "is_healthy": service.healthy,
                            "raw_data": service.line_str,
                        }
                    )
                return {"type": "list_services_report", "services": return_datas}
            elif message.get("type") == "keepalive":
                self.last_keepalive = time.time()
                return {"type": "keepalive_report"}
            elif message.get("type") == "sync_files":
                self.sync_services()
                return {"type": "sync_files_report"}
            elif message.get("type") == "is_service_running":
                service_name = message.get("service", "")
                is_running = self.service_manager.is_running(service_name)
                return {
                    "type": "is_service_running_report",
                    "service": service_name,
                    "is_running": is_running,
                }
            elif message.get("type") == "start_service":
                service_name = message.get("service", "")
                error, msg = self.service_manager.start(service_name)
                return {
                    "type": "start_service_report",
                    "service": service_name,
                    "error": error,
                    "message": msg,
                }
            elif message.get("type") == "stop_service":
                service_name = message.get("service", "")
                error, msg = self.service_manager.stop(service_name)
                return {
                    "type": "stop_service_report",
                    "service": service_name,
                    "error": error,
                    "message": msg,
                }
            elif message.get("type") == "restart_service":
                service_name = message.get("service", "")
                error, msg = self.service_manager.restart(service_name)
                return {
                    "type": "restart_service_report",
                    "service": service_name,
                    "error": error,
                    "message": msg,
                }
            elif message.get("type", "") == "rewrite_config":
                path = os.path.join(
                    self.config.services_folder, message.get("service", "")
                )
                if not os.path.exists(
                    os.path.join(path, message.get("path", "") + ".sample")
                ):
                    log_error(
                        f"Path does not exist: {os.path.join(path, message.get('path', ''))}"
                    )
                    return {
                        "type": "rewrite_config_report",
                        "success": False,
                    }
                else:
                    cfg_original_path = message.get("path", "") + ".sample"
                    cfg_new_path = message.get("path", "")
                    success = True
                    try:
                        shutil.copyfile(os.path.join(path, cfg_original_path), os.path.join(path, cfg_new_path))
                    except Exception as e:
                        log_error(f"Error occurred while copying config file: {e}")
                        success = False
                    return {
                        "type": "rewrite_config_report",
                        "success": success,
                    }
            elif message.get("type", "") == "run_service_command":
                path = os.path.join(
                    self.config.services_folder, message.get("service", "")
                )
                commands = message.get("commands", [])
                return_codes = []
                success = True
                for command in commands:
                    return_code = run_command(command, path)
                    if return_code != 0:
                        success = False
                    return_codes.append(return_code)
                return {
                    "type": "run_service_command_report",
                    "path": message.get("path", ""),
                    "return_codes": return_codes,
                    "success": success,
                }
            elif message.get("type", "") == "init_backup_as_service":
                service = message.get("service", "")
                datas = message.get("datas", [])
                type = message.get("b_type", "")
                transaction_id = message.get("transaction_id", "")
                success = self.backup_manager.init_backup_as_service(service, datas, type, transaction_id, self.ssl_context)
                return {
                    "type": "init_backup_as_sender_report",
                    "success": success,
                }
            elif message.get("type", "") == "init_backup_as_storage":
                path = message.get("path", "")
                service = message.get("service", "")
                type = message.get("b_type", "")
                config = message.get("config", {})
                transaction_id = message.get("transaction_id", "")
                self.backup_manager.init_backup_as_storage(path, service, type, config, transaction_id, self.ssl_context)
                return {
                    "type": "init_backup_as_receiver_report",
                    "success": True,
                }
            elif message.get("type", "") == "init_restore_as_service":
                service = message.get("service", "")
                datas = message.get("datas", [])
                transaction_id = message.get("transaction_id", "")
                success = self.backup_manager.init_restore_as_client(service, transaction_id, datas, self.ssl_context, os.path.join(self.config.services_folder, service))
                return {
                    "type": "init_restore_as_sender_report",
                    "success": success,
                }
            elif message.get("type", "") == "init_sync_as_storage":
                service = message.get("service", "")
                transaction_id = message.get("transaction_id", "")
                path = message.get("path", "")
                datas = message.get("datas", [])
                sync_path = os.path.join(path, service, "sync")
                if not os.path.exists(sync_path):
                    os.makedirs(sync_path)
                success = self.backup_manager.init_restore_as_client(service, transaction_id, datas, self.ssl_context, sync_path)
                return {
                    "type": "init_sync_as_storage_report",
                    "success": success,
                }
            elif message.get("type", "") == "init_sync_as_service":
                service = message.get("service", "")
                transaction_id = message.get("transaction_id", "")
                datas = message.get("datas", [])
                success = self.backup_manager.init_full_sync_as_service(service, transaction_id, datas, self.ssl_context)
                return {
                    "type": "init_sync_as_service_report",
                    "success": success,
                }
            elif message.get("type", "") == "init_restore_as_storage":
                path = message.get("path", "")
                service = message.get("service", "")
                type = message.get("b_type", "")
                config = message.get("config", {})
                transaction_id = message.get("transaction_id", "")
                backup_id = config.get("backup_id", None)
                success, response = self.backup_manager.init_restore_as_storage(path, service, type, config, transaction_id, self.ssl_context, backup_id)
                debug(f"Restore finished: {response}")
                return {
                    "type": "init_restore_as_receiver_report",
                    "success": success,
                    "message": response,
                }
            elif message.get("type", "") == "list_available_backups":
                path = message.get("path", "")
                with_size = message.get("size", False)
                backups = self.backup_manager.list_backups(path, with_size)
                return {
                    "type": "list_available_backups_report",
                    "success": True,
                    "backups": backups,
                }
            elif message.get("type", "") == "init_sync_restore_as_storage":
                service = message.get("service", "")
                transaction_id = message.get("transaction_id", "")
                path = message.get("path", "")
                success = self.backup_manager.init_sync_restore_as_storage(service, path, transaction_id, self.ssl_context)
                return {
                    "type": "init_sync_restore_as_storage_report",
                    "success": success,
                }
            elif message.get("type", "") == "init_sync_restore_as_service":
                service = message.get("service", "")
                transaction_id = message.get("transaction_id", "")
                datas = message.get("datas", [])
                success = self.backup_manager.init_restore_as_client(service, transaction_id, datas, self.ssl_context, os.path.join(self.config.services_folder, service))
                return {
                    "type": "init_sync_restore_as_service_report",
                    "success": success,
                }
            elif message.get("type", "") == "delete_backups":
                path = message.get("path", "")
                service_id = message.get("service_id", "")
                type = message.get("b_type", "")
                success = self.backup_manager.delete_backups(path, service_id, type)
                return {
                    "type": "delete_backups_report",
                    "success": success,
                }
            elif message.get("type", "") == "gen_config":
                path = os.path.join(
                    self.config.services_folder, message.get("service", "")
                )
                commands = message.get("commands", [])
                return_codes = []
                success = True
                for command in commands:
                    if not os.path.exists(
                        os.path.join(path, message.get("path", "") + ".sample")
                    ):
                        log_error(
                            f"Path does not exist: {os.path.join(path, message.get('path', ''))}"
                        )
                        success = False
                    else:
                        cfg_new_path = message.get("path", "")
                        command_copy = "\n"
                        if not os.path.exists(os.path.join(path, cfg_new_path)):
                            cfg_original_path = message.get("path", "") + ".sample"
                            command_copy = f"""
                            cp {cfg_original_path} {cfg_new_path}
                            """
                        return_code = run_command(
                            command.split("::")[0]
                            + "::"
                            + command_copy
                            + command.split("::", 1)[1],
                            path,
                        )
                        return_codes.append(str(return_code))
                        if return_code != 0:
                            success = False
                    return {
                        "type": "gen_config_report",
                        "path": message.get("path", ""),
                        "return_codes": return_codes,
                        "success": success,
                    }
            elif message["type"] == "check_storage":
                if message.get("path") is None:
                    return {
                        "type": "check_storage",
                        "success": False,
                        "invalid": True,
                    }
                valid = self.backup_manager.verify_path(message.get("path", ""), message.get("can_create", True))
                return {
                    "type": "check_storage",
                    "success": True,
                    "invalid": not valid,
                }
            elif message["type"] == "start_watchfiles":
                service = message.get("service", "")
                datas = message.get("datas", [])
                if self.watchfile_threads.get(service):
                    self.watchfile_threads[service].join(timeout=1)
                thread = self.service_manager.start_watchfiles(service, datas)
                self.watchfile_threads[service] = thread
                return {
                    "type": "start_watchfiles_report",
                    "success": True,
                }
            elif message.get("type") == "delete_file":
                service = message.get("service", "")
                file_path = message.get("file", "")
                path = message.get("path", "")
                self.service_manager.delete_file(service, file_path, path)
                return {
                    "type": "delete_file_report",
                    "success": True,
                }
            else:
                return {
                    "type": "error",
                    "success": False,
                    "message": f"Unknown command {message['type']}",
                }
        except Exception:
            log_error(
                f"ERROR while handling message {message}: {traceback.format_exc()}"
            )
            self.stop = True
