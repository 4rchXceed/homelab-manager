import json
from collections import Counter

from command_context import CommandContext
from database.models import Service
from error.exceptions import MissingConfigException
from helpers import get_current_context
from protocol.agent import Agent
from services.config_file import ConfigFile


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
            self.db_element = Service(id_str=self.id, name=self.name, last_config="{}")
            self.context.database.session.add(self.db_element)
            self.context.database.session.commit()
            self.need_update = True
        else:
            self.db_element = db_element

    def finish_init(self, cmd_context: CommandContext | None = None) -> list[str]:
        """To avoid the position of the services in the config to matter"""
        requests = []
        if self.db_element.last_config != json.dumps(self.config) or self.need_update:
            if self.db_element.last_config is None:
                requests.extend(self.update({}, cmd_context))
            else:
                requests.extend(
                    self.update(json.loads(self.db_element.last_config), cmd_context)
                )
            self.db_element.last_config = json.dumps(self.config)
            self.context.database.session.commit()
            self.need_update = False
        return requests

    def update(
        self, old_config: dict, cmd_context: CommandContext | None = None
    ) -> list[str]:
        requests = []
        if old_config.get("name") != self.name:
            self.db_element.name = self.name
            self.context.database.session.commit()
        if Counter(old_config.get("data", [])) != Counter(self.datas):
            # TODO: Add correct handling when self.data will be implemented
            self.db_element.last_config = json.dumps(self.config)
            self.context.database.session.commit()
        if json.dumps(old_config.get("configFiles", [])) != json.dumps(
            self.config_files_obj
        ):
            for config_file in self.config_files:
                request_uuid = config_file.regenerate(cmd_context)
                if request_uuid:
                    requests.append(request_uuid)

            self.db_element.last_config = json.dumps(self.config)
            self.context.database.session.commit()

        return requests

    def start_on(self, agent: Agent) -> tuple[bool, str]:
        # For now, we don't have the docker compose up made, so we just return an empty list
        # But we will assign it in the database
        if not agent.db_server:
            raise RuntimeError(f"Agent {agent.name} not initialized")
        is_error, error_message = agent.start_service(self.id)
        self.db_element.server = agent.db_server
        self.context.database.session.commit()
        return is_error, error_message

    def unassign(self) -> None:
        self.db_element.server = None
        self.context.database.session.commit()
        agent = self.get_agent()
        if agent:
            agent.stop_service(self.id)

    def get_agent(self) -> Agent | None:
        for agent in self.context.app.agents:
            if (
                agent.db_server
                and self.db_element.server
                and agent.db_server.id == self.db_element.server.id
            ):
                return agent
        return None
