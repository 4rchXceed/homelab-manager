from command_context import CommandContext
from helpers import get_current_context
from plugins.commands._template import CommandBase


class ServiceAssignCommand(CommandBase):
    NAME = "service:assign"

    @staticmethod
    def execute(arguments, cmd_context: CommandContext) -> bool:
        if not arguments:
            cmd_context.output_print("No arguments provided")
            return False
        service_needed = arguments[0] if len(arguments) > 0 else None
        if not service_needed:
            cmd_context.output_print("No service ID provided")
            return False
        server_to_assign = arguments[1] if len(arguments) > 1 else None
        if not server_to_assign:
            cmd_context.output_print("No server ID provided")
            return False
        context = get_current_context()
        agent = None
        for agent in context.agents:
            if agent.id == server_to_assign:
                agent = agent
                break

        if not agent:
            cmd_context.output_print(f"Agent {server_to_assign} not found")
            return False

        for service_id, service in context.app.services.items():
            if service_id == service_needed:
                cmd_context.output_print(
                    f"Service {service_id} found, assigning to server {server_to_assign}"
                )
                service.start_on(agent, cmd_context, do_not_sync_restore="do_not_sync" in arguments[2:] if len(arguments) > 2 else False)
                return True

        cmd_context.output_print(f"Service {service_needed} not found")
        return False

    @staticmethod
    def get_help() -> str:
        return f"Assign and start a service in a specific server. Usage: {ServiceAssignCommand.NAME} <service_id> <server_id> [do_not_sync]\ndo_not_sync: optional, if provided, the service will not be synced to his sync storage"
