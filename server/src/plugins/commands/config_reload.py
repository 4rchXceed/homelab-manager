from command_context import CommandContext
from helpers import get_current_context
from plugins.commands._template import CommandBase


class ConfigReloadCommand(CommandBase):
    NAME = "config:reload"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        context = get_current_context()
        sync_now = "sync_now" in arguments
        if sync_now:
            cmd_context.output_print("Reloading and syncing config...")
        else:
            cmd_context.output_print("Reloading config...")
        cmd_context.output_print(context.app.reload_config(sync_now, cmd_context))
        return True

    @staticmethod
    def get_help() -> str:
        return (
            "config:reload [sync_now] -> reload the config from the .json file. If you modify the general config (key: config), you will need to restart the whole server"
            "sync_now (optional flag): Sync/Apply the changes now"
        )
