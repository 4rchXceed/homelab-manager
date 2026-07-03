import secrets
import os
from typing import TYPE_CHECKING, Callable

from command_context import CommandContext
from database.models import UserVariable
from error.exceptions import MissingConfigException
from helpers import get_current_context
from logger import logger
from plugins.variable_providers._template import VariableProvider
from database.models import UserVarNeedsUpdate

if TYPE_CHECKING:
    from services.config_file import ConfigFile


class UserVarProvider(VariableProvider):
    OPTIONS = {"has_frontend": True}

    @staticmethod
    def get_db_element(user_var: UserVariable, config_file: "ConfigFile") -> UserVarNeedsUpdate|None:
        return get_current_context().database.session.query(UserVarNeedsUpdate).filter_by(user_variable_id=user_var.id, service_id=config_file.service.db_element.id).first()

    @staticmethod
    def frontend_init(datas: dict) -> UserVariable | None:
        if datas.get("name") is None:
            logger.warning(
                f"No name provided for user variable: {datas}. User might not know what to write. Please provide a name."
            )
        id = datas.get("id")
        if id is None:
            raise MissingConfigException("provider:type=UserVar->id key (missing)")
        context = get_current_context()
        user_var = (
            context.database.session.query(UserVariable).filter_by(id_str=id).first()
        )
        if user_var is None:
            if os.getenv("USERVAR_" + id.upper()):
                user_var = UserVariable(
                    id_str=id,
                    value=os.getenv("USERVAR_" + id.upper()),
                )
                context.database.session.add(user_var)
                context.database.session.commit()
        return user_var

    @staticmethod
    def cli_frontend(
        datas: dict,
        cmd_context: CommandContext,
        config_file: "ConfigFile",
    ) -> dict:
        user_var = UserVarProvider.frontend_init(datas)
        if user_var is None:
            var_name = datas.get("name", "No Name")
            value = cmd_context.output_input(
                f"Enter the value for the variable: (id: {datas.get('id')}) {var_name}: "
            )
            return {"value": value}
        else:
            return {"value": user_var.value}

    @staticmethod
    def frontend_builder(
        datas: dict,
        config_file: "ConfigFile",
    ) -> str:
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
    def backend_process(
        data: dict,
        jsOutput: dict | None,
        config_file: "ConfigFile",
    ) -> str:
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

        element = UserVarProvider.get_db_element(user_var, config_file)
        if element is None:
            element = UserVarNeedsUpdate(
                user_variable_id=user_var.id, service_id=config_file.service.db_element.id, last_value=jsOutput.get("value", "")
            )
            context.database.session.add(element)
            context.database.session.commit()
        else:
            element.last_value = jsOutput.get("value", "")
            context.database.session.commit()

        return jsOutput.get("value", "")

    @staticmethod
    def cleanup(data: dict, cmd_context: CommandContext, config_file: "ConfigFile"):
        return
