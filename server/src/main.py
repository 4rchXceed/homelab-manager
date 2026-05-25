import os
import shlex
import socket
import threading
from queue import Queue
from typing import Callable

from command_context import CommandContext
from config.general import GeneralConfig
from config.load import load_config
from config.servers import ConfigServers
from config_gen.generators import Generators
from context import HLMContext
from database.database import DatabaseEngine
from fileserver.rclone import FileServer
from helpers import set_current_context
from logger import logger
from plugins.commands.library import COMMANDS
from plugins.variable_providers.library import VARIABLE_PROVIDERS
from protocol.agent import Agent
from services.service import ServerService


class ServerApp:
    def __init__(self) -> None:
        self.config_raw = load_config()
        self.config_general = GeneralConfig(self.config_raw)
        self.config_servers = ConfigServers(self.config_raw)
        self.socket_comm_host = socket.gethostname()
        self.socket_comm_port = self.config_general.server_port
        self.server_socket_comm = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.server_socket_comm.bind((self.socket_comm_host, self.socket_comm_port))
        self.agents_message_queue = Queue()
        self.agents: list[Agent] = []
        self.temp_threads = []
        self.services: dict[str, ServerService] = {}

    def init(self) -> None:
        logger.info("Booting...")
        self.init_db()
        self.init_plugins()
        self.context = HLMContext(
            self.db,
            self.generators,
            self.var_providers,
            self.agents_message_queue,
            self.config_general,
            self.config_servers,
            self,
        )
        set_current_context(self.context)
        self.init_services()
        self.init_file_server()
        self.init_communication_socket()
        self.unix_socket_server()

    def temp_thread_wrapper(self, target: Callable) -> None:
        target()
        self.temp_threads.remove(threading.current_thread())

    def handle_socket_clients(self) -> None:
        while not self.context.kill_switch:
            try:
                conn, addr = self.server_socket_comm.accept()
                agent = Agent(conn, addr)
                init_thread = threading.Thread(
                    target=self.temp_thread_wrapper, args=(agent.init,)
                )
                init_thread.start()
                self.temp_threads.append(init_thread)
                self.agents.append(agent)
            except Exception as e:
                logger.error(f"Error accepting connection: {e}")

    def init_communication_socket(self) -> None:
        logger.debug("Initializing communication socket server...")
        self.server_socket_comm.listen(2)
        self.socket_clients_handler = threading.Thread(
            target=self.handle_socket_clients
        )
        self.socket_clients_handler.start()
        logger.info("Communication socket server initialized!")

    def init_db(self) -> None:
        logger.debug("Initializing database connection...")
        self.db = DatabaseEngine(self.config_general.database)
        logger.debug("Database connected!")

    def init_plugins(self) -> None:
        logger.debug("Initializing plugins...")
        self.generators = Generators(self.config_raw["generators"])
        self.var_providers = VARIABLE_PROVIDERS
        logger.debug("Plugins initialized!")

    def init_file_server(self) -> None:
        logger.debug("Initializing file server...")
        self.file_server = FileServer(self.config_general.services_folder)
        self.file_server.start()
        logger.debug("File server initialized!")

    def init_services(self) -> None:
        logger.debug("Initializing services...")
        services_obj = self.config_raw.get("services", {})

        for service, service_config in services_obj.items():
            self.services[service] = ServerService(service, service_config)
        logger.debug("Services initialized!")

    def unix_socket_server(self) -> None:
        socket_path = self.config_general.unix_socket_path
        try:
            os.unlink(socket_path)
        except OSError:
            if os.path.exists(socket_path):
                raise Exception(
                    "It seems like an instance of the server daemon is already running..."
                )

        self.cli_server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.cli_server.bind(socket_path)
        self.cli_server.listen(1)
        self.cli_connections_threads = []
        logger.debug("Unix socket server initialized!")
        while not self.context.kill_switch:
            try:
                conn, addr = self.cli_server.accept()
                thread = threading.Thread(
                    target=self.handle_cli_connection, args=(conn, addr)
                )
                thread.start()
                self.cli_connections_threads.append(thread)
            except Exception as e:
                logger.error(f"Error accepting connection (CLI Server): {e}")

    def handle_cli_connection(self, conn: socket.socket, addr: tuple) -> None:
        while not self.context.kill_switch:
            data = conn.recv(1024)
            if not data:
                break
            logger.debug(f"Received command: {data}")
            try:
                command = data.decode().strip()
                result = self.execute_command(command, conn)
                if result:
                    conn.sendall("OK\n".encode())
                else:
                    conn.sendall("ERROR\n".encode())
            except Exception as e:
                conn.sendall(f"Exception while handling command: {e}".encode())
                conn.sendall("ERROR".encode())
                logger.error(f"Error handling command: {e}")

    def execute_command(self, command: str, socket: socket.socket) -> bool:
        parts = shlex.split(command)
        if not parts:
            socket.sendall("Error: No command provided".encode())
            return False
        action = parts[0]
        command_handler = None
        if action == "help":
            for cmd in COMMANDS:
                if cmd.NAME == parts[1]:
                    command_handler = cmd
                    break
            if not command_handler:
                socket.sendall(
                    f"Error: Unknown command '{parts[1]}'. Cannot give help for unknown commands".encode()
                )
                return False
            socket.sendall(command_handler.get_help().encode())
            return True

        if action == "shutdown":
            socket.sendall("Shutting down...".encode())
            self.shutdown()
            return True
        for cmd in COMMANDS:
            if cmd.NAME == action:
                command_handler = cmd
                break

        if not command_handler:
            socket.sendall(f"Error: Unknown command '{action}'".encode())
            return False
        command_context = CommandContext()

        def output(msg: str):
            socket.sendall(msg.encode())

        def input(msg: str):
            socket.sendall((msg + "::INPUT").encode())
            return socket.recv(1024).decode().strip()

        command_context.output_print = output
        command_context.output_input = input
        return command_handler.execute(parts[1:], command_context)

    def shutdown(self):
        logger.info("Shutting down...")
        self.context.kill_switch = True
        self.file_server.stop()
        for thread in self.temp_threads:
            thread.join()
        self.cli_server.shutdown(socket.SHUT_RDWR)
        self.cli_server.close()
        for agent in self.agents:
            agent.close()

        logger.info("Shutdown complete")


if __name__ == "__main__":
    app = ServerApp()
    app.init()
