import json
import os

from config.load import get_config
from logger import logger
from config.emergency_procedure import EmergencyProcedure
from config.parser import parse_json_file

class EmergencyProceduresConfig:
    def __init__(self) -> None:
        self.event_listeners: dict[str, EmergencyProcedure] = {}
        self.load_config()
        self.apply_config()

    def load_config(self):
        emergency_config = os.getenv("EMERGENCY_CONFIG_FILE", None)
        if emergency_config is None:
            normal_config = get_config()
            emergency_config = os.path.join(
                os.path.dirname(normal_config), "emergency_proc.jsonc"
            )
        self.config_path = emergency_config

        if not os.path.exists(emergency_config):
            logger.warning(f"Config not found ({emergency_config})")
            self.config_raw = {}
        else:
            self.config_raw = parse_json_file(emergency_config)

    def apply_config(self):
        for event_name, listener in self.event_listeners.items():
            listener.unload()
        self.event_listeners.clear()
        for event_name, event_data in self.config_raw.items():
            listener = EmergencyProcedure(event_name, event_data)
            self.event_listeners[event_name] = listener

    def reload(self):
        self.load_config()
        self.apply_config()
