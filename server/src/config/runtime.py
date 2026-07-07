import threading
import json
import os

from command_context import CommandContext
from config.load import get_config
from helpers import get_current_context
from logger import logger
from protocol.agent import Agent
from services.service import ServerService
from config.parser import parse_json_file
from error.exceptions import GenericConfigException

class RuntimeConfig:
    def init(self) -> None:
        self.context = get_current_context()
        self.context.event_manager.register_event("config_reloaded", self.reload_backup_assignments)
        self.load_config()

    # do_not_check_backups is used (mainly) for tests
    def reload(self, cmd_context: CommandContext, no_backup_check = False):
        self.load_config()
        # Check for modifications and additions on assignments
        cmd_context.output_print("Checking for changes in service assignments...")
        for service_id, server_id in self.assignments.items():
            service = ServerService.get_from_id_str(service_id)
            agent = Agent.get_from_id_str(server_id)
            if not service or not agent or not agent.db_server:
                cmd_context.output_print(
                    f"Either {service_id} is not a valid service or {server_id} is not a valid agent/server -> ignoring assignment"
                )
            else:
                if not service.db_element.server:
                    cmd_context.output_print(f"Starting {service_id} on {server_id}...")
                    service.start_on(agent, cmd_context)
                elif service.db_element.server.id != agent.db_server.id:
                    cmd_context.output_print(
                        f"Stopping {service_id} on {service.db_element.server.id_str}..."
                    )
                    service.unassign(cmd_context)
                    cmd_context.output_print(f"Starting {service_id} on {server_id}...")
                    service.start_on(agent, cmd_context)
                else:
                    cmd_context.output_print(
                        f"{service_id} is already on {server_id} -> no action required"
                    )
        for service_id, service in self.context.app.services.items():
            if service_id not in self.assignments.keys():
                if service.db_element.server:
                    cmd_context.output_print(
                        f"Stopping {service_id} on {service.db_element.server.id_str}..."
                    )
                    service.unassign(cmd_context)
        cmd_context.output_print("Checking for changes in backup assignments...")
        if not no_backup_check:
            self.context.app.check_backups()

    def dump(self):
        # TODO: Finish this
        # Dump the current config to the file !! it overwrites the file entirely
        self.assignments.clear()
        for service_id, service in self.context.app.services.items():
            if service.db_element.server:
                self.assignments[service_id] = service.db_element.server.id_str
        self.config_raw = {"assignments": self.assignments}
        with open(self.config_path, "w", encoding="utf-8") as f_dst:
            json.dump(self.config_raw, f_dst, indent=4)

    def load_config(self):
        runtime_config = os.getenv("RUNTIME_CONFIG_FILE", None)
        if runtime_config is None:
            normal_config = get_config()
            runtime_config = os.path.join(
                os.path.dirname(normal_config), "runtime.jsonc"
            )
        self.config_path = runtime_config

        if not os.path.exists(runtime_config):
            logger.warning(f"Auto generating runtime config at: {runtime_config}")
            with open(runtime_config, "w", encoding="utf-8") as f_dst:
                f_dst.write("{}")

        self.config_raw = parse_json_file(runtime_config)
        self.assignments: dict[str, str] = self.config_raw.get("assignments", {})
        self.backup_assignments: dict = self.config_raw.get("backupAssignments", {})
        self.reload_backup_assignments()

    def reload_backup_assignments(self, _=None):
        for service_id, service in self.context.app.services.items():
            err_message = f" For 'backup security' purposes all backup configs must have an entry in the runtime config (even if the service is not assigned to a server)"
            if not service_id in self.assignments.keys() and len(service.backup_configs) > 0:
                raise GenericConfigException(f"Service {service_id} has backup configs, but is not in the runtime config. {err_message}")
            backup_assignment_service_config = self.backup_assignments.get(service_id, {})
            for backup_config in service.backup_configs:
                if not backup_config.id_str in backup_assignment_service_config.keys():
                    raise GenericConfigException(f"Backup config {backup_config.id_str} for service {service_id} is not in the runtime config. {err_message}")
                backup_assignments: list[dict] = backup_assignment_service_config.get(backup_config.id_str, [])
                for backup_assignment in backup_assignments:
                    if backup_assignment.get("server") is None or backup_assignment.get("storage") is None:
                        raise GenericConfigException("Backup assignment is missing required fields: 'server' and 'storage'")
                backup_config.targets = backup_assignments
