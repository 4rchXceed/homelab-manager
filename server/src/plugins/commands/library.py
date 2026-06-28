from plugins.commands._template import CommandBase
from plugins.commands.config_regen import RegenConfigCommand
from plugins.commands.config_reload import ConfigReloadCommand
from plugins.commands.config_runtime import ConfigRuntimeCommand
from plugins.commands.config_sync import ConfigSyncCommand
from plugins.commands.raw_command import RawCommand
from plugins.commands.server_add import ServerAddCommand
from plugins.commands.service_assign import ServiceAssignCommand
from plugins.commands.service_unassign import ServiceUnassignCommand
from plugins.commands.services_list import ServiceListCommand
from plugins.commands.services_sync import ServicesSync
from plugins.commands.emergency_procedure import EmergencyProcedureCommand

COMMANDS: list[type[CommandBase]] = [
    ConfigSyncCommand,
    ServiceAssignCommand,
    ServerAddCommand,
    ConfigReloadCommand,
    ServicesSync,
    ServiceUnassignCommand,
    ServiceListCommand,
    RawCommand,
    ConfigRuntimeCommand,
    RegenConfigCommand,
    EmergencyProcedureCommand
]
