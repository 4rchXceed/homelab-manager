from plugins.emergency_actions.command import CommandEmergencyAction
from plugins.emergency_actions.logger_action import LoggerEmergencyAction
from plugins.emergency_actions.notification import NotificationEmergencyAction
from plugins.emergency_actions.shell import ShellEmergencyAction
from plugins.emergency_actions._template import EmergencyActionTemplate


EMERGENCY_ACTIONS: dict[str, type[EmergencyActionTemplate]] = {
    "shell": ShellEmergencyAction,
    "logger": LoggerEmergencyAction,
    "command": CommandEmergencyAction,
    "notification": NotificationEmergencyAction
}
