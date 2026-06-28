import json
import socket
import threading
import time
import traceback
import uuid
from queue import Queue

from database.models import Server
from error.exceptions import MissingConfigException, ProgramStateError, TimeoutException
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
        self.keepalive_thread = None

        # FXXX circular imports
        from services.service import ServerService

        self.ServerService = ServerService

    # This is a shared resource between threads, and since every thread has it's own db session, this will f- up everything
    @property
    def db_server(self) -> Server | None:
        return (
            self.context.database.session.query(Server)
            .filter_by(id_str=self.id)
            .first()
        )

    def reload(self, _):
        self.sync_to_db()

    def init(self) -> None:
        connected = self.init_connection()
        if connected:
            self.sync_to_db()
            self.start_receiving()
            self.start_processing()
            self.context.event_manager.register_event("config_reload", self.reload)
            self.keepalive_thread = threading.Thread(target=self.keepalive)
            self.keepalive_thread.start()
            logger.info(f"Agent initialized with connection from {self.addr}")
        else:
            logger.warning(f"Agent {self.addr} tried to connect with wrong api key")
            self.context.app.agents.remove(
                self
            )  # Remove self from agent, because self is dirty

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

            db_server = (
                self.context.database.session.query(Server)
                .filter_by(id_str=self.id)
                .first()
            )
            if not db_server:
                db_server = Server(
                    id_str=self.id,
                    name=self.name,
                    ip=self.ip,
                    description=description,
                    disabled=False,
                )
                self.context.database.session.add(db_server)
                self.context.database.session.commit()
                self.migrate()
            else:
                if db_server.name != self.server["name"]:
                    db_server.name = self.server["name"]
                    self.context.database.session.commit()
                if db_server.ip != self.server["ip"]:
                    db_server.ip = self.server["ip"]
                    self.context.database.session.commit()
            self.db_server_id = db_server.id

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

    # @staticmethod
    # def check_ip(server: dict, addr: tuple) -> bool:
    #     if socket.gethostbyname(server["ip"]) != socket.gethostbyname(addr[0]):
    #         return False
    #     return True

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
        stop = False
        buffer = ""
        while not self.context.kill_switch and not stop:
            try:
                buffer += self.socket.recv(1024).decode()
                while "\n" in buffer:
                    line = buffer.split("\n")[0]
                    if buffer.count("\n") > 0:
                        buffer = buffer.split("\n", 1)[1]
                    try:
                        if line:
                            self.handle_request(json.loads(line))
                    except json.JSONDecodeError as e:
                        logger.error(f"JSON decode error: {e}")
            except Exception as e:
                self.handle_disconnect()
                stop = True
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
        except Exception:
            logger.error(f"Error during handle_request: {traceback.format_exc()}")

    def process_queue(self) -> None:
        while not self.context.kill_switch:
            if self.request_queue.qsize() > 0:
                request = self.request_queue.get(timeout=1)
                if request:
                    self.send(request)
                # OLD: 150% CPU usage
                # else:
                #     time.sleep(0.1)
            # New: 0.5% CPU usage
            time.sleep(0.1)

    def send(self, data: dict) -> None:
        if not data.get("r_uuid"):
            request_uuid = uuid.uuid4()
            data["r_uuid"] = str(request_uuid)
        self.requests[data["r_uuid"]] = data
        try:
            self.socket.sendall((json.dumps(data) + "\n").encode())
        except Exception as e:
            logger.error(f"Error during send: {e}")

    def send_pingpong(self, data: dict, timeout: float = 10.0) -> dict:
        request_id = data.get("r_uuid", str(uuid.uuid4()))
        data["r_uuid"] = request_id
        datas = None

        def handle_request_recv(data):
            nonlocal datas
            if data.get("r_uuid") == request_id:
                datas = data

        self.context.event_manager.register_event("request_recv", handle_request_recv)

        self.send(data)

        start_time = time.time()
        while datas is None:
            time.sleep(0.05)
            if time.time() - start_time > timeout:
                break

        self.context.event_manager.unregister_event("request_recv", handle_request_recv)

        if datas is None:
            raise TimeoutException(f"Request {request_id} timed out ({timeout}s)")

        return datas

    def close(self) -> None:
        try:
            self.socket.shutdown(socket.SHUT_RDWR)
            self.socket.close()
            self.context.agents.remove(self)
            self.context.app.agents.remove(self)
        except Exception as e:
            logger.error(f"Error during close: {e}")

    def start_service(self, service_name: str, timeout=120) -> tuple[bool, str]:
        service = self.ServerService.get_from_id_str(service_name)
        if not service:
            data_folders = []
        else:
            data_folders = service.datas
            data_folders.extend(service.ignored_folders)
        data = {
            "type": "start_service",
            "service": service_name,
            "data_folders": data_folders,
        }
        datas = self.send_pingpong(data, timeout=timeout)
        return datas.get("error", True), datas.get("message", "Generic error")

    def stop_service(self, service_name: str, timeout=120) -> tuple[bool, str]:
        data = {"type": "stop_service", "service": service_name}
        datas = self.send_pingpong(
            data,
            timeout=timeout,  # A service stop can take a while
        )
        return datas.get("error", True), datas.get("message", "Generic error")

    def restart_service(self, service_name: str) -> tuple[bool, str]:
        data = {"type": "restart_service", "service": service_name}
        datas = self.send_pingpong(data)
        return datas.get("error", True), datas.get("message", "Generic error")

    def list_services(self) -> list[dict]:
        data = {"type": "list_services"}
        datas = self.send_pingpong(data)
        return datas.get("services", [])

    def is_service_running(self, service_name: str) -> bool:
        data = {"type": "is_service_running", "service": service_name}
        datas = self.send_pingpong(data)
        return datas.get("is_running", False)

    def handle_disconnect(self) -> None:
        logger.warning(f"Agent {self.name} disconnected... Cleanup")
        self.close()

    def keepalive(self) -> None:
        stop = False
        while not self.context.kill_switch and not stop:
            try:
                self.send_pingpong({"type": "keepalive"})
                time.sleep(self.context.config_general.keepalive_interval)
            except ConnectionResetError:
                stop = True
            except Exception as e:
                logger.error(f"Error during keepalive: {e}")
                stop = True
        self.handle_disconnect()

    def sync_files(self) -> None:
        data = {"type": "sync_files"}
        self.send_pingpong(data)

    @staticmethod
    def get_from_id_str(id: str) -> "None | Agent":
        context = get_current_context()
        for agent in context.agents:
            if agent.server and agent.server.get("id") == id:
                return agent
        return None
