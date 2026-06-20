from plugins.generators_bases.bash import BashGeneratorBase
from plugins.generators_bases.bash_unsandboxed import BashUnsandboxedGeneratorBase

PLUGINS = {"bash": BashGeneratorBase, "bashUnsandboxed": BashUnsandboxedGeneratorBase}
