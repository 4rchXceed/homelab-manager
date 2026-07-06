import threading
import json
from collections import Counter

from command_context import CommandContext
from database.models import Service
from error.exceptions import MissingConfigException, ProgramStateError
from helpers import get_current_context
from logger import logger
from protocol.agent import Agent
from services.config_file import ConfigFile
from services.backup_config import ServiceBackupConfig
from protocol.storage import ServerStorage

class ServerService:
    def __init__(self, id: str, config: dict) -> None:
        self.id = id
        self.reload(config)

    def reload(self, config: dict):
        self.context = get_current_context()

        self.config = config

        name = self.config.get("name")
        if name is None:
            raise MissingConfigException("services.$.name")
        self.name = name
        self.datas = self.config.get("data", [])
        self.ignored_folders = self.config.get("ignoredRebuildFolders", [])
        self.config_files_obj = self.config.get("configFiles", [])
        self.config_files = [ConfigFile(data, self) for data in self.config_files_obj]
        self.need_update = False

        # Database
        db_element = (
            self.context.database.session.query(Service)
            .filter_by(id_str=self.id)
            .first()
        )
        if db_element is None:
            db_element = Service(
                id_str=self.id, name=self.name, last_config="{}", disabled=False
            )
            self.context.database.session.add(db_element)
            self.context.database.session.commit()
            self.need_update = True
        else:
            if db_element.disabled:
                self.need_update = True
                db_element.disabled = False
                self.context.database.session.commit()
            db_element = db_element
        self.db_element_id = db_element.id
        # Backups
        self.backup_configs = []
        for backup_config in self.config.get("backups", []):
            self.backup_configs.append(ServiceBackupConfig(backup_config, self))

    def check_backups(self) -> None:
        for backup_config in self.backup_configs:
            if backup_config.needs_backup():
                for backup_target in backup_config.targets:
                    agent = Agent.get_from_id_str(backup_target.get("server"))
                    if agent:
                        storage = agent.resolve_storage(backup_target.get("storage"))
                        if storage:
                            logger.info(
                                f"Running backup for service {self.name} on server {agent.name}"
                            )
                            th = threading.Thread(target=self.run_backup, args=(backup_config, storage))
                            th.start()
                        else:
                            logger.critical(
                                f"Backup target storage {backup_target.get('storage')} not found for service {self.name} on server {agent.name}!!!!"
                            )
                    else:
                        logger.critical(
                            f"Backup target server {backup_target.get('server')} not found for service {self.name}!!!!"
                        )

    def run_backup(self, backup_config: ServiceBackupConfig, backup_storage: ServerStorage) -> bool:
        return self.context.app.backup_manager.issue_backup(backup_config, backup_storage)


    # This is a shared resource between threads, and since every thread has it's own db session, this will f- up everything
    @property
    def db_element(self) -> Service:
        db_element = (
            self.context.database.session.query(Service)
            .filter_by(id=self.db_element_id)
            .first()
        )
        if not db_element:
            raise ProgramStateError(f"Database element for service {self.id} not found")
        return db_element

    def finish_init(self, cmd_context: CommandContext | None = None) -> list[dict]:
        """To avoid the position of the services in the config to matter"""
        responses = []
        if self.db_element.last_config != json.dumps(self.config) or self.need_update:
            if self.db_element.last_config is None:
                responses.extend(self.update({}, cmd_context))
            else:
                responses.extend(
                    self.update(json.loads(self.db_element.last_config), cmd_context)
                )
            self.db_element.last_config = json.dumps(self.config)
            self.context.database.session.commit()
            self.need_update = False
        return responses

    def update(
        self, old_config: dict, cmd_context: CommandContext | None = None
    ) -> list[dict]:
        responses = []
        if old_config.get("name") != self.name:
            self.db_element.name = self.name
            self.context.database.session.commit()
        if Counter(old_config.get("data", [])) != Counter(self.datas):
            # TODO: Add correct handling when self.data will be implemented
            self.db_element.last_config = json.dumps(self.config)
            self.context.database.session.commit()
        if self.db_element.server and (
            self.need_update
            or json.dumps(old_config.get("configFiles", []))
            != json.dumps(self.config_files_obj)
        ):
            for config_file in self.config_files:
                response = config_file.regenerate(cmd_context)
                if response:
                    responses.extend(response)

            self.db_element.last_config = json.dumps(self.config)
            self.context.database.session.commit()

        return responses

    def start_on(
        self, agent: Agent, cmd_context: CommandContext | None = None
    ) -> tuple[bool, str]:
        if not agent.db_server:
            raise RuntimeError(f"Agent {agent.name} not initialized")
        self.db_element.server = agent.db_server
        self.context.database.session.commit()
        self.need_update = True
        self.finish_init(cmd_context)
        is_error, error_message = agent.start_service(self.id)
        self.context.event_manager.trigger_event("service_updated", cmd_context)
        return is_error, error_message

    def unassign(self, cmd_context: CommandContext | None = None) -> None:
        agent = self.get_agent()
        if agent:
            agent.stop_service(self.id)
        self.db_element.server = None
        self.context.database.session.commit()
        self.context.event_manager.trigger_event("service_updated", cmd_context)

    def get_agent(self) -> Agent | None:
        for agent in self.context.app.agents:
            if (
                agent.db_server
                and self.db_element.server
                and agent.db_server.id == self.db_element.server.id
            ):
                return agent
        return None

    @staticmethod
    def get_from_id(service_id: int, context) -> "ServerService | None":
        for service in context.app.services.values():
            if service.db_element.id == service_id:
                return service
        return None

    @staticmethod
    def get_from_id_str(service_id: str) -> "ServerService | None":
        for service in get_current_context().app.services.values():
            if service.db_element.id_str == service_id:
                return service
        return None

    def build_service(self) -> bool:
        agent = self.get_agent()
        if agent:
            response = agent.send_pingpong({
                "type": "run_service_command",
                "service": self.id,
                "commands": [
                    "FREE::docker compose build",
                    "FREE::docker compose down",
                    "FREE::docker compose up"
                ]
            }, timeout=120)
            if response:
                return True
        return False
