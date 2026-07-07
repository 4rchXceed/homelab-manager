from helpers import get_current_context
from plugins.commands._template import CommandBase
from command_context import CommandContext
from database.models import UserVariable, UserVarNeedsUpdate
from services.service import ServerService

class VarListCommand(CommandBase):
    NAME = "var:list"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        context = get_current_context()
        vars = context.database.session.query(UserVariable).all()
        cmd_context.output_print("Variables:")
        for var in vars:
            cmd_context.output_print(f"- {var.id_str}")
        return True

    @staticmethod
    def get_help() -> str:
        return """
        Lists all variables.
        Usage: var:list
        Formatted like:
            - key
            - key2
        Use var:get <key> to get the value of a variable.
        """
