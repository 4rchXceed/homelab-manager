import json
import socket
from ssl import SSLContext, SSLSocket
import shutil
import time
from config import AgentConfig
import os
from messaging.log import debug
from backup.backup_fsutils import strip_path, all_versions_folders, folder_size, hash_file, hashes_neq, is_path_part_of
from backup.backup_utils import after_full_cleanup, after_incremential_cleanup, all_versions_folders, buffered_recv, connect_as, get_index, incremental_check_for_changes, recv_file, restore_files, send_file, set_index, create_backup_summary_client, get_restore_needs, is_deleted, send_file_raw, recv_file_raw

class FileQueueEntry:
    def __init__(self, service: str, file_name: str, file_path: str):
        self.service = service
        self.file_name = file_name
        self.file_path = file_path

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

    def init_backup_as_service(self, service: str, datas: list[str], type: str, transaction_id: str, ssl_context: SSLContext) -> bool:
        if not AgentConfig.instance:
            return False
        services_folder = AgentConfig.instance.services_folder
        backup_from_path = os.path.join(services_folder, service)
        if not os.path.exists(backup_from_path):
            debug(f"Service {service} does not exist in {services_folder}.")
            return False
        all_ok = True
        client = connect_as(transaction_id, "service", ssl_context)
        if client:
            ready_response = client.recv(1024).decode()
            if ready_response == "START":
                if type == "incremental":
                    debug(f"Starting incremental backup for service {service}: generating summary and sending to client...")
                    summary = create_backup_summary_client(os.path.join(services_folder, service), datas)
                    client.sendall(f"{json.dumps(summary)}\n".encode())
                    debug(f"Summary sent to client: waiting for changes...")
                    changes = json.loads(buffered_recv(client))
                    debug(f"Changes received: starting to send files...")
                    for change in changes:
                        file_path = os.path.join(backup_from_path, change)
                        try:
                            if os.path.exists(file_path) and os.path.isfile(file_path):
                                send_file(client, file_path, change.removeprefix(os.sep))
                            else:
                                debug(f"File {file_path} does not exist or is not a file.")
                                all_ok = False
                        except Exception as e:
                            debug(f"Error sending file {file_path}: {e}")
                            all_ok = False
                    if all_ok:
                        debug(f"All files sent successfully")
                    else:
                        debug(f"Some files failed to send")
                if type == "full":
                    for data in [data for data in datas if not data.startswith("!")]:
                        data_path = os.path.join(backup_from_path, data)

                        if os.path.exists(data_path) and os.path.isdir(data_path):
                            for root, _, files in os.walk(data_path):
                                for file in files:
                                    file_path = os.path.join(root, file)
                                    try:
                                        send_file(client, file_path, strip_path(file_path, backup_from_path).removeprefix(os.sep))
                                    except Exception as e:
                                        debug(f"Error sending file {file_path}: {e}")
                                        all_ok = False
                        elif os.path.isfile(data_path):
                            try:
                                send_file(client, data_path, strip_path(data_path, backup_from_path).removeprefix(os.sep))
                            except Exception as e:
                                debug(f"Error sending file {data_path}: {e}")
                                all_ok = False

            client.sendall(b"END\n")
            client.close() # Close signal announces the end of the transfer
        return all_ok

    def init_backup_as_storage(self, path: str, service: str, type: str, config: dict, transaction_id: str, ssl_context: SSLContext):
        index_data = {}
        stopped = False
        client = connect_as(transaction_id, "backup", ssl_context)
        if client:
            if type == "incremental":
                base_path = os.path.join(path, service, "incremental")
                folder_id = str(int(time.time()))
                save_path = os.path.join(base_path, folder_id)

                client_summary = json.loads(buffered_recv(client))

                index_data = get_index(base_path)
                all_deletions = index_data.get("deletions", {})
                last_deletion_entry_id = max([int(id) for id in all_deletions.keys()], default=None)
                if last_deletion_entry_id is not None:
                    last_deletion_entry = all_deletions.get(str(last_deletion_entry_id), [])
                else:
                    last_deletion_entry = []
                changes = incremental_check_for_changes(path, service, client_summary, last_deletion_entry)
                deletions = changes[1]

                client.sendall(f"{json.dumps(changes[0])}\n".encode())

                if not os.path.exists(os.path.join(base_path, "base")):
                    save_path = os.path.join(base_path, "base")
                else:
                    index_data["deletions"][folder_id] = deletions
                set_index(base_path, index_data)

                os.makedirs(save_path, exist_ok=True)

                while not stopped:
                    stopped = recv_file(client, save_path)
            if type == "full":
                if not os.path.exists(os.path.join(path, service, "full")):
                    os.makedirs(os.path.join(path, service, "full"), exist_ok=True)
                if not os.path.exists(os.path.join(path, service, "full", "base")):
                    os.makedirs(os.path.join(path, service, "full", "base"), exist_ok=True)
                    folder_id = "base"
                else:
                    folder_id = str(int(time.time()))
                save_path = os.path.join(path, service, "full", folder_id)

                os.makedirs(save_path, exist_ok=True)
                while not stopped:
                    stopped = recv_file(client, save_path)

            client.close() # Close signal announces the end of the transfer
            if type == "incremental":
                after_incremential_cleanup(path, service, config["max_size"], config["max_age"], index_data.get("deletions", {}))
            if type == "full":
                after_full_cleanup(path, service, config["max_size"], config["max_age"])


    def init_restore_as_client(self, service: str, transaction_id: str, datas: list[str], ssl_context: SSLContext, restore_path: str) -> bool:
        if not AgentConfig.instance:
            return False
        ok = True
        client = connect_as(transaction_id, "backup", ssl_context)
        if client:
            if not os.path.exists(restore_path):
                return False
            files_to_check = json.loads(buffered_recv(client))
            files_changed = get_restore_needs(files_to_check, restore_path)

            for file_folder in datas:
                # TODO: more configurable restore path, for now we will restore to a temporary location and then move the files to the original location
                target = os.path.join("/tmp", f"restore_{service}_{int(time.time())}")
                if not os.path.exists(target):
                    os.makedirs(target, exist_ok=True)
                data_dir = os.path.join(restore_path, file_folder.removeprefix("./"))
                if not os.path.isfile(data_dir):
                    for dir, _, files in os.walk(data_dir):
                        for file in files:
                            file_path = os.path.join(dir, file)
                            if is_deleted(strip_path(file_path, restore_path).removeprefix("/").removeprefix("./"), files_to_check): # We want to remove files that are not deleted in the backup
                                file_target = os.path.join(target, file_folder.removeprefix("./"), file_path)
                                if not os.path.exists(os.path.dirname(file_target)):
                                    os.makedirs(os.path.dirname(file_target), exist_ok=True)
                                shutil.move(file_path, file_target)
                else:
                    if is_deleted(file_folder.removeprefix("./").removesuffix("/"), files_to_check):
                        target = os.path.join(target, file_folder.removeprefix("./"))
                        if not os.path.exists(os.path.dirname(target)):
                            os.makedirs(os.path.dirname(target), exist_ok=True)
                        shutil.move(data_dir, target)
            if os.getenv("BACKUP_DUMP") is not None:
                if not os.path.exists("logs"):
                    os.makedirs("logs", exist_ok=True)
                with open(f"logs/debug_restore_{int(time.time())}.json", "w") as f:
                    json.dump({
                        "files_total": files_to_check,
                        "restoring_files": files_changed
                    }, f, indent=4)

            client.sendall(f"{json.dumps(files_changed)}\n".encode())
            while ok:
                ok = not recv_file(client, restore_path)
            client.close()
        return True

    def init_full_sync_as_service(self, service: str, transaction_id: str, datas: list[str], ssl_context: SSLContext) -> bool:
        # Sync storage runs the method: init_restore_as_client, since it shares the same logic as restoring a backup, but instead of restoring a backup it syncs the files from the service to the storage.
        if not AgentConfig.instance:
            return False
        all_ok = True
        client = connect_as(transaction_id, "service", ssl_context)
        if client:
            ready_response = client.recv(1024).decode()
            if ready_response == "START":
                services_folder = AgentConfig.instance.services_folder
                sync_path = os.path.join(services_folder, service)
                files_to_check: list[dict[str,str]] = [] # List of dicts with keys "path", "hash"
                # Send only the datas
                for data in datas:
                    data_clean = data.removeprefix("./").removesuffix("/")
                    data_path = os.path.join(sync_path, data_clean)
                    if os.path.exists(data_path) and os.path.isdir(data_path):
                        for root, _, files in os.walk(data_path):
                            for file in files:
                                file_path = os.path.join(root, file)
                                relative_path = strip_path(file_path, sync_path).removeprefix(os.sep)
                                try:
                                    files_to_check.append({"path": relative_path, "hash": hash_file(file_path)})
                                except OSError:
                                    continue
                    elif os.path.isfile(data_path):
                        relative_path = strip_path(data_path, sync_path).removeprefix(os.sep)
                        try:
                            files_to_check.append({"path": relative_path, "hash": hash_file(data_path)})
                        except OSError:
                            continue
                client.sendall(f"{json.dumps(files_to_check)}\n".encode())
                r = buffered_recv(client)
                files_to_restore = json.loads(r)

                restore_files(files_to_restore, sync_path, client)


            client.sendall(b"END\n")
            client.close() # Close signal announces the end of the transfer
        return all_ok

    def init_restore_as_storage(self, path: str, service: str, type: str, config: dict, transaction_id: str, ssl_context: SSLContext, restore_path: str|None) -> tuple[bool, str]:
        if not AgentConfig.instance:
            return False, "Not supposed to happen, AgentConfig.instance is None."
        # If the restore_path is not provided, we will restore the latest backup for the service and type
        if not restore_path:
            all_versions_path = os.path.join(path, service, type)
            if not os.path.exists(all_versions_path):
                return False, f"No backups found for service {service} in {all_versions_path}."
            all_versions = all_versions_folders(all_versions_path)
            all_versions.insert(0, "base")  # We add the base folder to the list of folders to check
            if len(all_versions) == 0:
                return False, f"No backups found for service {service} in {all_versions_path}."
            restore_path = all_versions[-1] # Restore the latest backup if no specific restore path

        full_path = os.path.join(path, service, type, restore_path)
        if not os.path.exists(full_path):
            return False, f"Backup with id {restore_path} not found for service {service} in {full_path}."

        messages = ""
        client = connect_as(transaction_id, "service", ssl_context) # "services" is the first connection, "backup" is the second connection
        if client:
            start_msg = client.recv(1024).decode()
            if start_msg == "START":
                if type == "incremental":
                    base_path = os.path.join(path, service, "incremental")

                    # Get all folders from the base to the restore_path
                    if restore_path == "base":
                        current_folders = ["base"]
                    else:
                        all_folders = all_versions_folders(base_path)
                        all_folders.insert(0, "base")  # We add the base folder to the list of folders to check
                        current_index = all_folders.index(restore_path)
                        current_folders = all_folders[:current_index + 1]
                    files_to_check: list[dict[str,str]] = [] # List of dicts with keys "path", "hash"
                    for folder in current_folders:
                        folder_path = os.path.join(base_path, folder)
                        for root, _, files in os.walk(folder_path):
                            for file in files:
                                file_path = os.path.join(root, file)
                                relative_path = strip_path(file_path, folder_path).removeprefix(os.sep)
                                try:
                                    files_to_check.append({"path": relative_path, "hash": hash_file(file_path), "folder": folder})
                                except OSError:
                                    continue
                    client.sendall(f"{json.dumps(files_to_check)}\n".encode())
                    files_to_restore = json.loads(buffered_recv(client))

                    messages += restore_files(files_to_restore, base_path, client, is_incremental=True)

                if type == "full":
                    files_to_check: list[dict[str,str]] = [] # List of dicts with keys "path", "hash"
                    for root, _, files in os.walk(full_path):
                        for file in files:
                            file_path = os.path.join(root, file)
                            relative_path = strip_path(file_path, full_path).removeprefix(os.sep)
                            try:
                                files_to_check.append({"path": relative_path, "hash": hash_file(file_path)})
                            except OSError:
                                continue

                    client.sendall(f"{json.dumps(files_to_check)}\n".encode())
                    files_to_restore = json.loads(buffered_recv(client))

                    messages += restore_files(files_to_restore, full_path, client)
            client.sendall("END\n".encode())
            client.close()
        return True, messages

    def sync_file_to_storage(self, service: str, file_path: str, file_name: str, transfer_socket: SSLSocket) -> None:
        file_size = os.path.getsize(file_path)
        modification_time = os.path.getmtime(file_path)
        payload = f"{service}?{file_size}?{file_name}?{modification_time}\n"
        transfer_socket.sendall(payload.encode())
        send_file_raw(file_path, transfer_socket)

    def handle_as_storage(self, sock: SSLSocket):
        stop = False
        while not stop:
            datas = buffered_recv(sock)
            if datas.count("?") == 4:
                parts = datas.split("?")
                service = parts[0]
                path = parts[1]
                debug(f"RECEIVING FILE FOR SERVICE {service} IN PATH {path}")
                try:
                    file_size = int(parts[2])
                except:
                    debug(f"PROTOCOL ERROR: {file_size} IS NOT AN INT!")
                    continue
                file_name = parts[3]
                modification_time = parts[4]
                parent_path = os.path.join(path, service, "sync")
                full_path = os.path.join(parent_path, file_name)
                recv_file_raw(full_path, sock, file_size, modification_time)
            else:
                debug(f"PROTOCOL ERROR: INVALID HEADER: {datas}")
                if not datas:
                    stop = True

    def list_backups(self, path: str, with_size: bool) -> dict:
        backups = {}
        services = os.listdir(path)
        for service in services:
            service_path = os.path.join(path, service)
            if os.path.isdir(service_path):
                backups[service] = {}
                for backup_type in ["full", "incremental"]:
                    type_path = os.path.join(service_path, backup_type)
                    if os.path.exists(type_path):
                        all = all_versions_folders(type_path)
                        all.insert(0, "base")
                        if with_size:
                            total = 0
                            backups[service][backup_type] = []
                            for folder in all:
                                folder_path = os.path.join(type_path, folder)
                                size = folder_size(folder_path)
                                total += size
                                backups[service][backup_type].append({"folder": folder, "size": size})
                            backups[service][backup_type].append({"total_size": total})
                        else:
                            backups[service][backup_type] = []
                            for folder in all:
                                backups[service][backup_type].append({"folder": folder})
        return backups
