from command_context import CommandContext
from helpers import get_current_context
from plugins.commands._template import CommandBase


class ConfigSyncCommand(CommandBase):
    NAME = "config:sync"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        context = get_current_context()
        for service in context.app.services.values():
            cmd_context.output_print(f"Starting config sync for {service.name}...")
            if service.db_element.server:
                responses = service.finish_init(cmd_context)
                for response in responses:
                    if response.get("success", False):
                        cmd_context.output_print(
                            f"[{response.get('path', 'Unknown')}]: File generated successfully."
                        )
                    else:
                        cmd_context.output_print(
                            f"[{response.get('path', 'Unknown')}]: File generation failed. Details: {', '.join(response.get('return_codes', []))}"
                        )
            else:
                cmd_context.output_print(
                    f"Skipping config sync for {service.name} (no server assigned)"
                )
        cmd_context.output_print("Config sync completed.")
        context.event_manager.trigger_event("config_synced")
        return True

    @staticmethod
    def get_help() -> str:
        return "Syncs the configuration of all services."
