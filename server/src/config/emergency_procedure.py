from logger import logger
from error.exceptions import GenericConfigException, MissingConfigException
from plugins.emergency_events.library import EMERGENCY_EVENTS


class EmergencyProcedure:
    def __init__(self, event_name: str, config: dict) -> None:
        self.event_name = event_name
        self.config = config
        self.listener_config = self.config.get("listener", {})

        self.listener_type = self.listener_config.get("type")

        if self.listener_type is None:
            raise MissingConfigException("emergency_proc.XY.listener.type")

        self.listener_class = EMERGENCY_EVENTS.get(self.listener_type)

        if self.listener_class is None:
            raise GenericConfigException(f"Unknown listener type: {self.listener_type}")

        self.listener = self.listener_class()

        if len(self.config.get("actions", [])) == 0:
            logger.warning(f"No actions defined for emergency procedure {self.event_name}, is this intended?")

        self.listener.init(self.config.get("actions", []))

        self.listener.inject(self.listener_config)

    def unload(self) -> None:
        self.listener.cancel(self.listener_config)
