import datetime
from command_context import CommandContext
from plugins.commands._template import CommandBase
from services.service import ServerService
from helpers import get_current_context
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

        r = agent.send_pingpong({
            "type": "list_available_backups",
            "path": storage.path
        })
        backups = r.get("backups", {})

        cmd_context.output_print(f"Available backups for storage \"{storage_id}\" in agent \"{agent_id}\":\n")
        for service_id, backup_list in backups.items():
            cmd_context.output_print(f"Service ID: {service_id}\n")
            for type in ["full", "incremental"]:
                if type in backup_list:
                    cmd_context.output_print(f"  {type.capitalize()} Backups:\n")
                    for backup in backup_list[type]:
                        if backup == "base":
                            cmd_context.output_print(f"    - Base Backup: {backup}\n")
                        else:
                            time = datetime.datetime.fromtimestamp(int(backup)).strftime('%Y-%m-%d %H:%M:%S')
                            cmd_context.output_print(f"    - Backup ID: {backup}, Time: {time}\n")
                else:
                    cmd_context.output_print(f"  No {type} backups found.\n")

        return True

    @staticmethod
    def get_help() -> str:
        return """
        Lists all backups of a specified storage in a server. Human-readable format.
        Usage: backup:list <server_id> <storage_id>
        - server_id: The ID of the server to list backups for.
        - storage_id: The ID of the storage to list backups for.
        """
