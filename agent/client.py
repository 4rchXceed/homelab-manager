import json
import os
import threading
import time
import uuid
from queue import Queue
from socket import socket

from config import AgentConfig
from config_gen.runner import run_command
from fileclient.sync import sync
from messaging.log import debug, info
from messaging.report_error import log_error, report_error
from runner.service_manager import ServiceManager


class Client:
    def __init__(
        self,
    ):
        if not AgentConfig.instance:
            raise RuntimeError("AgentConfig not initialized")
        self.config = AgentConfig.instance
        self.message_queue = Queue()
        self.init_services()
        self.last_keepalive = time.time()
        self.keepalive_thread = None

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
        self.client = socket()
        try:
            self.client.connect((host, port))
        except Exception as e:
            report_error(
                "HOMELAB-MGR SERVER DOWN/CAN'T CONNECT",
                f"ERROR: {str(e)}. All future requests will fail.",
                3,
            )
            raise ConnectionError(f"Failed to connect: {str(e)}")
        self.client.send(api_key.encode())
        data = self.client.recv(1024).decode()
        if data != "OK":
            report_error(
                f"Homelab-MGR: Client {self.config.id} ({self.config.name}): connection issue",
                f"Failed to connect: {data}. Probably due to wrong API key. This instance will not receive any commands.",
                2,
            )
            raise ConnectionError(f"Failed to connect: {data}")
        self.stop = False
        info("Connected to server.")
        info(
            "Syncronizing services folder (this might take a while if there are many services)..."
        )
        self.keepalive_thread = threading.Thread(target=self.keepalive_check)
        self.keepalive_thread.start()
        self.sync_services()
        threading.Thread(target=self.queue_processor).start()
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
        sync(
            f"{self.config.server['host']}:{self.config.server['fsport']}",
            self.config.services_folder,
        )

    def queue_processor(self):
        while not self.stop:
            message = self.message_queue.get()
            if message:
                response = self.handle_message(message)
                if response:
                    if not response.get("r_uuid"):
                        response["r_uuid"] = message.get("r_uuid", uuid.uuid4())
                    print(f"Sending message: {response}")
                    self.client.sendall((json.dumps(response) + "\n").encode())
            self.message_queue.task_done()

    def init_services(self):
        self.service_manager = ServiceManager(self.config.services_folder)

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
            elif message.get("type", "") == "gen_config":
                path = os.path.join("services", message.get("service", ""))
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
                        cfg_original_path = message.get("path", "") + ".sample"
                        cfg_new_path = message.get("path", "")
                        command_copy = f"""
                        cp {cfg_original_path} {cfg_new_path}
                        """
                        return_code = run_command(command_copy + command, path)
                        return_codes.append(str(return_code))
                        if return_code != 0:
                            success = False
                        return {
                            "type": "gen_config_report",
                            "path": message.get("path", ""),
                            "return_codes": return_codes,
                            "success": success,
                        }
        except Exception as e:
            log_error(f"ERROR while handling message {message}: {str(e)}")
            self.stop = True
