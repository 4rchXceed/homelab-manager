import json
import os

from command_context import CommandContext
from config.load import get_config
from helpers import get_current_context
from logger import logger
from protocol.agent import Agent
from services.service import ServerService


class RuntimeConfig:
    def init(self) -> None:
        self.context = get_current_context()
        self.load_config()

    def reload(self, cmd_context: CommandContext):
        self.load_config()
        # Check for modifications and additions on assignments
        for service_id, server_id in self.assignments.items():
            service = ServerService.get_from_id_str(service_id)
            agent = Agent.get_from_id_str(server_id)
            if not service or not agent or not agent.db_server:
                cmd_context.output_print(
                    f"Either {service_id} is not a valid service or {server_id} is not a valid agent/server -> ignoring assignment"
                )
            else:
                if not service.db_element.server:
                    cmd_context.output_print(f"Starting {service_id} on {server_id}...")
                    service.start_on(agent, cmd_context)
                elif service.db_element.server.id != agent.db_server.id:
                    cmd_context.output_print(
                        f"Stopping {service_id} on {service.db_element.server.id_str}..."
                    )
                    service.unassign()
                    cmd_context.output_print(f"Starting {service_id} on {server_id}...")
                    service.start_on(agent, cmd_context)
                else:
                    cmd_context.output_print(
                        f"{service_id} is already on {server_id} -> no action required"
                    )
        for service_id, service in self.context.app.services.items():
            if service_id not in self.assignments.keys():
                if service.db_element.server:
                    cmd_context.output_print(
                        f"Stopping {service_id} on {service.db_element.server.id_str}..."
                    )
                    service.unassign()

    def dump(self):
        # Dump the current config to the file !! it overwrites the file entirely
        self.assignments.clear()
        for service_id, service in self.context.app.services.items():
            if service.db_element.server:
                self.assignments[service_id] = service.db_element.server.id_str
        self.config_raw = {"assignments": self.assignments}
        with open(self.config_path, "w", encoding="utf-8") as f_dst:
            json.dump(self.config_raw, f_dst, indent=4)

    def load_config(self):
        runtime_config = os.getenv("RUNTIME_CONFIG_FILE", None)
        if runtime_config is None:
            normal_config = get_config()
            runtime_config = os.path.join(
                os.path.dirname(normal_config), "runtime.json"
            )
        self.config_path = runtime_config

        if not os.path.exists(runtime_config):
            logger.warning(f"Auto generating runtime config at: {runtime_config}")
            with open(runtime_config, "w", encoding="utf-8") as f_dst:
                f_dst.write("{}")

        with open(runtime_config, "r", encoding="utf-8") as f:
            self.config_raw = json.load(f)
        self.assignments: dict[str, str] = self.config_raw.get("assignments", {})
