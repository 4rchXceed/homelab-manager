from config.load import get_config


class MissingConfigException(Exception):
    def __init__(self, key_name: str) -> None:
        """Raised when a required config value is missing"""
        super().__init__(f"Config: {get_config()}. Missing config value: {key_name}")


class GenericConfigException(Exception):
    def __init__(self, message: str) -> None:
        """Raised when a config value is invalid"""
        super().__init__(f"Config: {get_config()}. {message}")


class ProgramStateError(Exception):
    def __init__(self, message: str) -> None:
        """Raised when the program state is invalid"""
        super().__init__(
            f"The program is in an invalid state (shouldn't happen): {message}"
        )


class TimeoutException(Exception):
    def __init__(self, message: str) -> None:
        """Raised when a timeout occurs"""
        super().__init__(f"The agent did not respond in time: {message} (TimeoutError)")
