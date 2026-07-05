from command_context import CommandContext
from plugins.commands._template import CommandBase
from services.service import ServerService


class ServiceBuildCommand(CommandBase):
    NAME = "service:build"
    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        if len(arguments) != 1:
            cmd_context.output_print(ServiceBuildCommand.get_help())
            return False
        cmd_context.output_print(f"Building service: {arguments[0]}")
        service = ServerService.get_from_id_str(arguments[0])
        if not service:
            cmd_context.output_print("Failed to find service.")
            return False

        success = service.build_service()
        if not success:
            cmd_context.output_print("Failed to build service.")
            return False
        cmd_context.output_print("Service built successfully.")
        return True

    @staticmethod
    def get_help() -> str:
        return """
        Build a service. (docker compose build)
        - Usage: service:build <service_name>
        """
