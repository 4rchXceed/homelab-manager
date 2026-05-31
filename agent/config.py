import json
import os

import apprise


class AgentConfig:
    instance = None

    def __init__(self, configs_path: str):
        if AgentConfig.instance is not None:
            raise RuntimeError("AgentConfig is a singleton")
        AgentConfig.instance = self

        self.configs_path = configs_path
        server_config_path = os.path.join(configs_path, "server.json")
        report_notif_path = os.path.join(configs_path, "report.json")
        self.apobj = apprise.Apprise()

        if not os.path.exists(server_config_path):
            raise FileNotFoundError(f"Config file not found: {server_config_path}")
        if not os.path.exists(report_notif_path):
            raise FileNotFoundError(f"Config file not found: {report_notif_path}")

        with open(server_config_path, "r") as f:
            self.server_config = json.load(f)
        with open(report_notif_path, "r") as f:
            self.report_notif = json.load(f)

        self.server = {
            "host": self.server_config["host"],
            "port": self.server_config["port"],
            "api_key": self.server_config["api_key"],
            "fsport": self.server_config.get("file_server_port", 4399),
            "keepalive_interval": self.server_config.get("keepalive_interval", 10),
        }

        self.services_folder = self.server_config.get("services_folder", "./services")

        self.apprise_urls = self.report_notif["apprise_url"]
        for url in self.apprise_urls:
            self.apobj.add(url)

        self.id = self.server_config["this_id"]
        self.name = self.server_config["this"]
