from plugins.generators_bases.bash import BashGeneratorBase
from plugins.generators_bases.bash_unsandboxed import BashUnsandboxedGeneratorBase
from plugins.generators_bases._template import GeneratorBase

PLUGINS: dict[str, type[GeneratorBase]] = {"bash": BashGeneratorBase, "bashUnsandboxed": BashUnsandboxedGeneratorBase}
