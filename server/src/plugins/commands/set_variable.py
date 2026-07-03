from helpers import get_current_context
from plugins.commands._template import CommandBase
from command_context import CommandContext
from database.models import UserVariable, UserVarNeedsUpdate
from services.service import ServerService

class SetUserVarCommand(CommandBase):
    NAME = "var:set"

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        if len(arguments) < 1:
            cmd_context.output_print(SetUserVarCommand.get_help())
            return False

        var_id = arguments[0]
        context = get_current_context()

        db_element = context.database.session.query(UserVariable).filter_by(id_str=var_id).first()

        if db_element is None:
            cmd_context.output_print(f"Variable '{var_id}' does not exist. Please add it to your configuration.")
            return False

        if len(arguments) < 2:
            value = cmd_context.output_input(f"Enter the value for '{var_id}': ")
        else:
            value = arguments[1]

        db_element.value = value
        context.database.session.commit()
        dependencies = context.database.session.query(UserVarNeedsUpdate).filter_by(user_variable=db_element).all()
        for dependency in dependencies:
            service = ServerService.get_from_id(dependency.service_id, context)
            if service is not None:
                print(f"Marking service '{service.name}' for update due to variable '{var_id}' change.")
                for config_file in service.config_files:
                    print(f"Regenerating config file '{config_file.path}' for service '{service.name}' due to variable update.")
                    config_file.regenerate(cmd_context)

        return True

    @staticmethod
    def get_help() -> str:
        return """
        This command sets the value of a user variable. (declared in the configuration file).
        !! this will regenerate all config files that depend on this variable !!
        Usage: var:set <variable_id> [value]"""
