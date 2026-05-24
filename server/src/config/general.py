from error.exceptions import GenericConfigException, MissingConfigException


class GeneralConfig:
    def __init__(self, json_datas: dict) -> None:
        if json_datas.get("config") is None:
            raise MissingConfigException("config")
        self.dict = json_datas.get("config")

        if not isinstance(self.dict, dict):
            raise GenericConfigException("config must be a dictionary")

        # Actual config
        self.services_folder = self.dict.get(
            "servicesFolder",
            "services",
        )
        if self.dict.get("database") is None:
            raise MissingConfigException("config.database")
        self.database = {
            "host": self.dict.get("database", {}).get("postgresHost"),
            "port": self.dict.get("database", {}).get("postgresPort"),
            "username": self.dict.get("database", {}).get("postgresUsername"),
            "password": self.dict.get("database", {}).get("postgresPassword"),
            "database": self.dict.get("database", {}).get("postgresDbName"),
        }
