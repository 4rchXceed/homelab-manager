import datetime
from command_context import CommandContext
from plugins.commands._template import CommandBase
from services.service import ServerService
from helpers import get_current_context, format_size
from services.backup_config import BackupConfig
from protocol.agent import Agent

class BackupListCommand(CommandBase):
    NAME = "backup:list"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        if len(arguments) < 2:
            cmd_context.output_print(BackupListCommand.get_help())
            return False

        agent_id = arguments[0]
        storage_id = arguments[1]

        agent = Agent.get_from_id_str(agent_id)
        if not agent:
            cmd_context.output_print(f"Error: Agent with ID \"{agent_id}\" not found.")
            return False

        storage = agent.resolve_storage(storage_id)

        if not storage:
            cmd_context.output_print(f"Error: Storage with ID \"{storage_id}\" not found in agent \"{agent_id}\".")
            return False

        if storage.id != storage_id:
            cmd_context.output_print("WARNING [ACTION REQUIRED]: THE STORAGE IS INVALID OR DOESN'T EXIST. THE BACKUP SYSTEM IS CURRENTLY USING A FALLBACK STORAGE. ANY BACKUPS MADE WILL BE STORED IN THE FALLBACK STORAGE.")
        with_size = "with_size" in arguments[2:]

        r = agent.send_pingpong({
            "type": "list_available_backups",
            "path": storage.path,
            "size": with_size
        })
        backups = r.get("backups", {})

        cmd_context.output_print(f"Available backups for storage \"{storage_id}\" in agent \"{agent_id}\":\n")
        for service_id, backup_list in backups.items():
            cmd_context.output_print(f"Service ID: {service_id}\n")
            for type in ["full", "incremental"]:
                if type in backup_list:
                    cmd_context.output_print(f"  {type.capitalize()} Backups:\n")
                    total_size = 0
                    for backup in backup_list[type]:
                        if not backup.get("total_size"):
                            backup_name = backup["folder"]
                            if with_size:
                                size = backup.get("size", None)
                                size_str = f"Size: {format_size(size)}" if size is not None else "Size: Unknown"
                            else:
                                size_str = ""
                            if backup_name == "base":
                                cmd_context.output_print(f"    - Base Backup: {backup_name}. {size_str}\n")
                            else:
                                time = datetime.datetime.fromtimestamp(int(backup_name)).strftime('%Y-%m-%d %H:%M:%S')
                                cmd_context.output_print(f"    - Backup ID: {backup_name}, Time: {time}. {size_str}\n")
                        else:
                            total_size = backup.get("total_size", 0)
                    if with_size:
                        cmd_context.output_print(f"  Total backups size: {format_size(total_size)}\n")
                else:
                    cmd_context.output_print(f"  No {type} backups found.\n")

        return True

    @staticmethod
    def get_help() -> str:
        return """
        Lists all backups of a specified storage in a server. Human-readable format.
        Usage: backup:list <server_id> <storage_id> [with_size]
        - server_id: The ID of the server to list backups for.
        - storage_id: The ID of the storage to list backups for.
        - with_size: Optional flag to include the size of each backup in the output. Don't use it if you have a lot of backups, as it will take a long time to calculate the size of each backup.
        """
