from command_context import CommandContext
from helpers import get_current_context
from plugins.commands._template import CommandBase


class ServiceUnassignCommand(CommandBase):
    NAME = "service:unassign"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        if len(arguments) != 1:
            cmd_context.output_print("Usage: service:unassign <service_name>")
            return False

        service_name = arguments[0]
        context = get_current_context()
        if service_name not in context.app.services.keys():
            cmd_context.output_print(f"Service '{service_name}' not found")
            return False
        service = context.app.services[service_name]
        service.unassign(cmd_context)
        cmd_context.output_print(f"Service '{service_name}' unassigned")
        return True

    @staticmethod
    def get_help() -> str:
        return "Unassign a service from the agent. Usage: service:unassign <service_name>. Please notice that the service will be stopped from the agent."
