from plugins.generators_bases._template import GeneratorBase


class BashUnsandboxedGeneratorBase(GeneratorBase):
    @staticmethod
    def ensure_valid_config(generator_config: dict) -> bool:

        if generator_config.get("run") is None:
            return False
        return True

    @staticmethod
    def generate(generator_config: dict, arguments: list[str]) -> tuple[str, bool]:

        command = "FREE::"
        for i, arg in enumerate(arguments):
            command += f'\nARG{i + 1}="{arg.replace('"', '\\"')}"'
        command += "\n"

        deps = generator_config.get("installDeps")
        run = generator_config.get("run")
        if deps is not None:
            command += deps + " && "
        if run is not None:
            command += run
        return command, generator_config.get("needsRoot", False)
