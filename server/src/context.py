from queue import Queue
from typing import Literal

from config.general import GeneralConfig
from config_gen.generators import Generators
from database.database import DatabaseEngine
from plugins.variable_providers._template import VariableProvider


class HLMContext:
    def __init__(
        self,
        database: DatabaseEngine,
        generators: Generators,
        providers: dict[str, type[VariableProvider]],
        agent_command_queue: Queue,
        config_general: GeneralConfig,
    ) -> None:
        self.database = database
        self.generators = generators
        self.providers = providers
        self.message_queue = agent_command_queue
        self.env: Literal["webui", "cli"] = "cli"
        self.config_general = config_general
