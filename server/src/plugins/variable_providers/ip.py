from calendar import c
from typing import TYPE_CHECKING

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
    def backend_process(
        data: dict, jsOutput: dict | None, config_file: "ConfigFile"
    ) -> str:
        if data.get("get") is None:
            raise MissingConfigException("provider:type=ip->get key (missing)")
        get_key = data.get("get")
        context = get_current_context()
        service = (
            context.database.session.query(Service).filter_by(id_str=get_key).first()
        )
        if service is None:
            raise GenericConfigException(f"Service not found: id={get_key}")

        if service.server is None:
            if data.get("raiseNotFound", False):
                raise GenericConfigException(
                    f"No server associated with service: id={get_key}. Set raiseNotFound to false to suppress this warning."
                )
            else:
                logger.warning(
                    f'No server associated with service: id={get_key}. Will return dummy IP. To raise an error, set "raiseNotFound": true in the config.'
                )
            return "127.0.0.1"
        depends_on = (
            context.database.session.query(NeedsUpdate)
            .filter_by(
                service_trigger_id=service.id,
                service_updated_id=config_file.service.db_element.id,
            )
            .first()
        )
        if not depends_on:
            depends_on = NeedsUpdate(
                service_trigger_id=service.id,
                service_updated_id=config_file.service.db_element.id,
                last_ip=service.server.ip,
            )
            context.database.session.add(depends_on)
            context.database.session.commit()
        elif depends_on.last_ip != service.server.ip:
            depends_on.last_ip = service.server.ip
            context.database.session.commit()

        return service.server.ip
