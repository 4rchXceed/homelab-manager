import subprocess

from plugins.emergency_actions._template import EmergencyActionTemplate
from logger import logger


class LoggerEmergencyAction(EmergencyActionTemplate):
    @staticmethod
    def call(config: dict):
        level = config.get("level", "warning")
        message = config.get("message", None)
        if level == "warning":
            logger.warning(message)
        elif level == "info":
            logger.info(message)
        elif level == "debug":
            logger.debug(message)
        elif level == "error":
            logger.error(message)
        else:
            logger.warning(f"Unknown log level: {level}")
