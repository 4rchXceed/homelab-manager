import os

from command_context import CommandContext
from error.exceptions import GenericConfigException, MissingConfigException
from helpers import get_current_context
from logger import logger


class ConfigFile:
    def __init__(self, data: dict, service) -> None:
        self.service = service
        self.context = get_current_context()
        self.data = data
        self.path = self.data.get("path")
        if self.path is None:
            raise MissingConfigException("services.$.configFiles.$.path")
        self.generators_obj: list[dict] = self.data.get("generators", [])
        if len(self.generators_obj) == 0:
            logger.warning("No generators found for config file at " + self.path)
        self.real_path = os.path.join(
            self.context.config_general.services_folder,
            str(service.id),
            self.path + ".sample",
        )
        if not os.path.exists(self.real_path):
            raise GenericConfigException(
                f"Config file not found at {self.real_path}. Service: {service.id}, path: {self.path}. Hint: A .sample file with the same base name is required, so it doesn't override the original config file. It will be renamed to {self.path} when the config is written."
            )

    def handle_provider(
        self, provider_datas: dict, cmd_context: CommandContext | None = None
    ) -> str:
        if provider_datas.get("type") is None:
            raise MissingConfigException(
                f"services.$.configFiles.$.providers.$.type->configPath={self.path}"
            )
        provider_type = provider_datas.get("type", "")
        provider = self.context.providers.get(provider_type)
        if provider is None:
            raise GenericConfigException(
                f"Provider {provider_type} not found (available: {list(self.context.providers.keys())}). At config file {self.path}"
            )
        has_frontend = provider.OPTIONS.get("has_frontend", False)
        if has_frontend:
            if self.context.env == "cli":
                if cmd_context is None:
                    raise Exception(
                        f"cmd_context is required when using cli frontend for provider {provider_type}"
                    )
                datas = provider.cli_frontend(
                    provider_datas,
                    cmd_context.output_print,
                    cmd_context.output_input,
                    config_file=self,
                )
            else:
                # TODO: Complete this when WebUI is ready
                datas = {}
            return provider.backend_process(provider_datas, datas, config_file=self)
        else:
            return provider.backend_process(provider_datas, None, config_file=self)

    def regenerate(
        self, cmd_context: CommandContext | None = None
    ) -> list[dict] | None:
        responses = []
        response = self.context.send_from_service(
            self.service.id,
            {
                "type": "rewrite_config",
                "service": self.service.id,
                "path": self.path,
            },
        )
        for generator in self.generators_obj:
            generator_name = generator.get("generator")
            generator_arguments = generator.get("generatorArgs", [])
            if generator_name is None:
                logger.error(generator)
                raise MissingConfigException(
                    "services.$.configFiles.$.generators.$.generator"
                )
            if generator_arguments is None:
                generator_arguments = []
            generator = self.context.generators.generators.get(generator_name)
            if generator is None:
                raise GenericConfigException(
                    f"Generator {generator_name} not found (available: {list(self.context.generators.generators.keys())}). At config file {self.path}"
                )
            arguments = []
            for arg in generator_arguments:
                if isinstance(arg, dict):
                    arguments.append(self.handle_provider(arg, cmd_context))
                else:
                    arguments.append(arg)
            # TODO: For now all commands are run as root (since it's in a container)
            command = generator[1].generate(
                generator[0],
                arguments,
            )
            # print(command[0])  # TODO: Send these to the srv
            # self.context.message_queue.put(
            #     {
            #         "type": "gen_config",
            #         "service": self.service.id,
            #         "path": self.path,
            #         "commands": [command[0]],
            #     }
            # )
            try:
                response = self.context.send_from_service(
                    self.service.id,
                    {
                        "type": "gen_config",
                        "service": self.service.id,
                        "path": self.path,
                        "commands": [command[0]],
                    },
                    timeout=generator[0].get(
                        "timeout", 30
                    ),  # Since you sometimes need to install deps
                )
            except TimeoutError:
                response = None
            if not response:
                logger.error("Failed to send config regeneration request")
            else:
                responses.append(response)

        return responses
