class MissingConfigFile(Exception):
    def __init__(self, config_path: str):
        self.config_path = config_path
        super().__init__(f"Config file not found: {config_path}")
