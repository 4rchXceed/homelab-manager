import logging
import os

import colorlog

handler = colorlog.StreamHandler()
handler.setFormatter(
    colorlog.ColoredFormatter("%(log_color)s%(levelname)s:%(name)s:%(message)s")
)
logger = logging.getLogger(__name__)
logger.setLevel(logging.DEBUG)
logger.addHandler(handler)
if not os.path.exists("logs"):
    os.makedirs("logs")
logger.addHandler(logging.FileHandler("logs/log.txt"))
logger.info("Logger initialized")
