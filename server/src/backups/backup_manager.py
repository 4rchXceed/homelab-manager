import traceback
import threading
import time
import uuid
import socket
from services.backup_config import ServiceBackupConfig
from protocol.storage import ServerStorage
from helpers import get_current_context
from logger import logger
from command_context import CommandContext

class BackupManager:
    def __init__(self):
        self.context = get_current_context()
        self.backups = {}
        self.backup_socket = self.context.app.ssl_ctx.wrap_socket(socket.socket(socket.AF_INET, socket.SOCK_STREAM), server_side=True)
        self.backup_socket.bind(("0.0.0.0", self.context.config_general.backup_transfer_port))
        self.socket_thread_instance = threading.Thread(target=self.socket_thread, daemon=True)
        self.socket_thread_instance.start()

    def create_transaction(self) -> str:
        transaction_id = str(uuid.uuid4())
        self.backups[transaction_id] = {
            "clients": {}
        }
        return transaction_id

    def issue_restore(self, backup_config: ServiceBackupConfig, backup_storage: ServerStorage, cmd_context: CommandContext, backup_id: str | None, connect_timeout=20) -> bool:
        ok = self.send_request(True, backup_config, backup_storage, connect_timeout, cmd_context, backup_id)
        return ok

    def issue_backup(self, from_config: ServiceBackupConfig, to_storage: ServerStorage, connect_timeout=20) -> bool:
        return self.send_request(False, from_config, to_storage, connect_timeout, CommandContext())

    def send_request(self, is_restore: bool, from_config: ServiceBackupConfig, to_storage: ServerStorage, connect_timeout, cmd_context: CommandContext, backup_id: str | None = None) -> bool:
        transaction_id = self.create_transaction()
        service_agent = from_config.service.get_agent()
        backup_agent = to_storage.agent
        if not service_agent or not backup_agent:
            logger.error(f"service or backup agent not found: {service_agent}, {backup_agent}")
            return False
        r_service = []
        r_reciver = []
        backup_action = "init_backup"
        if is_restore:
            backup_action = "init_restore"
        def _t1(r_reciver):
            r_reciver.append(backup_agent.send_pingpong({
                "type": f"{backup_action}_as_storage",
                "service": from_config.service.id,
                "config": {
                    "max_size": from_config.max_size,
                    "max_age": from_config.max_age,
                    "backup_id": backup_id
                },
                "b_type": from_config.type,
                "transaction_id": transaction_id,
                "path": to_storage.path
            }, timeout=3600))
        def _t2(r_service):
            r_service.append(service_agent.send_pingpong({
                "type": f"{backup_action}_as_service",
                "service": from_config.service.id,
                "datas": from_config.service.datas,
                "config": {
                    "max_size": from_config.max_size,
                    "max_age": from_config.max_age,
                },
                "b_type": from_config.type,
                "transaction_id": transaction_id,
            }, timeout=3600))
        threading.Thread(target=_t1, args=(r_reciver,)).start()
        threading.Thread(target=_t2, args=(r_service,)).start()
        s = time.time()
        while not self.context.kill_switch and time.time() - s < connect_timeout and len(self.backups[transaction_id]["clients"]) < 2:
            time.sleep(0.5)
        return self.handle_request(is_restore, from_config, transaction_id, r_service, r_reciver, cmd_context)

    def handle_request(self, is_restore: bool, from_config: ServiceBackupConfig, transaction_id: str, r_service, r_reciver, cmd_context: CommandContext) -> bool:
        cmd_context.output_print(f"{'Restore' if is_restore else 'Backup'} initiated for {from_config.service.id} with transaction ID {transaction_id}. Waiting for clients to connect...")
        if len(self.backups[transaction_id]["clients"]) == 2:
            service: socket.socket = self.backups[transaction_id]["clients"]["service"]
            backup: socket.socket = self.backups[transaction_id]["clients"]["backup"]
            try:
                def other_thread_fn():
                    while not self.context.kill_switch:
                        data = backup.recv(1024)
                        if not data:
                            break
                        service.sendall(data)
                other_thread = threading.Thread(target=other_thread_fn, daemon=True)
                other_thread.start()
                service.sendall(b"START")
                while not self.context.kill_switch:
                    data = service.recv(1024)
                    if not data:
                        break
                    backup.sendall(data)
            except Exception: # When the backup is done, the socket is closed and an exception is raised
                service.close()
                backup.close()
            additional_message = r_reciver[0].get("message", "") if len(r_reciver) > 0 else ""
            cmd_context.output_print(f"{'Restore' if is_restore else 'Backup'}  completed for {from_config.service.id} with transaction ID {transaction_id}. Logs: {additional_message}")
            while len(r_service) == 0 or len(r_reciver) == 0:
                time.sleep(0.39)
            return r_service[0].get("success", False) # TODO: Handle the case when the backup is not successful
        else:
            additional_message = r_reciver[0].get("message", None) if len(r_reciver) > 0 else None
            if additional_message is None and len(r_service) > 0:
                additional_message = r_service[0].get("message", None)
            cmd_context.output_print(f"{'Restore' if is_restore else 'Backup'}  failed for {from_config.service.id} with transaction ID {transaction_id}. Not all clients connected in time. Additional message: {additional_message}")
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
                        if role in ["service", "backup"] and role not in self.backups[transaction_id]["clients"].keys():
                            logger.info(f"Client connected for transaction {transaction_id} as {role}")
                            client_socket.sendall(b"OK")
                            self.backups[transaction_id]["clients"][role] = client_socket
                        else:
                            client_socket.sendall(b"ERROR")
                    else:
                        client_socket.sendall(b"ERROR")
            except Exception as e:
                logger.info(e)
