import os

from client import Client
from config import AgentConfig

config_path = "./etc"
if os.getenv("CONFIG_FOLDER"):
    config_path = os.getenv("CONFIG_FOLDER", config_path)
AgentConfig(config_path)
client = Client()
client.connect()
