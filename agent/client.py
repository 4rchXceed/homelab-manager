import json
import os
import threading
from queue import Queue
from socket import socket

from config import AgentConfig
from config_gen.runner import run_command
from messaging.log import debug, info
from messaging.report_error import log_error, report_error


class Client:
    def __init__(
        self,
    ):
        if not AgentConfig.instance:
            raise RuntimeError("AgentConfig not initialized")
        self.config = AgentConfig.instance
        self.message_queue = Queue()

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
        threading.Thread(target=self.queue_processor).start()
        while not self.stop:
            try:
                data = self.client.recv(1024).decode()
                if data:
                    debug(f"Received message: {data}... Handling...")
                    self.message_queue.put(json.loads(data))
                else:
                    self.stop = True
            except Exception as e:
                log_error(f"Error: {str(e)}")
                self.stop = True

    def queue_processor(self):
        while not self.stop:
            message = self.message_queue.get()
            if message:
                message = self.handle_message(message)
                if message:
                    print(f"Sending message: {message}")
                    self.client.sendall(json.dumps(message).encode())
            self.message_queue.task_done()

    def handle_message(self, message: dict) -> dict | None:
        try:
            if message.get("type", "") == "gen_config":
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
                        return_codes.append(return_code)
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
