import datetime
import time
import threading
import json
import os

from command_context import CommandContext
from config.load import get_config
from helpers import get_current_context, parse_time
from logger import logger
from protocol.agent import Agent
from services.service import ServerService
from config.parser import parse_json_file
from error.exceptions import GenericConfigException

class RuntimeConfig:
    def init(self) -> None:
        self.context = get_current_context()
        self.context.event_manager.register_event("config_reloaded", self.reload_backup_assignments)
        self.context.event_manager.register_event("agent_connected", self.reload_syncs_from_db)
        self.load_config()

    def reload_syncs_from_db(self, _):
        for service_id, service in self.context.app.services.items():
            if not service.sync_storage and service.db_element.sync_server and service.db_element.sync_storage_id_str:
                agent_id = service.db_element.sync_server.id_str
                agent = Agent.get_from_id_str(agent_id)
                if agent:
                    storage_id = service.db_element.sync_storage_id_str
                    storage = agent.resolve_storage(storage_id)
                    if storage:
                        if storage.id != storage_id:
                            logger.warning(f"!! Storage {storage_id} invalid. Using fallback: {storage.id}")
                        service.sync_storage = storage
                        success = service.update_sync()
                        if not success:
                            logger.error(f"Failed to update the sync for service: {service_id}")
                    else:
                        logger.error(f"!! Sync storage for service {service_id} not found for agent {agent_id}")
                else:
                    logger.error(f"!! Sync agent {agent_id} for service {service_id} is not found! Cannot sync!!")

    # do_not_check_backups is used (mainly) for tests
    def reload(self, cmd_context: CommandContext, no_backup_check = False):
        self.load_config()
        self.reload_assignments(cmd_context)
        self.reload_syncs(cmd_context)
        if not no_backup_check:
            self.context.app.check_backups()

    def reload_syncs(self, cmd_context: CommandContext):
         # First: check for modifications and additions on syncs
         for service_id, storage_config in self.syncs.items():
            service = ServerService.get_from_id_str(service_id)
            if not storage_config.get("server") or not storage_config.get("storage"):
                cmd_context.output_print(f"Either server or storage config key is missing for sync config: {service_id}")
            if service:
                agent_id  = storage_config.get("server")
                agent = Agent.get_from_id_str(agent_id)
                if agent:
                    storage_id = storage_config.get("storage")
                    storage = agent.resolve_storage(storage_id)
                    if storage:
                        ok = True
                        if storage.id != storage_id:
                            cmd_context.output_print(f"/!\\ Storage {storage_id} was invalid. Fallback is {storage.id}")
                            ok = cmd_context.output_input(f"Use fallback {storage.id} ? (y/N)").lower() == "y"
                        if ok:
                            change = False
                            sync_time = parse_time(storage_config.get("fullSyncInterval", None))
                            if sync_time:
                                if not service.db_element.sync_server or service.db_element.sync_server.id_str != agent_id:
                                    if agent.db_server:  # Should never be None
                                        cmd_context.output_print("> Sync server has changed... updating config")
                                        service.db_element.sync_server = agent.db_server
                                        self.context.database.session.commit()
                                        change = True
                                if service.db_element.sync_storage_id_str != storage_id:
                                    cmd_context.output_print("> Sync storage has changed... updating config")
                                    service.db_element.sync_storage_id_str = storage_id
                                    self.context.database.session.commit()
                                    change = True
                                if service.db_element.sync_time != sync_time:
                                    cmd_context.output_print("> Sync time has changed... updating config (and resetting next sync time. run sync:full to run a full sync now)")
                                    service.db_element.sync_time = sync_time
                                    service.db_element.last_sync = datetime.datetime.now()
                                    self.context.database.session.commit()
                                    change = True
                                if change:
                                    service.sync_storage = storage
                                    success = service.update_sync()
                                    service.full_sync()
                                    if success:
                                        cmd_context.output_print(f"Successfully updated sync config for service {service_id}!")
                                    else:
                                        cmd_context.output_print(f"{service_id} has is probably not assigned to any server / other sync config issue")
                            else:
                                cmd_context.output_print(f"/!\\ Invalid sync time for service {service_id}. Ignoring sync assignment")
                        else:
                            cmd_context.output_print("/!\\ Aborting")
                    else:
                        cmd_context.output_print(f"/!\\ Storage {storage_id} invalid / not found and no fallback storage found")
                else:
                    cmd_context.output_print(f"/!\\ Agent with id {agent_id} not found. Ignoring sync assignment")
            else:
                cmd_context.output_print(f"/!\\ Service {service_id} is not a valid service. Ignoring sync assignment")

    def reload_assignments(self, cmd_context: CommandContext):
        # Check for modifications and additions on assignments
        cmd_context.output_print("Checking for changes in service assignments...")
        for service_id, server_id in self.assignments.items():
            service = ServerService.get_from_id_str(service_id)
            agent = Agent.get_from_id_str(server_id)
            if not service or not agent or not agent.db_server:
                cmd_context.output_print(
                    f"/!\\ Either {service_id} is not a valid service or {server_id} is not a valid agent/server -> ignoring assignment"
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



    def dump(self):
        # Dump the current config to the file !! it overwrites the file entirely
        self.assignments.clear()
        for service_id, service in self.context.app.services.items():
            if service.db_element.server:
                self.assignments[service_id] = service.db_element.server.id_str
        self.config_raw = {"assignments": self.assignments, "backupAssignments": self.backup_assignments}
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
        self.syncs = self.config_raw.get("syncs", {})
        self.reload_backup_assignments()

    def reload_backup_assignments(self, _=None):
        for service_id, service in self.context.app.services.items():
            err_message = f" For 'backup security' purposes all backup configs should have an entry in the runtime config (even if the service is not assigned to a server)"
            if not service_id in self.assignments.keys() and len(service.backup_configs) > 0:
                logger.error(f"Service {service_id} has backup configs, but is not in the runtime config. {err_message}")
            backup_assignment_service_config = self.backup_assignments.get(service_id, {})
            for backup_config in service.backup_configs:
                if not backup_config.id_str in backup_assignment_service_config.keys():
                    logger.error(f"Backup config {backup_config.id_str} for service {service_id} is not in the runtime config. {err_message}")
                backup_assignments: list[dict] = backup_assignment_service_config.get(backup_config.id_str, [])
                for backup_assignment in backup_assignments:
                    if backup_assignment.get("server") is None or backup_assignment.get("storage") is None:
                        logger.error("Backup assignment is missing required fields: 'server' and 'storage'")
                backup_config.targets = backup_assignments
