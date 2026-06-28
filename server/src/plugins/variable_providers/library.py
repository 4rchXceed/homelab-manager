from plugins.variable_providers.ip import IpVarProvider
from plugins.variable_providers.user_var import UserVarProvider
from plugins.variable_providers._template import VariableProvider

VARIABLE_PROVIDERS: dict[str, type[VariableProvider]] = {"ip": IpVarProvider, "userVar": UserVarProvider}
