from command_context import CommandContext
from helpers import get_current_context
from plugins.commands._template import CommandBase


class EmergencyProcedureCommand(CommandBase):
    NAME = "config:emergency_proc"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        if len(arguments) == 0:
            cmd_context.output_print(EmergencyProcedureCommand().get_help())
            return False
        context = get_current_context()
        if arguments[0] == "reload":
            if context.app.emergency_procedure_config:
                context.app.emergency_procedure_config.reload()
                cmd_context.output_print("Emergency procedure config reloaded")
            else:
                cmd_context.output_print("Emergency procedure config not loaded")
        else:
            cmd_context.output_print(f"Unknown command: {arguments[1]}")
            return False
        return True

    @staticmethod
    def get_help() -> str:
        return """
        Helper to work with the emergency procedure config.
        Supports the following subcommands:
            - reload: Reloads the emergency procedure config from the emergency_proc.json file
        """
