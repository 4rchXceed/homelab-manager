from command_context import CommandContext
from helpers import get_current_context
from plugins.commands._template import CommandBase


class RawCommand(CommandBase):
    NAME = "exec:raw"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        if len(arguments) == 0:
            cmd_context.output_print("Usage: exec:raw <action> <params>")
            return False
        action = arguments[0]
        context = get_current_context()
        if action == "service":
            if len(arguments) < 3 and (len(arguments) != 2 or arguments[2] != "list"):
                cmd_context.output_print(
                    "Usage: exec:raw service <agent_id> <action:stop,start,restart,list> <service_id>"
                )
                return False
            agent_id = arguments[1]
            action = arguments[2]
            service_id = arguments[3] if action != "list" else ""
            agent = None
            for app_agent in context.app.agents:
                if app_agent.id == agent_id:
                    agent = app_agent
                    break
            if agent is not None:
                if action == "stop":
                    cmd_context.output_print(str(agent.stop_service(service_id)))
                elif action == "start":
                    cmd_context.output_print(str(agent.start_service(service_id)))
                elif action == "restart":
                    cmd_context.output_print(str(agent.restart_service(service_id)))
                elif action == "list":
                    cmd_context.output_print(
                        f"Agent {agent_id} services: {agent.list_services()}"
                    )
                else:
                    cmd_context.output_print(f"Unknown action: {action}")
                    return False
            else:
                cmd_context.output_print(f"Agent {agent_id} not found")
                return False

        return True

    @staticmethod
    def get_help() -> str:
        return f"""
        This execute "raw" commands, bypassing most of the integrity checks.
        This is ONLY used when you have a app state problem as it might cause state corruption.
        Supported commands:
            - {RawCommand.NAME} service <agent_id> <action:stop,start,restart> <service_id>
            - {RawCommand.NAME} service <agent_id> list
        """
