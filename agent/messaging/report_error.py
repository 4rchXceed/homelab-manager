import logging

from config import AgentConfig

logger = logging.getLogger(__name__)
logger.setLevel(logging.DEBUG)
logger.addHandler(logging.StreamHandler())
logger.addHandler(logging.FileHandler("agent.log"))


def report_error(title: str, message: str, level: int = 0):
    if AgentConfig.instance:
        AgentConfig.instance.apobj.notify(title=f"[{level}/3]: {title}", body=message)


def log_error(message: str):
    logger.error(message)
