class GeneratorBase:
    @staticmethod
    def install_deps() -> str:
        """
        Installs the dependencies required for the generator to run.
        Example: if it's a generator based on python, it might return "apt install -y ..."
        !! The only supported system (for now) is Debian-based (apt/apt-get) !!
        Returns:
            str: The command to install the dependencies.
        """
        return ""

    @staticmethod
    def ensure_valid_config(generator_config: dict) -> bool:
        """
        Ensures the generator config is valid.
        Args:
            generator_config (dict): The configuration (abstract) for the generator.
        Returns:
            bool: True if the config is valid, False otherwise.
        """
        return True

    @staticmethod
    def generate(generator_config: dict, arguments: list[str]) -> tuple[str, bool]:
        """
        Generates the commands to run the generator.
        Args:
            generator_config (dict): The configuration (abstract) for the generator.
            arguments (list[str]): The arguments to pass to the generator.
        Returns:
            str: The generated command. (sh!!)
            bool: True if the command needs to be run as root, False otherwise.
        """
        return "", False
