import json
from collections import Counter

from database.models import Service
from error.exceptions import MissingConfigException
from helpers import get_current_context
from services.config_file import ConfigFile


class ServerService:
    def __init__(self, id: str, config: dict) -> None:
        self.id = id
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
            self.context.database.session.query(Service).filter_by(id_str=id).first()
        )
        if db_element is None:
            self.db_element = Service(id_str=id, name=self.name, last_config="{}")
            self.context.database.session.add(self.db_element)
            self.context.database.session.commit()
            self.need_update = True
        else:
            self.db_element = db_element

    def finish_init(self):
        """To avoid the position of the services in the config to matter"""
        if self.db_element.last_config != json.dumps(self.config) or self.need_update:
            if self.db_element.last_config is None:
                self.update({})
            else:
                self.update(json.loads(self.db_element.last_config))
            self.db_element.last_config = json.dumps(self.config)
            self.context.database.session.commit()
            self.need_update = False

    def update(self, old_config: dict) -> bool:
        changed = False
        if old_config.get("name") != self.name:
            self.db_element.name = self.name
            self.context.database.session.commit()
            changed = True
        if Counter(old_config.get("data", [])) != Counter(self.datas):
            # TODO: Add correct handling when self.data will be implemented
            self.db_element.last_config = json.dumps(self.config)
            self.context.database.session.commit()
            changed = True
        if json.dumps(old_config.get("configFiles", [])) != json.dumps(
            self.config_files_obj
        ):
            for config_file in self.config_files:
                config_file.regenerate()
            self.db_element.last_config = json.dumps(self.config)
            self.context.database.session.commit()
            changed = True

        return changed
