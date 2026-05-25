import time

from command_context import CommandContext
from helpers import get_current_context
from plugins.commands._template import CommandBase


class ConfigSyncCommand(CommandBase):
    NAME = "config:sync"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        context = get_current_context()
        all_requests = []

        def handle_request_recv(data):
            if data["r_uuid"] in all_requests:
                all_requests.remove(data["r_uuid"])

        context.event_manager.register_event("request_recv", handle_request_recv)

        for service in context.app.services.values():
            cmd_context.output_print(f"Starting config sync for {service.name}...")
            if service.db_element.server:
                all_requests.extend(service.finish_init(cmd_context))
            else:
                cmd_context.output_print(
                    f"Skipping config sync for {service.name} (no server assigned)"
                )
        cmd_context.output_print("Waiting for config sync to complete...")
        while len(all_requests) > 0:
            time.sleep(0.1)
        context.event_manager.unregister_event("request_recv", handle_request_recv)
        cmd_context.output_print("Config sync completed.")
        return True

    @staticmethod
    def get_help() -> str:
        return "Syncs the configuration of all services."
