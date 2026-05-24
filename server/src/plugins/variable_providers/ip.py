from database.models import Service
from error.exceptions import GenericConfigException, MissingConfigException
from helpers import get_current_context
from logger import logger
from plugins.variable_providers._template import VariableProvider


class IpVarProvider(VariableProvider):
    OPTIONS = {"has_frontend": False}

    @staticmethod
    def backend_process(data: dict, jsOutput: dict | None) -> str:
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
        return service.server.ip
