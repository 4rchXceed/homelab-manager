import time

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
        all_requests = []

        def handle_request_recv(data):
            if data["r_uuid"] in all_requests:
                all_requests.remove(data["r_uuid"])

        context.event_manager.register_event("request_recv", handle_request_recv)

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
                all_requests.extend(service.start_on(agent))
                return True
        if not all_requests:
            cmd_context.output_print(f"Service {service_needed} not found")

        while len(all_requests) > 0:
            time.sleep(0.1)
        context.event_manager.unregister_event("request_recv", handle_request_recv)
        return False
