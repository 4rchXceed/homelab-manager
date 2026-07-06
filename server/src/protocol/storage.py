from asyncio.log import logger
from typing import TYPE_CHECKING
from error.exceptions import MissingConfigException

if TYPE_CHECKING:
    from protocol.agent import Agent


class ServerStorage:
    def __init__(self, id: str, config: dict[str,str], agent: "Agent") -> None:
        self.id = id
        if not config.get("path"):
            raise MissingConfigException(f"servers.$.storages.{id}.path")
        self.path = config.get("path", "")
        self.fallback = config.get("fallback", "fallback")
        self.do_not_create = config.get("do_not_create", False)
        self.agent = agent
        self.is_invalid = self.check()
        if self.is_invalid:
            logger.warning(f"Storage {self.id} is invalid, path: {self.path}. Will be using fallback storage: {self.fallback}")

    def check(self) -> bool:
        response = self.agent.send_pingpong({
            "type": "check_storage",
            "path": self.path,
            "can_create": not self.do_not_create
        })
        return response.get("invalid", False)
