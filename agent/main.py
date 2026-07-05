import os

from client import Client
from config import AgentConfig

with open(".agent_state", "w") as f:
    f.write("STARTING")

config_path = "./etc"
if os.getenv("CONFIG_FOLDER"):
    config_path = os.getenv("CONFIG_FOLDER", config_path)
cert_path = "etc/server.crt"
if os.getenv("CERT_PATH"):
    cert_path = os.getenv("CERT_PATH", cert_path)
if not os.path.exists(cert_path):
    print(f"Certificate file not found: {cert_path}")
    print("Please copy the certificate file to the specified path OR set the CERT_PATH environment variable")
    exit(1)
try:
    AgentConfig(config_path, cert_path)
    client = Client()
    client.connect()
except Exception as e:
    with open(".agent_state", "w") as f:
        f.write("FAILED")
    raise e
