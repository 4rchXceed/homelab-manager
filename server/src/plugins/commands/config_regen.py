from command_context import CommandContext
from helpers import get_current_context
from plugins.commands._template import CommandBase
from services.service import ServerService


class RegenConfigCommand(CommandBase):
    NAME = "config:regen"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        if not len(arguments) > 0:
            cmd_context.output_print("Usage: config:regen <service_id>")
            return False
        service_id = arguments[0]
        context = get_current_context()
        service = ServerService.get_from_id_str(service_id=service_id)
        if not service:
            cmd_context.output_print(f"Service with id {service_id} not found")
            return False
        for config_file in service.config_files:
            cmd_context.output_print(f"Regenerating config file: {config_file.path}")
            config_file.regenerate(cmd_context)
            context.event_manager.trigger_event("config_synced")
        cmd_context.output_print("Config regeneration complete")
        return True
