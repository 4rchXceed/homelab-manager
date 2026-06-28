import subprocess

from plugins.emergency_actions._template import EmergencyActionTemplate
from logger import logger


class ShellEmergencyAction(EmergencyActionTemplate):
    @staticmethod
    def call(config: dict):
        command = config.get("command")
        if not command:
            logger.warning("No command provided for shell emergency action")
            return
        logger.info(f"Executed shell command: {command}") # Still debug something, to "warn" the user
        subprocess.run(command, shell=True)
