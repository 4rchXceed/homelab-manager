from queue import Queue
from typing import TYPE_CHECKING, Literal

if TYPE_CHECKING:
    from config.runtime import RuntimeConfig
    from main import ServerApp
from config.general import GeneralConfig
from config.servers import ConfigServers
from config_gen.generators import Generators
from database.database import DatabaseEngine
from database.models import Service
from events import EventManager
from logger import logger
from plugins.variable_providers._template import VariableProvider


class HLMContext:
    def __init__(
        self,
        database: DatabaseEngine,
        generators: Generators,
        providers: dict[str, type[VariableProvider]],
        message_queue: Queue,
        config_general: GeneralConfig,
        config_servers: ConfigServers,
        runtime_config: "RuntimeConfig",
        app: "ServerApp",
    ) -> None:
        self.database = database
        self.generators = generators
        self.providers = providers
        self.message_queue = message_queue
        self.env: Literal["webui", "cli"] = "cli"
        self.config_general = config_general
        self.config_servers = config_servers
        self.kill_switch = False
        self.app = app
        self.agents = self.app.agents
        self.event_manager = EventManager()
        self.runtime_config = runtime_config

    def send_from_service(
        self, service_id: str, message: dict, timeout=10
    ) -> dict | None:
        service = (
            self.database.session.query(Service)
            .filter_by(disabled=False)
            .filter_by(id_str=service_id)
            .first()
        )
        if service:
            if service.server:
                for a in self.agents:
                    if a.id == service.server.id_str:
                        return a.send_pingpong(message, timeout=timeout)
                logger.warning(
                    f"Agent {service.server.id_str} connected, but tried to send message"
                )
            else:
                logger.warning(
                    f"Service {service_id} has no server associated, but tried to send message"
                )
            return
        logger.warning(f"Service {service_id} not found, cannot send message")
        return
