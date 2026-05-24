# THIS IS A TEST IMPLEMENTATION AND IS NOT SUPPOSED TO BE USED IN PRODUCTION
import json
import socket
import threading
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

host = socket.gethostname()
port = 4398

server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server_socket.bind((host, port))
server_socket.listen(2)

queue = Queue()

api_key = "sjsdjs2"


def handle_client_send(conn, addr):
    data = conn.recv(1024).decode()
    if data != api_key:
        conn.close()
        return
    else:
        conn.sendall(b"OK")
    threading.Thread(target=handle_client_recv, args=(conn, addr)).start()
    while True:
        data = queue.get()
        if not data:
            continue
        conn.sendall(json.dumps(data).encode())
        queue.task_done()


def handle_client_recv(conn, addr):
    while True:
        data = conn.recv(1024).decode()
        if data:
            try:
                data_obj = json.loads(data)
                if data_obj.get("type", "") == "gen_config_report":
                    if data_obj.get("success", False):
                        logger.info(
                            f"[{data_obj.get('path', 'Unknown')}]: File generated successfully."
                        )
                    else:
                        logger.error(
                            f"[{data_obj.get('path', 'Unknown')}]: File generation failed. Details: {', '.join(data_obj.get('return_codes', []))}"
                        )
            except json.JSONDecodeError as e:
                logger.error(f"JSON decode error: {e}")


def handle_clients():
    while True:
        conn, addr = server_socket.accept()
        print(f"Connected by {addr}")
        threading.Thread(target=handle_client_send, args=(conn, addr)).start()


threading.Thread(target=handle_clients).start()

logger.info("Booting...")
config = load_config()
config_general = GeneralConfig(config)
db = DatabaseEngine(config_general.database)
generators = Generators(config["generators"])
context = HLMContext(db, generators, VARIABLE_PROVIDERS, queue, config_general)
set_current_context(context)

# Maybe: put the next lines in a separate file
services_obj = config.get("services", {})
services: dict[str, ServerService] = {}

for service, service_config in services_obj.items():
    services[service] = ServerService(service, service_config)

logger.info("Boot OK!")

stop = False

while not stop:
    command = input("admin@homelab-manager: ")
    if command == "exit":
        stop = True
    elif command == "config:sync":
        for service in services.values():
            print(f"Syncing {service.name} ({service.id})...")
            service.finish_init()
        print(
            "Some commands might have been queued. You should see the results soon."
        )  # TODO: Add timeout
