from helpers import get_current_context
from plugins.commands._template import CommandBase
from command_context import CommandContext
from database.models import UserVariable, UserVarNeedsUpdate
from services.service import ServerService

class VarGetCommand(CommandBase):
    NAME = "var:get"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        if len(arguments) != 1:
            cmd_context.output_print(VarGetCommand.get_help())
            return False
        var_id = arguments[0]
        context = get_current_context()
        vars = context.database.session.query(UserVariable).filter_by(id_str=var_id).first()
        if vars is None:
            cmd_context.output_print(f"Variable with id {var_id} not found.")
            return False
        cmd_context.output_print(f"Variable {var_id}: {vars.value}")
        return True

    @staticmethod
    def get_help() -> str:
        return """
        Gets a variable's value.
        Usage: var:get <var_id>
        You can get a list of all variables by using the command var:list
        You can set a variable's value by using the command var:set
        """
