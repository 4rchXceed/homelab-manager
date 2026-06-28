import threading
import time

from database.models import Server
from helpers import get_current_context
from logger import logger
from plugins.emergency_events._template import EmergencyEventTemplate
from protocol.agent import Agent


class ServerOfflineEmergencyEvent(EmergencyEventTemplate):
    def inject(self, config: dict):
        if not config.get("server"):
            logger.warning(
                "No server specified for server_offline listener (probably in emergency_proc.json config)"
            )
            return
        self.server = config.get("server")
        self.timeout = config.get("timeout", 60)
        self.context = get_current_context()
        self.thread = threading.Thread(target=self.wait)
        self.thread.start()

    def wait(self):
        time.sleep(self.timeout)
        if self.server == "*":
            servers = self.context.database.session.query(Server).filter_by(
                disabled=False
            )
            for server in servers:
                id_str = server.id_str
                agent = Agent.get_from_id_str(id_str)
                if not agent:
                    self.fire(
                        {"server": f"{id_str} ({server.name})", "server_id_str": id_str}
                    )
        elif self.server is not None:
            agent = Agent.get_from_id_str(self.server)
            if not agent:
                self.fire({"server": self.server, "server_id_str": self.server})

    def cancel(self, args: dict[str, str]):
        self.thread.join()
