import json
import socket
from ssl import SSLContext, SSLSocket
import shutil
import time
from config import AgentConfig
import os
from messaging.log import debug
class BackupManager:
    def verify_path(self, path: str, can_create: bool) -> bool:
        if not os.path.exists(path):
            if can_create:
                try:
                    os.makedirs(path, exist_ok=True)
                except OSError:
                    return False
            else:
                return False
        if os.path.isdir(path):
            return os.access(path, os.W_OK)
        return False

    def analyze_directory(self, path: str, exclude: list[str]) -> dict:
        summary = {}
        if os.path.exists(path) and os.path.isdir(path):
            for files in os.listdir(path):
                file_path = os.path.join(path, files)
                if any(os.path.abspath(file_path).startswith(os.path.abspath(os.path.join(path, ex))) for ex in exclude):
                    continue
                if os.path.isfile(file_path):
                    try:
                        modification_time = os.path.getmtime(file_path)
                        hash = str(modification_time)
                        summary[files] = hash
                    except OSError:
                        continue
                elif os.path.isdir(file_path):
                    summary[files] = self.analyze_directory(file_path, exclude)
        return summary

    def create_backup_summary_client(self, service: str, datas: list[str]) -> dict:
        summary = {}
        if AgentConfig.instance:
            service_path = os.path.join(AgentConfig.instance.services_folder, service)
            exclude = [data[1:] for data in datas if data.startswith("!")]
            datas = [data for data in datas if not data.startswith("!")]
            datas_dirs = [os.path.join(service_path, data) for data in datas]
            for data_dir in datas_dirs:
                if os.path.exists(data_dir) and os.path.isdir(data_dir):
                    current_summary = self.analyze_directory(data_dir, exclude)
                    last_current = None
                    current = summary
                    clean_path = os.path.abspath(data_dir).removeprefix(os.path.abspath(service_path)).removeprefix("/")
                    if clean_path == "":
                        summary = current_summary
                    else:
                        for part in clean_path.split("/"):
                            if isinstance(current, dict):
                                last_current = current
                                if part not in summary:
                                    current[part] = {}
                                    current = current[part]

                        if last_current is not None:
                            last_current[part] = current_summary
        return summary

    def files_to_summary(self, parent: str, files: list[str]) -> dict:
        summary = {}
        for file in files:
            try:
                modification_time = os.path.getmtime(os.path.join(parent, file))
                hash = str(modification_time)
                last = None
                current = summary
                for part in file.split("/")[1:]: # Skip the first part, which is the incremental folder name
                    last = current
                    if part not in current:
                        current[part] = {}
                    current = current[part]
                if last is not None:
                    last[part] = [hash, file]
            except OSError:
                continue
        return summary

    def compare_summaries(self, summary_local: dict, summary_distant: dict) -> list[str]:
        changes = []
        for key, value in summary_local.items():
            if key not in summary_distant:
                changes.append(key)
            elif isinstance(value, dict) and isinstance(summary_distant[key], dict):
                sub_changes = self.compare_summaries(value, summary_distant[key])
                changes.extend([os.path.join(key, sub_change) for sub_change in sub_changes])
            elif value[0] != summary_distant[key]:
                changes.append(key)
        return changes

    def incremental_check_for_changes(self, backups_folder: str, service: str, summary_distant: dict) -> list[str]:
        backups_folder = backups_folder.removesuffix("/")  # Remove trailing slash if present
        backup_folder = os.path.join(backups_folder, service, "incremental")
        if not os.path.exists(backup_folder):
            os.makedirs(backup_folder, exist_ok=True)
            return self.compare_summaries({}, summary_distant)
        base_backup_folder = os.path.join(backup_folder, "base") # The "base" folder contains the full backup of the service, after that, the incremental backups are stored in 1, 2, 3, ... folders
        local_files = []
        all_versions_folders = self.all_versions_folders(backup_folder)
        if os.path.exists(base_backup_folder) and os.path.isdir(base_backup_folder):
            for root, _, files_in_dir in os.walk(base_backup_folder):
                for file in files_in_dir:
                    file_path = os.path.join(root, file)
                    local_files.append(os.path.abspath(file_path).removeprefix(os.path.abspath(backup_folder)).removeprefix("/"))
        for version_folder in all_versions_folders:
            version_folder_path = os.path.join(backup_folder, version_folder)
            for root, _, files_in_dir in os.walk(version_folder_path):
                for file in files_in_dir:
                    file_path = os.path.join(root, file)
                    local_files.append(os.path.abspath(file_path).removeprefix(os.path.abspath(backup_folder)).removeprefix("/"))
        summary_local = self.files_to_summary(backup_folder, local_files)
        return self.compare_summaries(summary_local, summary_distant)

    def all_versions_folders(self, backup_folder: str) -> list[str]:
        all_versions_folders = [f for f in os.listdir(backup_folder) if os.path.isdir(os.path.join(backup_folder, f)) and f.isdigit()]
        all_versions_folders.sort(key=lambda x: int(x))
        return all_versions_folders

    def after_incremential_cleanup(self, backups_folder: str, service: str, max_size: int, max_age: int):
        backups_folder = backups_folder.removesuffix("/")  # Remove trailing slash if present
        backup_folder = os.path.join(backups_folder, service, "incremental")
        while (os.path.getctime(backup_folder) < time.time()-max_age or self.folder_size(backup_folder) > max_size) and len(self.all_versions_folders(backup_folder)) > 0: # TODO: self.folder_size can take a verrry long time
            oldest_folder = self.all_versions_folders(backup_folder)[0]
            oldest_folder_path = os.path.join(backup_folder, oldest_folder)
            if os.path.exists(oldest_folder_path) and os.path.isdir(oldest_folder_path):
                try:
                    for root, dirs, files in os.walk(oldest_folder_path, topdown=False):
                        for name in files:
                            full_path = os.path.join(root, name)
                            full_path_base = os.path.join(backup_folder, "base", os.path.relpath(full_path, oldest_folder_path))
                            if not os.path.exists(os.path.dirname(full_path_base)):
                                os.makedirs(os.path.dirname(full_path_base), exist_ok=True)
                            shutil.copy2(full_path, full_path_base) # Copy the file to the base folder before deleting it, so the incremental backup is not lost
                    shutil.rmtree(oldest_folder_path)
                except OSError as e:
                    debug(f"Error deleting folder {oldest_folder_path}: {e}")

    def after_full_cleanup(self, backups_folder: str, service: str, max_size: int, max_age: int):
        backups_folder = backups_folder.removesuffix("/")  # Remove trailing slash if present
        backup_folder = os.path.join(backups_folder, service, "full")
        while (os.path.getctime(backup_folder) < time.time()-max_age or self.folder_size(backup_folder) > max_size) and len(self.all_versions_folders(backup_folder)) > 0: # TODO: self.folder_size can take a verrry long time
            oldest_folder = self.all_versions_folders(backup_folder)[0]
            oldest_folder_path = os.path.join(backup_folder, oldest_folder)
            if os.path.exists(oldest_folder_path) and os.path.isdir(oldest_folder_path):
                try:
                    shutil.rmtree(oldest_folder_path)
                except OSError as e:
                    debug(f"Error deleting folder {oldest_folder_path}: {e}")

    def folder_size(self, path: str) -> float:
        total_size = 0
        for dirpath, _, filenames in os.walk(path):
            for f in filenames:
                fp = os.path.join(dirpath, f)
                if os.path.exists(fp):
                    total_size += os.path.getsize(fp)/1024/1024 # Size in MB
        return total_size

    def send_file(self, sock: SSLSocket, file_path: str, file_path_to_send: str):
        fsize = os.path.getsize(file_path)
        sock.sendall(f"{fsize}:{file_path_to_send}\n".encode())
        with open(file_path, "rb") as f:
            while True:
                bytes_read = f.read(4096)
                if not bytes_read:
                    break
                sock.sendall(bytes_read)
        # ack = sock.recv(1024).decode()
        # if ack != "OK":
        #     debug(f"Error sending file {file_path}: {ack}")
        # return ack == "OK"

    def recv_file(self, sock: SSLSocket, save_path: str):
        file_infos = self.buffered_recv(sock)
        if not file_infos:
            return
        fsize_str, file_path = file_infos.split(":", 1)
        fsize = int(fsize_str)
        full_save_path = os.path.join(save_path, file_path)
        os.makedirs(os.path.dirname(full_save_path), exist_ok=True)
        with open(full_save_path, "wb") as f:
            bytes_received = 0
            while bytes_received < fsize:
                chunk = sock.recv(min(4096, fsize - bytes_received))
                if not chunk:
                    break
                f.write(chunk)
                bytes_received += len(chunk)
        # sock.sendall("OK".encode())

    def init_backup_as_sender(self, service: str, datas: list[str], type: str, config: dict, transaction_id: str, ssl_context: SSLContext) -> bool:
        if not AgentConfig.instance:
            return False
        client = ssl_context.wrap_socket(socket.socket(socket.AF_INET, socket.SOCK_STREAM), server_hostname=AgentConfig.instance.server["host"])
        client.connect((AgentConfig.instance.server["host"], AgentConfig.instance.backup_relay_port))
        payload = f"{transaction_id}:sender"
        client.sendall(payload.encode())
        all_ok = True
        response = client.recv(1024).decode()
        if response == "OK":
            ready_response = client.recv(1024).decode()
            if ready_response == "START":
                if type == "incremental":
                    summary = self.create_backup_summary_client(service, datas)
                    client.sendall(json.dumps(summary).encode())
                    changes = json.loads(self.buffered_recv(client))
                    for change in changes:
                            file_path = os.path.join(AgentConfig.instance.services_folder, service, change)
                            try:
                                if os.path.exists(file_path) and os.path.isfile(file_path):
                                    self.send_file(client, file_path, file_path.removeprefix(os.path.join(AgentConfig.instance.services_folder, service)).removeprefix("/"))
                                else:
                                    debug(f"File {file_path} does not exist or is not a file.")
                                    all_ok = False
                            except Exception as e:
                                debug(f"Error sending file {file_path}: {e}")
                                all_ok = False
                if type == "full":
                    exclude_paths = []
                    for data in datas:
                        if data.startswith("!"):
                            exclude_paths.append(os.path.abspath(os.path.join(AgentConfig.instance.services_folder, service, data.removeprefix("!"))))
                    for data in [data for data in datas if not data.startswith("!")]:
                        data_path = os.path.join(AgentConfig.instance.services_folder, service, data)
                        if os.path.exists(data_path) and os.path.isdir(data_path):
                            for root, _, files in os.walk(data_path):
                                for file in files:
                                    file_path = os.path.join(root, file)
                                    if any(os.path.abspath(file_path).startswith(exclude) for exclude in exclude_paths):
                                        debug(f"Skipping excluded file {file_path}")
                                        continue
                                    try:
                                        self.send_file(client, file_path, file_path.removeprefix(os.path.join(AgentConfig.instance.services_folder, service)).removeprefix("/"))
                                    except Exception as e:
                                        debug(f"Error sending file {file_path}: {e}")
                                        all_ok = False
                        elif os.path.isfile(data_path):
                            try:
                                self.send_file(client, data_path, data_path.removeprefix(os.path.join(AgentConfig.instance.services_folder, service)).removeprefix("/"))
                            except Exception as e:
                                debug(f"Error sending file {data_path}: {e}")
                                all_ok = False
        client.close() # Close signal announces the end of the transfer
        return all_ok

    def buffered_recv(self, sock: SSLSocket) -> str:
        buffer = b""
        while True:
            data = sock.recv(4096)
            if not data:
                break
            buffer += data
            if b"\n" in buffer:
                break
        return buffer.decode().strip()

    def init_backup_as_storage(self, path: str, service: str, type: str, config: dict, transaction_id: str, ssl_context: SSLContext):
        if not AgentConfig.instance:
            return
        client = ssl_context.wrap_socket(socket.socket(socket.AF_INET, socket.SOCK_STREAM), server_hostname=AgentConfig.instance.server["host"])
        client.connect((AgentConfig.instance.server["host"], AgentConfig.instance.backup_relay_port))
        payload = f"{transaction_id}:receiver"
        client.sendall(payload.encode())
        response = client.recv(1024).decode()
        if response == "OK":
            if type == "incremental":
                client_summary = json.loads(self.buffered_recv(client))
                changes = self.incremental_check_for_changes(path, service, client_summary)
                client.sendall(json.dumps(changes).encode())
                save_path = os.path.join(path, service, "incremental", str(int(time.time())))
                os.makedirs(save_path, exist_ok=True)
                try:
                    while True:
                        self.recv_file(client, save_path)
                except BrokenPipeError:
                    debug("Transfer completed.")
            if type == "full":
                save_path = os.path.join(path, service, "full", str(int(time.time())))
                os.makedirs(save_path, exist_ok=True)
                try:
                    while True:
                        self.recv_file(client, save_path)
                except BrokenPipeError:
                    debug("Transfer completed.")
        client.close() # Close signal announces the end of the transfer
        if type == "incremental":
            self.after_incremential_cleanup(path, service, config["max_size"], config["max_age"])
        if type == "full":
            self.after_full_cleanup(path, service, config["max_size"], config["max_age"])

if __name__ == "__main__":
    backup_manager = BackupManager()
    test_path = "./test_backup"
    datas_path = "./test_backup/data"
    print("Analyzing directory")
    summary_distant = backup_manager.create_backup_summary_client("data", ["./"])
    print("Summary distant:", summary_distant)
    print(backup_manager.incremental_check_for_changes("test_backup/", "service", summary_distant))
