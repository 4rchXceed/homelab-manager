from command_context import CommandContext
from helpers import get_current_context
from plugins.commands._template import CommandBase


class ServiceListCommand(CommandBase):
    NAME = "services:list"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        context = get_current_context()

        for service_id, service in context.app.services.items():
            cmd_context.output_print(
                f"[{service_id}]: {service.name} -> On server {service.db_element.server.id_str + ' (' + service.db_element.server.name + ')' if service.db_element.server else 'none'}\n"
            )
        return True

    @staticmethod
    def get_help() -> str:
        return "List all services, with the server they're currently on (no arguments needed)"
