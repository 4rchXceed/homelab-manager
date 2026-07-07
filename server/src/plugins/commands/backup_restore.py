import datetime
from command_context import CommandContext
from plugins.commands._template import CommandBase
from services.service import ServerService
from helpers import get_current_context
from services.backup_config import BackupConfig
from protocol.agent import Agent

class BackupRestoreCommand(CommandBase):
    NAME = "backup:restore"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        if len(arguments) < 4:
            cmd_context.output_print(BackupRestoreCommand.get_help())
            return False
        service = ServerService.get_from_id_str(arguments[0])
        if service is None:
            cmd_context.output_print(f"Service with ID {arguments[0]} not found.")
            return False
        backup_config_id = arguments[1]
        server_id = arguments[2]
        storage_id = arguments[3]
        backup_id = arguments[4] if len(arguments) > 4 else None
        agent = Agent.get_from_id_str(server_id)
        if agent is None:
            cmd_context.output_print(f"Agent with ID {server_id} not found.")
            return False
        storage = agent.resolve_storage(storage_id)
        if storage is None:
            cmd_context.output_print(f"Storage with ID {storage_id} not found on agent {server_id}.")
            return False
        if storage.id != storage_id:
            cmd_context.output_print(f"Storage with ID: {storage_id} is invalid or doesn't exists. Fallback: {storage.id} has been chosen instead.")
            if cmd_context.output_input(f"Do you want to use the fallback storage: {storage.id} with path: {storage.path}? (y/N)").lower() != "y":
                cmd_context.output_print("Aborting restore operation.")
                return False
        backup_config = None
        i = 0
        while i < len(service.backup_configs) and backup_config is None:
            if service.backup_configs[i].id_str == backup_config_id:
                backup_config = service.backup_configs[i]
            i += 1
        if not backup_config:
            cmd_context.output_print(f"Backup configuration with ID {backup_config_id} not found for service {service.id}.")
            return False
        cmd_context.output_print(f"Restoring backup {backup_id} for service {service.id} using backup configuration {backup_config.id_str} from storage {storage.id}.")
        success = get_current_context().app.backup_manager.issue_restore(backup_config, storage, cmd_context, backup_id=backup_id)
        cmd_context.output_print(f"Restore operation {'succeeded' if success else 'failed'}.")
        return success

    @staticmethod
    def get_help() -> str:
        return """
        Usage: backup:restore <service_id> <backup_config_id> <backup_server_id> <storage_id> [backup_id]
        - service_id: The ID of the service to restore the backup for.
        - backup_config_id: The ID of the backup configuration to use.
        - backup_server_id: The ID of the backup server to restore the backup from.
        - storage_id: The ID of the storage where the backup is stored.
        - backup_id: (Optional) The ID of the specific backup to restore. If not provided, the latest backup will be restored.
        You cannot just specify a backup_id, since a backup_id can have multiple targets (server & storages)
        """
