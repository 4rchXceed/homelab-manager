from error.exceptions import GenericConfigException, MissingConfigException
from plugins.generators_bases._template import GeneratorBase
from plugins.generators_bases.library import PLUGINS


class Generators:
    def __init__(self, generators: dict[str, dict]):
        self.generators_obj = generators
        self.generators: dict[str, tuple[dict, type[GeneratorBase]]] = {}
        for generator_id, generator in generators.items():
            base = generator.get("base")
            if base is None:
                raise MissingConfigException(
                    "services.$.configFiles.$.generators.$.base"
                )
            plugin = PLUGINS.get(base)
            if plugin is None:
                raise GenericConfigException(
                    f"Base {base} not found (generator.{generator_id})"
                )
            self.generators[generator_id] = (
                generator,
                plugin,
            )
