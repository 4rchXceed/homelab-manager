from command_context import CommandContext
from helpers import get_current_context
from plugins.commands._template import CommandBase
from services.service import ServerService


class SyncFullCommand(CommandBase):
    NAME = "sync:full"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        if len(arguments) < 1:
            cmd_context.output_print(SyncFullCommand.get_help())
            return False
        service = ServerService.get_from_id_str(arguments[0])
        if service is None:
            cmd_context.output_print(f"Service with id {arguments[0]} not found.")
            return False
        cmd_context.output_print(f"Syncing service {service.id}... (this may take a while, depending on how much data the service has)")
        service.full_sync()
        cmd_context.output_print(f"Service {service.id} synced successfully.")
        return True

    @staticmethod
    def get_help() -> str:
        return """
        Issues a full sync of this service. Useful if you want to be sure that all data is up to date from the service to the sync.
        Usage: sync:full <service_id>
        """
