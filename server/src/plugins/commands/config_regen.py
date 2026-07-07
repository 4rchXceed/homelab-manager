from command_context import CommandContext
from helpers import get_current_context
from plugins.commands._template import CommandBase
from services.service import ServerService


class RegenConfigCommand(CommandBase):
    NAME = "config:regen"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        context = get_current_context()
        if not len(arguments) > 0:
            yes_no = cmd_context.output_input(
                "Are you sure you want to regenerate the configuration for all active services? (y/N) "
            )
            if yes_no.lower() == "y":
                for service_id, service in context.app.services.items():
                    if service.db_element.server is not None:
                        cmd_context.output_print(
                            f"Regenerating configs for {service_id}"
                        )
                        for config_file in service.config_files:
                            config_file.before_regenerate()
                        for config_file in service.config_files:
                            cmd_context.output_print(
                                f"Regenerating config file: {config_file.path}"
                            )
                            config_file.regenerate(cmd_context)
                            context.event_manager.trigger_event("config_synced")
                        cmd_context.output_print(
                            f"Config regeneration complete for {service_id}"
                        )
                cmd_context.output_print("All config regenerated")
                return True
            else:
                return False

        service_id = arguments[0]
        service = ServerService.get_from_id_str(service_id=service_id)
        if not service:
            cmd_context.output_print(f"Service with id {service_id} not found")
            return False
        for config_file in service.config_files:
            config_file.before_regenerate()
        for config_file in service.config_files:
            cmd_context.output_print(f"Regenerating config file: {config_file.path}")
            config_file.regenerate(cmd_context)
            context.event_manager.trigger_event("config_synced")
        cmd_context.output_print("Config regeneration complete")
        return True

    @staticmethod
    def get_help() -> str:
        return """
        Regenerates a config. Useful when you changed something, and want it to be applied, but the auto-reload doesn't work.
        Usage: config:regen <service_id>
        If you don't pass any service_id, it will ask to regenerate ALL active services
        """
