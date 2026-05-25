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
        self.server_port = 4398
        self.file_server_port = 4399
        if self.dict.get("net") is not None:
            self.server_port = self.dict.get("net", {}).get(
                "serverPort", self.server_port
            )
            self.file_server_port = self.dict.get("net", {}).get(
                "fileServerPort", self.file_server_port
            )
        self.unix_socket_path = self.dict.get(
            "unix_socket_path",
            "/tmp/homelabmanager.sock",
        )
