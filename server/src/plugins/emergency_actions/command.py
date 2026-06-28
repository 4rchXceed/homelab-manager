
import subprocess

from plugins.emergency_actions._template import EmergencyActionTemplate
from logger import logger
from helpers import get_current_context
from plugins.commands.library import COMMANDS
from command_context import CommandContext

class CommandEmergencyAction(EmergencyActionTemplate):
    @staticmethod
    def call(config: dict):
        command = config.get("command")
        arguments = config.get("arguments", [])
        inputs = config.get("inputs", [])

        if not command:
            logger.warning("No command provided for command emergency action")
            return

        context = get_current_context()
        command_handler = None
        for cmd in COMMANDS:
            if cmd.NAME == command:
                command_handler = cmd
                break
        if not command_handler:
            logger.warning(f"Command '{command}' not found")
            return
        def custom_print(msg: str):
            logger.info(msg)
        input_i = 0
        def custom_input(msg: str):
            nonlocal input_i
            input = inputs[input_i]
            input_i += 1
            # Prevent input_i from going out of bounds, always return the last one
            if input_i == len(inputs):
                input_i -= 1
            return input

        command_context = CommandContext()
        command_context.output_input = custom_input
        command_context.output_print = custom_print

        command_handler.execute(arguments, command_context)
