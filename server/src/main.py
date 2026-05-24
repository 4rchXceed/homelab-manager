from queue import Queue

from config.general import GeneralConfig
from config.load import load_config
from config_gen.generators import Generators
from context import HLMContext
from database.database import DatabaseEngine
from helpers import set_current_context
from logger import logger
from plugins.variable_providers.library import VARIABLE_PROVIDERS
from services.service import ServerService

# TODO: Put all of this in a class
logger.info("Booting...")
config = load_config()
config_general = GeneralConfig(config)
db = DatabaseEngine(config_general.database)
generators = Generators(config["generators"])
context = HLMContext(db, generators, VARIABLE_PROVIDERS, Queue(), config_general)
set_current_context(context)

# Maybe: put the next lines in a separate file
services_obj = config.get("services", {})
services = {}
for service, service_config in services_obj.items():
    services[service] = ServerService(service, service_config)

for service in services.values():
    service.finish_init()

logger.info("Boot OK!")
