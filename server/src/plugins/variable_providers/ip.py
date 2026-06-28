from calendar import c
from typing import TYPE_CHECKING

from command_context import CommandContext
from database.models import NeedsUpdate, Service
from error.exceptions import GenericConfigException, MissingConfigException
from helpers import get_current_context
from logger import logger
from plugins.variable_providers._template import VariableProvider

if TYPE_CHECKING:
    from services.config_file import ConfigFile


class IpVarProvider(VariableProvider):
    OPTIONS = {"has_frontend": False}

    @staticmethod
    def get_db_elements(
        data: dict, config_file: "ConfigFile"
    ) -> tuple[NeedsUpdate | None, Service | None]:
        if data.get("get") is None:
            raise MissingConfigException("provider:type=ip->get key (missing)")
        get_key = data.get("get")
        context = get_current_context()
        service = (
            context.database.session.query(Service)
            .filter_by(disabled=False)
            .filter_by(id_str=get_key)
            .first()
        )
        if service is None:
            raise GenericConfigException(f"Service not found: id={get_key}")

        db_elem = (
            context.database.session.query(NeedsUpdate)
            .filter_by(
                service_trigger_id=service.id,
                service_updated_id=config_file.service.db_element.id,
            )
            .first()
        )
        return db_elem, service

    @staticmethod
    def backend_process(
        data: dict, jsOutput: dict | None, config_file: "ConfigFile"
    ) -> str:
        context = get_current_context()
        depends_on, service = IpVarProvider.get_db_elements(data, config_file)
        if not service:
            return "127.0.0.1"
        if not depends_on:
            ip = "127.0.0.1"
            if service.server is not None:
                ip = service.server.ip
            depends_on = NeedsUpdate(
                service_trigger_id=service.id,
                service_updated_id=config_file.service.db_element.id,
                last_ip=ip,
            )
            context.database.session.add(depends_on)
            context.database.session.commit()
        elif service.server is not None and depends_on.last_ip != service.server.ip:
            depends_on.last_ip = service.server.ip
            context.database.session.commit()

        if service.server is None:
            logger.warning(
                f"No server associated with service: id={data.get('get')}. Will return dummy IP."
            )
            return "127.0.0.1"
        return service.server.ip

    @staticmethod
    def cleanup(data: dict, cmd_context: CommandContext, config_file: "ConfigFile"):
        context = get_current_context()
        depends_on, _ = IpVarProvider.get_db_elements(data, config_file)
        if depends_on:
            context.database.session.delete(depends_on)
            context.database.session.commit()
