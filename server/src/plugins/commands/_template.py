from typing import Callable

from command_context import CommandContext


class CommandBase:
    NAME = "command:base"  # The name (you can put everything, except spaces/other whitespace characters)

    @staticmethod
    def execute(arguments: list[str], cmd_context: CommandContext) -> bool:
        """
        Execute the command with the given arguments.
        You can use get_current_context() to get the current context. (with the database connection, server sockets, and much more)
        Parameters:
        - arguments: A list of strings representing the command arguments.
        - cmd_context: The command context. (see CommandContext in command_context.py)
        Returns:
        - Is the command successful?
        """
        return True

    @staticmethod
    def get_help() -> str:
        """
        Get the help text for the command.
        Returns:
        - A string representing the help text.
        """
        return ""
