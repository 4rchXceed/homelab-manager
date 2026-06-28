import subprocess

from plugins.emergency_actions._template import EmergencyActionTemplate
from logger import logger
from error.report import report_error


class NotificationEmergencyAction(EmergencyActionTemplate):
    @staticmethod
    def call(config: dict):
        message = config.get("message", None)
        title = config.get("title", None)
        level = config.get("level", 0)
        if not message:
            logger.warning("No message provided for NotificationEmergencyAction")
            return
        if not title:
            logger.warning("No title provided for NotificationEmergencyAction")
            title = "No title"
        report_error(title, message, level)
