from command_context import CommandContext
from helpers import get_current_context
from plugins.commands._template import CommandBase


class ServicesSync(CommandBase):
    NAME = "services:sync"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        can_do_actions = "allow_actions" in arguments
        debug = "debug" in arguments
        context = get_current_context()
        errors = context.app.sync_services(
            can_do_actions=can_do_actions,
            custom_logger=cmd_context.output_print,
            debug=debug,
            cmd_context=cmd_context,
        )
        if errors > 0:
            cmd_context.output_print(
                f"Sync services failed with {errors} errors (see output)"
            )
        else:
            cmd_context.output_print("Sync services completed successfully")
        return True

    @staticmethod
    def get_help() -> str:
        return (
            "Sync all services that are in the database and the irl/real-world services that are running. Can do most of the job, but certain actions can't be done automatically. Flags:\n"
            + "allow_actions: can start/stop services when we are pretty sure that it's what the user wants. Do not set this flag on critical infrastructure. If not set, the only actions that will be performed are those who affect the database, and not the actual services\n"
            + "debug: enable debug mode, which will print more verbose output (progress for example)"
        )
