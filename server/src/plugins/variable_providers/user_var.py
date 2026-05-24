from database.models import UserVariable
from error.exceptions import MissingConfigException
from helpers import get_current_context
from logger import logger
from plugins.variable_providers._template import VariableProvider


class UserVarProvider(VariableProvider):
    OPTIONS = {"has_frontend": True}

    @staticmethod
    def frontend_init(datas: dict) -> UserVariable | None:
        if datas.get("name") is None:
            logger.warning(
                f"No name provided for user variable: {datas}. User might not know what to write. Please provide a name."
            )
        if datas.get("id") is None:
            raise MissingConfigException("provider:type=UserVar->id key (missing)")
        context = get_current_context()
        user_var = (
            context.database.session.query(UserVariable)
            .filter_by(id_str=datas.get("id"))
            .first()
        )
        return user_var

    @staticmethod
    def cli_frontend(datas: dict) -> dict:
        user_var = UserVarProvider.frontend_init(datas)
        if user_var is None:
            var_name = datas.get("name", "No Name")
            value = input(f"Enter the value for the variable: {var_name}: ")
            return {"value": value}
        else:
            return {"value": user_var.value}

    @staticmethod
    def frontend_builder(datas: dict) -> str:
        user_var = UserVarProvider.frontend_init(datas)
        if user_var is None:
            var_name_safe = datas.get("name", "No Name").replace('"', "'")
            return f"""
            const variableName = "{var_name_safe}";
            const value = prompt("Enter value for {var_name_safe}: ");
            return {{value: value}};
            """
        else:
            if user_var.value is None:
                value_safe = ""
            else:
                value_safe = user_var.value.replace("`", "\\`")
            return f"""
            return {{value: `{value_safe}`}};
            """

    @staticmethod
    def backend_process(data: dict, jsOutput: dict | None) -> str:
        if jsOutput is None:
            return ""
        context = get_current_context()
        user_var = (
            context.database.session.query(UserVariable)
            .filter_by(id_str=data.get("id"))
            .first()
        )
        if user_var is None:
            user_var = UserVariable(id_str=data.get("id"), value=jsOutput.get("value"))
            context.database.session.add(user_var)
            context.database.session.commit()
        else:
            user_var.value = jsOutput.get("value")
            context.database.session.commit()
        return jsOutput.get("value", "")
