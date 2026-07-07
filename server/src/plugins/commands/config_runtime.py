from command_context import CommandContext
from helpers import get_current_context
from plugins.commands._template import CommandBase


class ConfigRuntimeCommand(CommandBase):
    NAME = "config:runtime"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        if len(arguments) == 0:
            cmd_context.output_print(ConfigRuntimeCommand().get_help())
            return False
        context = get_current_context()
        if arguments[0] == "reload":
            context.runtime_config.reload(cmd_context, no_backup_check="no_backup_check" in arguments[1:])
        elif arguments[0] == "dump":
            context.runtime_config.dump()
        else:
            cmd_context.output_print(f"Unknown command: {arguments[1]}")
            return False
        return True

    @staticmethod
    def get_help() -> str:
        return """
        Helper to work with the runtime.json config. This config can automate things, so instead of running command you have a config.
        Usage: config:runtime reload/dump
        - reload: Reads to config and apply it to the current state of the app. Arguments: no_backup_check -> used for testing, if you want to reload the config without checking if a backup needs to be done
        - dump: Writes the current state of the app into the config (OVERWRITES ANY DATA INSIDE!!!)
        """
