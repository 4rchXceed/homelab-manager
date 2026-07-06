from error.exceptions import GenericConfigException, MissingConfigException


class GeneralConfig:
    def __init__(self, json_datas: dict) -> None:
        self.reload(json_datas)

    def reload(self, json_datas: dict):
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
        self.backup_transfer_port = 4397
        if self.dict.get("net") is not None:
            self.server_port = self.dict.get("net", {}).get(
                "serverPort", self.server_port
            )
            self.file_server_port = self.dict.get("net", {}).get(
                "fileServerPort", self.file_server_port
            )
            self.backup_transfer_port = self.dict.get("net", {}).get(
                "backupTransferPort", self.backup_transfer_port
            )
        self.unix_socket_path = self.dict.get(
            "unixSocketPath",
            "/tmp/homelabmanager.sock",
        )
        self.startup_timeout = self.dict.get(
            "startupTimeout",
            120,
        )
        self.keepalive_interval = self.dict.get(
            "keepaliveInterval",
            5,
        )
        self.notification_urls = self.dict.get(
            "notifications",
            [],
        )
        if self.dict.get("binds") is None:
            raise MissingConfigException("config.binds")
        self.binds = self.dict.get(
            "binds",
            [],
        )
        if self.dict.get("fileserverAuth") is None:
            raise MissingConfigException("config.fileserverAuth")
        self.fileserver_auth = self.dict.get(
            "fileserverAuth",
            "admin:pass",
        )
        self.backup_check_interval = self.dict.get(
            "backupCheckInterval",
            1,
        )
