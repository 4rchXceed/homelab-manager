import datetime
from command_context import CommandContext
from plugins.commands._template import CommandBase
from services.service import ServerService
from helpers import get_current_context
from services.backup_config import BackupConfig

class BackupServiceCommand(CommandBase):
    NAME = "backup:create"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        if len(arguments) < 2:
            cmd_context.output_print(BackupServiceCommand.get_help())
            return False
        service = ServerService.get_from_id_str(arguments[0])
        if service is None:
            cmd_context.output_print(f"Service with ID {arguments[0]} not found.")
            return False
        backup_id = arguments[1]
        reset_counter = "reset_counter" in arguments[2:]
        i = 0
        backup_config = None
        while backup_config is None and i < len(service.backup_configs):
            if service.backup_configs[i].id_str == backup_id:
                backup_config = service.backup_configs[i]
            i += 1
        if backup_config is None:
            cmd_context.output_print(f"Backup with ID {backup_id} not found for service {service.id}.")
            return False
        cmd_context.output_print(f"Starting backup for service {service.id} with backup ID {backup_id}.")
        cmd_context.output_print(f"Backup configuration: {backup_config}")
        service.backup(backup_config, nothread=True)
        cmd_context.output_print(f"Backup for service {service.id} with backup ID {backup_id} completed.")
        if reset_counter:
            db_element = get_current_context().database.session.query(BackupConfig).filter_by(id=backup_config.db_element_id).first()
            if db_element:
                db_element.last = datetime.datetime.now()
        return True

    @staticmethod
    def get_help() -> str:
        return """
        Starts a backup of the service.
        Usage: backup_create <service_id> <backup_id> [reset_counter]
        - service_id: The ID of the service to backup.
        - backup_id: The ID of the backup for the service.
        - reset_counter: Optional. If provided, resets the counter for the next auto backup. If not provided, the counter will not be reset.
        """
