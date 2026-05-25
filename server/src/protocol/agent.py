import json
import socket
import threading
import time
import uuid
from queue import Queue

from database.models import Server
from error.exceptions import MissingConfigException, ProgramStateError
from helpers import get_current_context
from logger import logger


class Agent:
    def __init__(self, socket: socket.socket, addr: tuple) -> None:
        self.socket = socket
        self.addr = addr
        self.context = get_current_context()
        self.request_queue = Queue()  # Custom queue for every agent
        self.requests = {}
        self.server = None

    def init(self) -> None:
        connected = self.init_connection()
        if connected:
            self.sync_to_db()
            self.start_receiving()
            self.start_processing()
            logger.info(f"Agent initialized with connection from {self.addr}")

    def sync_to_db(self) -> None:
        if self.server:
            self.id = self.server.get("id")
            self.name = self.server.get("name")
            self.ip = self.server.get("ip")
            description = self.server.get("description", "No description provided")
            if not self.id:
                raise MissingConfigException("servers.$.id")
            if not self.name:
                raise MissingConfigException("servers.$.name")
            if not self.ip:
                raise MissingConfigException("servers.$.ip")

            self.db_server = (
                self.context.database.session.query(Server)
                .filter_by(id_str=self.id)
                .first()
            )
            if not self.db_server:
                self.db_server = Server(
                    id_str=self.id, name=self.name, ip=self.ip, description=description
                )
                self.context.database.session.add(self.db_server)
                self.context.database.session.commit()
            self.migrate()

    def migrate(self) -> None:
        if not self.db_server or not self.server:
            return
        if not self.db_server.api_key:
            raise ProgramStateError("How did we get here?")
        if self.db_server.description != self.server["description"]:
            self.db_server.description = self.server["description"]
            self.context.database.session.commit()
        if self.db_server.ip != self.server["ip"]:
            self.db_server.ip = self.server["ip"]
            self.context.database.session.commit()
        if self.db_server.name != self.server["name"]:
            self.db_server.name = self.server["name"]
            self.context.database.session.commit()

    def start_receiving(self) -> None:
        receive_thread = threading.Thread(target=self.receive)
        receive_thread.start()

    def start_processing(self) -> None:
        process_thread = threading.Thread(target=self.process_queue)
        process_thread.start()

    def check_api_key(self, api_key: str) -> dict | None:
        server = (
            self.context.database.session.query(Server)
            .filter_by(api_key=api_key)
            .first()
        )
        if server:
            for server_obj in self.context.config_servers.servers:
                if server_obj["id"] == server.id_str:
                    return server_obj
        return None

    def init_connection(self) -> bool:
        try:
            data = self.socket.recv(1024).decode()
            if data:
                self.server = self.check_api_key(data)
                if self.server:
                    self.socket.sendall(b"OK")
                    return True
                else:
                    self.socket.sendall(b"ERROR")
        except Exception as e:
            logger.error(f"Error during connection init: {e}")
            self.socket.sendall(b"ERROR")
        return False

    def receive(self) -> None:
        while not self.context.kill_switch:
            try:
                data = self.socket.recv(1024).decode()
                if data:
                    try:
                        self.handle_request(json.loads(data))
                    except json.JSONDecodeError as e:
                        logger.error(f"JSON decode error: {e}")
            except Exception as e:
                logger.error(f"Error during receive: {e}")

    def handle_request(self, data: dict) -> None:
        try:
            self.context.event_manager.trigger_event("request_recv", data)
            request_uuid = data.pop("r_uuid")
            request = self.requests.pop(request_uuid)
            if not request:
                logger.warning(f"Received request with unknown UUID: {request_uuid}")
                return

            if request.get("type", "") == "gen_config_report":
                if data.get("success", False):
                    logger.info(
                        f"[{data.get('path', 'Unknown')}]: File generated successfully."
                    )
                else:
                    logger.error(
                        f"[{data.get('path', 'Unknown')}]: File generation failed. Details: {', '.join(data.get('return_codes', []))}"
                    )
        except Exception as e:
            logger.error(f"Error during handle_request: {e}")

    def process_queue(self) -> None:
        while not self.context.kill_switch:
            if not self.request_queue.empty():
                request = self.request_queue.get(timeout=1)
                if request:
                    self.send(request)
                else:
                    time.sleep(0.1)

    def send(self, data: dict) -> None:
        if not data.get("r_uuid"):
            request_uuid = uuid.uuid4()
            data["r_uuid"] = str(request_uuid)
        self.requests[data["r_uuid"]] = data
        try:
            self.socket.sendall(json.dumps(data).encode())
        except Exception as e:
            logger.error(f"Error during send: {e}")

    def close(self) -> None:
        try:
            self.socket.shutdown(socket.SHUT_RDWR)
            self.socket.close()
        except Exception as e:
            logger.error(f"Error during close: {e}")
