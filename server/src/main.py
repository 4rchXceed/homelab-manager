import os
import shlex
import socket
import threading
import time
import traceback
from queue import Queue
from typing import Callable

from command_context import CommandContext
from config.general import GeneralConfig
from config.load import load_config, load_new_config
from config.runtime import RuntimeConfig
from config.servers import ConfigServers
from config_gen.generators import Generators
from context import HLMContext
from database.database import DatabaseEngine
from database.models import NeedsUpdate, Server, Service
from dotenv import load_dotenv
from fileserver.rclone import FileServer
from helpers import set_current_context
from logger import logger
from plugins.commands.library import COMMANDS
from plugins.variable_providers.library import VARIABLE_PROVIDERS
from protocol.agent import Agent
from services.service import ServerService
from sqlalchemy.orm import Session


class ServerApp:
    def __init__(self) -> None:
        config_raw, config_raw_str = load_config()
        self.config_raw = config_raw
        self.config_raw_str = config_raw_str
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
        self.init_thread = None
        self.postinit_completed = False

    def check_deleted_services(self, cmd_context: CommandContext | None = None) -> None:
        services = (
            self.context.database.session.query(Service).filter_by(disabled=False).all()
        )
        for service in services:
            if service.id_str not in self.config_raw.get("services", {}).keys():
                msg = f"!! Service {service.id_str} has been deleted from config...Disabling it...."
                if cmd_context:
                    cmd_context.output_print(msg)
                else:
                    logger.warning(msg)
                server_service = ServerService.get_from_id_str(service.id_str)
                if server_service:
                    server_service.unassign(cmd_context)
                service.disabled = True
                self.context.database.session.commit()
                del self.services[service.id_str]

    def reload_config(self, cmd_context: CommandContext | None) -> str:
        if cmd_context is None:

            def fake_input(msg):
                raise Exception("Non-interactive env")

            cmd_context = CommandContext()
            cmd_context.output_input = fake_input
        self.config_raw, self.config_raw_str = load_new_config()

        if self.config_general.dict != self.config_raw.get("config"):
            cmd_context.output_print(
                "!! you changed the general config (config: {...}). You must restart the server to apply the new config!"
            )
        if len(self.config_raw.get("servers", [])) < len(self.config_servers.servers):
            cmd_context.output_print(
                "!! You deleted an agent. FOR NOW, you need to restart the server to apply the config!!!."
            )  # TODO: Handle this
        self.context.event_manager.trigger_event("config_reload", self.config_raw)
        for agent in self.agents:
            agent.sync_files()
        self.config_servers.reload(self.config_raw)
        for service_id, service in self.services.items():
            service_data = self.config_raw.get("services", {}).get(service_id)
            if service_data:
                service.reload(service_data)
                service.finish_init(cmd_context)

        for service_id, config_service in self.config_raw.get("services", {}).items():
            if service_id not in self.services.keys():
                cmd_context.output_print(f"Service {service_id} added to the config!")
                self.services[service_id] = ServerService(service_id, config_service)
        self.check_deleted_services(cmd_context)
        self.context.event_manager.trigger_event("config_synced")
        self.check_ip_updates(cmd_context=cmd_context)

        return "Reload DONE"

    def wait_init_complete(self) -> None:
        number_agents = (
            self.context.database.session.query(Server)
            .filter_by(disabled=False)
            .count()
        )
        timeout = self.config_general.startup_timeout
        logger.info(f"Waiting for {number_agents} agents to initialize...")
        start_time = time.time()
        while len(self.agents) < number_agents:
            time.sleep(0.1)
            if time.time() - start_time > timeout:
                logger.error(f"Startup timeout: {timeout}s")
                raise TimeoutError(
                    "Startup timeout"
                )  # TimeoutException is reserved for transport timeouts
        self.sync_services(
            can_do_actions=False,
            debug=True,
        )  # Prevent automated actions during init. Debug is enable by default so the owner can see what's happening in case of issues
        self.context.event_manager.trigger_event("post_init")

        logger.info("Post-init sync complete")
        self.postinit_completed = True
        self.init_thread = None

    def check_ip_updates(
        self,
        restrict_to: None | Agent = None,
        cmd_context: CommandContext | None = None,
    ) -> None:
        has_regenerated = False
        for _, service in self.services.items():
            needs_updates = self.context.database.session.query(NeedsUpdate).filter_by(
                service_trigger_id=service.db_element.id
            )
            for needs_update in needs_updates:
                if not (
                    restrict_to is not None
                    and restrict_to.db_server is not None
                    and needs_update.service_trigger.server_id
                    != restrict_to.db_server.id
                ):
                    service_class = ServerService.get_from_id(
                        service.db_element.id, self.context
                    )
                    if service_class is not None:
                        agent = service_class.get_agent()
                        if agent is not None:
                            if agent.ip != needs_update.last_ip:
                                service_updated_class = ServerService.get_from_id(
                                    needs_update.service_updated_id, self.context
                                )
                                if service_updated_class is not None:
                                    for (
                                        config_file
                                    ) in service_updated_class.config_files:
                                        config_file.regenerate(cmd_context)
                                        has_regenerated = True
        if has_regenerated:
            self.context.event_manager.trigger_event("config_synced")

    def on_config_synced(self) -> None:
        with open(
            os.getenv("CONFIG_FILE", "../conf/config.json") + ".donottouch.internal",
            "w",
            encoding="utf-8",
        ) as f:
            f.write(self.config_raw_str)

    def init(self) -> None:
        logger.info("Booting...")
        self.init_db()
        self.init_plugins()
        self.runtime_config = RuntimeConfig()
        self.context = HLMContext(
            self.db,
            self.generators,
            self.var_providers,
            self.agents_message_queue,
            self.config_general,
            self.config_servers,
            self.runtime_config,
            self,
        )
        set_current_context(self.context)
        for db_server in (
            self.context.database.session.query(Server).filter_by(disabled=False).all()
        ):
            found = False
            i = 0
            while not found and i < len(self.config_servers.servers):
                if self.config_servers.servers[i].get("id", "") == db_server.id_str:
                    found = True
                else:
                    i += 1
            if not found:
                # Server has been deleted, disable it from the database
                db_server.disabled = True
                self.context.database.session.commit()
                # del self.config_servers.servers[i] Not needed since it's at startup
        self.init_thread = threading.Thread(target=self.wait_init_complete)
        self.init_thread.start()
        self.context.event_manager.register_event(
            "config_synced", self.on_config_synced
        )
        self.context.event_manager.register_event(
            "service_updated", lambda a: self.check_ip_updates(None, a)
        )
        self.init_services()
        self.init_file_server()
        self.init_communication_socket()
        self.runtime_config.init()
        self.check_ip_updates()
        self.unix_socket_server()  # WARNING: THIS FUNCTION NEVER ENDS (it's a server), DO NOT PUT ANYTHING AFTER THAT

    def temp_thread_wrapper(self, target: Callable) -> None:
        target()
        if threading.current_thread() in self.temp_threads:
            self.temp_threads.remove(threading.current_thread())

    def handle_socket_clients(self) -> None:
        while not self.context.kill_switch:
            try:
                conn, addr = self.server_socket_comm.accept()
                agent = Agent(conn, addr)

                def init_wrapper():
                    agent.init()
                    # If post-initialization is completed, start the sync thread, because else the agent would not be synced
                    # This happens only due to a disconnect from an agent.

                    if self.postinit_completed:
                        sync_thread = threading.Thread(
                            target=self.temp_thread_wrapper, args=(self.sync_services,)
                        )
                        sync_thread.start()
                        self.temp_threads.append(sync_thread)

                init_thread = threading.Thread(
                    target=self.temp_thread_wrapper, args=(init_wrapper,)
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
        self.check_deleted_services()
        logger.debug("Services initialized!")

    def sync_services(
        self,
        can_do_actions: bool = False,
        custom_logger=None,
        debug: bool = False,
        cmd_context: CommandContext | None = None,
    ) -> int:
        def base_log(message):
            if custom_logger:
                custom_logger(message)
            else:
                logger.info(message)

        def log_error(message):
            base_log("ERROR: " + message)

        def log_warning(message):
            base_log("WARNING: " + message)

        base_log(
            "Syncing services... (this will take a while if there are errors / desync, progress will be logged below)"
        )
        errors = 0
        # Agent side
        i = 0
        for agent in self.agents:
            i += 1
            if debug:
                base_log(
                    f"Syncing services from agent {i}/{len(self.agents)}... Phase 1/2"
                )
            if can_do_actions:
                agent.sync_files()
            services = agent.list_services()
            for service in services:
                name = service["name"]
                if service.get("is_running"):
                    if name not in self.services.keys():
                        log_warning(
                            f"Service {name} not found in local services (badly removed from config?), skipping..."
                        )
                        if can_do_actions:
                            log_warning(
                                f"Service {name} can be destroyed! Destroying... This will take a while..."
                            )
                            is_error, error_message = agent.stop_service(
                                name, timeout=120
                            )
                            if is_error:
                                log_error(
                                    f"Failed to stop service {name}: {error_message}"
                                )
                                errors += 1
                        else:
                            log_error(
                                "Could not resolve this issue due to the flag can_do_actions"
                            )
                            errors += 1
                    else:
                        if agent.db_server:
                            server_service = self.services[name]
                            if not server_service.db_element.server:
                                log_warning(
                                    f"Service {name} is not supposed to be running. (Did you start it manually? If so, please use the manager... syncing)"
                                )
                                server_service.db_element.server = agent.db_server
                                session = Session.object_session(
                                    server_service.db_element
                                )
                                if session:
                                    session.commit()
                            else:
                                if (
                                    server_service.db_element.server.id
                                    != agent.db_server.id
                                ):
                                    log_warning(
                                        f"Service {name} is supposed to be running on server {agent.db_server.name}, but is running on server {server_service.db_element.server.name}. (Did you start it manually? If so, please use the manager...)"
                                        + "Generating report of the issue..."
                                    )
                                    real_agent = server_service.get_agent()

                                    if real_agent:
                                        is_running = real_agent.is_service_running(name)
                                        if is_running:
                                            log_error(
                                                "Two instances of the server daemon are running on the same server! PANIC! IDK WHAT TO DO!!!! "
                                                + "Try running: agent:stop agent_id service_id"
                                            )
                                            errors += 1
                                        else:
                                            log_warning(
                                                f"Service {name} is not running on agent {real_agent.name}... Syncing the database..."
                                            )
                                            server_service.db_element.server = (
                                                real_agent.db_server
                                            )
                                            session = Session.object_session(
                                                server_service.db_element
                                            )
                                            if session:
                                                session.commit()
                                    else:
                                        log_error(
                                            f"Could not find agent for server {server_service.db_element.server.name}"
                                        )
                                        errors += 1

        # Database side
        i = 0
        for service_id, service in self.services.items():
            i += 1
            if debug:
                base_log(f"Syncing service {i}/{len(self.services)}... Phase 2/2")
            if service.db_element.server:
                agent = service.get_agent()
                if agent is None:
                    log_error(
                        f"Could not find agent for server {service.db_element.server.name}. Absolutely not supposed to happen!"
                    )
                    errors += 1
                else:
                    response = agent.list_services()
                    for service_status in response:
                        if service_id == service_status.get("name", ""):
                            if not service_status.get("is_running"):
                                if can_do_actions:
                                    logger.warning(
                                        f"Service {service_id} is not running. Starting..."
                                    )
                                    is_error, error_message = service.start_on(
                                        agent, cmd_context
                                    )
                                    if is_error:
                                        log_error(
                                            f"Failed to start service {service_id}: {error_message}"
                                        )
                                        errors += 1
                                else:
                                    log_error(f"Service {service_id} is not running")
                                    errors += 1
                            break
        return errors

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
            except Exception:
                conn.sendall(
                    f"Exception while handling command: {traceback.format_exc()}".encode()
                )
                conn.sendall("ERROR".encode())
                logger.error(f"Error handling command: {traceback.format_exc()}")

    def execute_command(self, command: str, socket: socket.socket) -> bool:
        parts = shlex.split(command)
        if not parts:
            socket.sendall("Error: No command provided".encode())
            return False
        action = parts[0]
        command_handler = None
        if action == "help":
            if len(parts) == 1:
                commands = []
                for cmd in COMMANDS:
                    commands.append(cmd.NAME)
                socket.sendall(f"help, shutdown, {', '.join(commands)}".encode())
                return True
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
            buffer = ""
            while "\n" not in buffer:
                buffer += socket.recv(1024).decode()
            return buffer.split("\n", 1)[0]

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
    load_dotenv()
    app = ServerApp()
    app.init()
