from error.exceptions import MissingConfigException


class ConfigServers:
    def __init__(self, config: dict) -> None:
        self.config = config
        if not self.config.get("servers"):
            raise MissingConfigException("servers")
        self.servers: list[dict] = self.config["servers"]
