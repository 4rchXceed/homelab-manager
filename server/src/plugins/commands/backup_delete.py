from turtle import back
from command_context import CommandContext
from helpers import get_current_context, format_size
from plugins.commands._template import CommandBase
from services.service import ServerService
from protocol.agent import Agent

class BackupDeleteCommand(CommandBase):
    NAME = "backup:delete"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        if len(arguments) < 4:
            cmd_context.output_print(BackupDeleteCommand.get_help())
            return False
        service_id = arguments[0]
        backup_server_id = arguments[1]
        backup_storage_id = arguments[2]
        backup_type = arguments[3]
        service = ServerService.get_from_id_str(service_id)
        if service is None:
            cmd_context.output_print(f"ERROR: Service with ID {service_id} not found.")
            return False
        agent = Agent.get_from_id_str(backup_server_id)
        if agent is None:
            cmd_context.output_print(f"ERROR: Backup server with ID {backup_server_id} not found.")
            return False
        storage = agent.resolve_storage(backup_storage_id)
        if storage is None:
            cmd_context.output_print(f"ERROR: Backup storage with ID {backup_storage_id} not found on server {backup_server_id}.")
            return False
        if storage.id != backup_storage_id:
            cmd_context.output_print(f"ERROR: The backup storage {backup_storage_id} is invalid for the backup server {backup_server_id}. A fallback storage has been selected: {storage.id}. Please re-type the command with the correct storage ID.")
            return False
        cmd_context.output_print("Evaluating backup size for confirmation...")
        r = agent.send_pingpong({
            "type": "list_available_backups",
            "path": storage.path,
            "size": True
        })
        backups = r.get("backups", {})
        if not service_id in backups.keys():
            cmd_context.output_print(f"ERROR: No backups found for service {service_id} on storage {backup_storage_id}.")
            return False
        if not backup_type in backups[service_id].keys():
            cmd_context.output_print(f"ERROR: No backups of type {backup_type} found for service {service_id} on storage {backup_storage_id}.")
            return False
        total = 0
        for backup in backups[service_id][backup_type]:
            if backup.get("total_size"):
                total = backup["total_size"]
            else:
                kw = "Backup"
                if backup_type == "sync":
                    total = backup["size"]
                    kw = "Sync"
                cmd_context.output_print(f"{kw}: {backup['folder']} - Size: {format_size(backup['size'])}")
        cmd_context.output_print(f"Total size of backups to delete: {format_size(total)}")
        confirmation = cmd_context.output_input(f"Are you sure you want to delete ALL backups of type {backup_type} for service {service_id} on storage {backup_storage_id}? (yes/no): ")
        if confirmation.lower() != "yes":
            cmd_context.output_print("Aborting deletion of backups.")
            return False
        confirmation_2 = cmd_context.output_input(f"This will delete {format_size(total)} of datas. THIS CANNOT BE REVERSED! Please type the exact id of the service ({service.id}) to confirm: ")
        if confirmation_2 != service.id:
            cmd_context.output_print("Service name does not match. Aborting deletion of backups.")
            return False
        total = format_size(total)
        cmd_context.output_print("Btw: you have an option: maxSize in the backup config. Please set this instead of deleting all backups. This is a better way to manage your backups.")
        confirmation_3 = cmd_context.output_input(f"Are you REALLY sure? Please type the total size of the backups to delete ({total}) to confirm: ")
        if confirmation_3 != total:
            cmd_context.output_print("Total size does not match. Aborting deletion of backups.")
            return False
        cmd_context.output_print(f"Deleting ALL backups of type {backup_type} for service {service_id} on storage {backup_storage_id}...")
        r = agent.send_pingpong({
            "type": "delete_backups",
            "path": storage.path,
            "service_id": service_id,
            "backup_type": backup_type
        })
        if not r.get("success", False):
            cmd_context.output_print(f"ERROR: Failed to delete backups. Please check the backup server logs for more information.")
            return False
        cmd_context.output_print(f"Successfully deleted ALL backups of type {backup_type} for service {service_id} on storage {backup_storage_id}.")
        cmd_context.output_print(f"If you deleted this backup by accident, you're doomed XD")
        return True

    @staticmethod
    def get_help() -> str:
        return """
        Deletes ALL backups of one type for one service for one storage.
        !! THIS WILL DELETE EVERY SINGLE BACKUP OF THE SPECIFIED TYPE FOR THE SPECIFIED SERVICE !!
        !! THERE WILL BE A LOT OF CONFIRMATION PROMPTS, SO BE SURE YOU WANT TO DO THIS !!
        !! THIS IS A DANGEROUS COMMAND, USE WITH CAUTION !!
        !! REMINDER: THIS IS A SERVER MANAGER !!
        Usage: backup:delete <service> <backup_server_id> <backup_storage_id> <backup_type>
        """
