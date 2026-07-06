import time
from typing import TYPE_CHECKING
from error.exceptions import MissingConfigException, GenericConfigException
from helpers import parse_time, get_current_context
from database.models import BackupConfig

if TYPE_CHECKING:
    from services.service import ServerService


class ServiceBackupConfig:
    def __init__(self, config: dict, service: "ServerService"):
        self.service = service
        self.reload(config)

    def reload(self, config: dict):
        if config.get("id") is None:
            raise MissingConfigException(f"services.{self.service.id}.backups.$.id")
        if config.get("type") is None:
            raise MissingConfigException(f"services.{self.service.id}.backups.$.type")
        if config.get("maxSize") is None:
            raise MissingConfigException(f"services.{self.service.id}.backups.$.maxSize")
        self.id_str = config.get("id","")
        if not config.get("type") in ["full", "incremental"]:
            raise GenericConfigException(f"services.{self.service.id}.backups.$.type: Invalid backup type, must be one of: full, incremental")
        self.type = config.get("type","")
        self.max_size = config.get("maxSize", 0)
        self.max_age = parse_time(config.get("maxAge", "300y")) # We have a little bit of time I guess :)
        if config.get("schedule") is not None:
            self.schedule = parse_time(config.get("schedule", 0))
        else:
            self.schedule = None
        context = get_current_context()
        db_element = context.database.session.query(BackupConfig).filter_by(id_str=self.id_str).first()
        if db_element is None:
            db_element = BackupConfig(id=self.id_str, service_id=self.service.id, last=None, disabled=False) # TODO: disabled is never used
            context.database.session.add(db_element)
        self.last = db_element.last
        self.targets = [] # Will be set by runtime config

    def needs_backup(self) -> bool:
        if self.last is None:
            return True
        if self.schedule is not None and time.time() - self.last.timestamp() > self.schedule:
            return True
        return False
