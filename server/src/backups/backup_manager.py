import threading
import time
import uuid
import socket
from services.backup_config import ServiceBackupConfig
from protocol.storage import ServerStorage
from helpers import get_current_context
from logger import logger

class BackupManager:
    def __init__(self):
        self.context = get_current_context()
        self.backups = {}
        self.backup_socket = self.context.app.ssl_ctx.wrap_socket(socket.socket(socket.AF_INET, socket.SOCK_STREAM), server_side=True)
        self.backup_socket.bind(("0.0.0.0", self.context.config_general.backup_transfer_port))
        self.socket_thread_instance = threading.Thread(target=self.socket_thread, daemon=True)
        self.socket_thread_instance.start()

    def issue_backup(self, from_config: ServiceBackupConfig, to_storage: ServerStorage, connect_timeout=20) -> bool:
        transaction_id = str(uuid.uuid4())
        self.backups[transaction_id] = {
            "from_config": from_config,
            "to_storage": to_storage,
            "clients": {}
        }

        sender_agent = from_config.service.get_agent()
        receiver_agent = to_storage.agent
        if not sender_agent or not receiver_agent:
            logger.error(f"Sender or receiver agent not found: {sender_agent}, {receiver_agent}")
            return False
        receiver_agent.send({
            "type": "init_backup_as_receiver",
            "service": from_config.service.id,
            "config": {
                "max_size": from_config.max_age,
                "max_age": from_config.max_age,
            },
            "b_type": from_config.type,
            "transaction_id": transaction_id,
            "path": to_storage.path
        })
        sender_agent.send({
            "type": "init_backup_as_sender",
            "service": from_config.service.id,
            "datas": from_config.service.datas,
            "config": {
                "max_size": from_config.max_age,
                "max_age": from_config.max_age,
            },
            "b_type": from_config.type,
            "transaction_id": transaction_id,
        })

        s = time.time()
        while not self.context.kill_switch and time.time() - s < connect_timeout and len(self.backups[transaction_id]["clients"]) < 2:
            time.sleep(0.5)
        if len(self.backups[transaction_id]["clients"]) == 2:
            sender: socket.socket = self.backups[transaction_id]["clients"]["sender"]
            receiver: socket.socket = self.backups[transaction_id]["clients"]["receiver"]
            try:
                sender.sendall(b"START")
                while not self.context.kill_switch:
                    data = sender.recv(1024)
                    if not data:
                        break
                    receiver.sendall(data)
            except Exception: # When the backup is done, the socket is closed and an exception is raised
                sender.close()
                receiver.close()
            return True # TODO: Handle the case when the backup is not successful
        else:
            return False

    def socket_thread(self) -> None:
        self.backup_socket.listen(2)
        logger.info(f"Backup socket listening on port {self.context.config_general.backup_transfer_port}")
        while not self.context.kill_switch:
            try:
                client_socket, _ = self.backup_socket.accept()
                datas = client_socket.recv(1024)
                if datas:
                    datas_str = datas.decode()
                    transaction_id = datas_str.split(":")[0]
                    role = datas_str.split(":")[1]
                    if transaction_id in self.backups:
                        if role in ["sender", "receiver"] and role not in self.backups[transaction_id]["clients"].keys():
                            client_socket.sendall(b"OK")
                            self.backups[transaction_id]["clients"][role] = client_socket
                        else:
                            client_socket.sendall(b"ERROR")
                    else:
                        client_socket.sendall(b"ERROR")
            except Exception as e:
                logger.info(e)
