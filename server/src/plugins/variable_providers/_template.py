from typing import TYPE_CHECKING, Callable

from command_context import CommandContext

if TYPE_CHECKING:
    from services.config_file import ConfigFile


class VariableProvider:
    """This is a template for a variable provider"""

    OPTIONS = {"has_frontend": True}

    @staticmethod
    def cli_frontend(
        datas: dict,
        cmd_context: CommandContext,
        config_file: "ConfigFile",
    ) -> dict:
        """
        Runs the CLI frontend for this variable provider
        This will be called when the config needs to be updated via CLI (no webui or ran from the terminal)
        Parameters:
            - datas: dict - the data that are written in the config
            - cmd_context: CommandContext - stdout+stdin utils functions (print/input())
            - config_file: the config OBJECT (useful to get the context, such as current service, etc.)
        Returns:
            - dict - data to be sent to the backend_process
        """
        return {}

    @staticmethod
    def frontend_builder(datas: dict, config_file: "ConfigFile") -> str:
        """
        Builds the frontend JavaScript for this variable provider
        Parameters:
            - datas: dict - the data that are written in the config
        Returns:
            - str - the frontend JavaScript (eval will be used to execute it SO BE CAREFUL !!)
        """
        return """
        // Frontend JavaScript goes here
        // What's returned will be sent to the backend_process method !! NEEDS TO BE AN OBJECT ({...}) !!
        """

    @staticmethod
    def backend_process(
        data: dict, jsOutput: dict | None, config_file: "ConfigFile"
    ) -> str:
        """
        Processes the data sent from the frontend and returns a string (value of the variable)
        Parameters:
            - data: dict - the data that are written in the config
            - jsOutput: dict | None - the output of the frontend JavaScript (the data to be processed). None if no frontend (OPTIONS["has_frontend"] is False)
        Returns:
            - str - the value of the variable
        """
        return ""

    @staticmethod
    def cleanup(data: dict, cmd_context: CommandContext, config_file: "ConfigFile"):
        """
        Cleanup done BEFORE re-generating the files
        Parameters:
            - data: dict - the datas that are written in the config
            - cmd_context: CommandContext - stdout+stdin utils functions (print/input())
            - config_file: the config OBJECT (useful to get the context, such as current service, etc.)
        Returns:
            - None
        """
        return
