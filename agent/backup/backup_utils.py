import json
import socket
from ssl import SSLContext, SSLSocket
import shutil
import time
from config import AgentConfig
import os
from messaging.log import debug
from backup.backup_fsutils import strip_path, all_versions_folders, folder_size, hash_file, hashes_neq, is_path_part_of

def create_backup_summary_client(service_path: str, datas: list[str]) -> dict:
    summary = {}
    for data in datas:
        data_full = os.path.join(service_path, data)
        if os.path.exists(data_full):
            if os.path.isfile(data_full):
                try:
                    summary[strip_path(data_full, service_path).removeprefix(os.sep)] = hash_file(data_full)
                except OSError:
                    continue
            elif os.path.isdir(data_full):
                for root, _, files in os.walk(data_full):
                    for file in files:
                        file_path = os.path.join(root, file)
                        try:
                            summary[strip_path(file_path, service_path).removeprefix(os.sep)] = hash_file(file_path)
                        except OSError:
                            continue
    return summary

def buffered_recv(sock: SSLSocket) -> str:
    buffer = b""
    while True:
        data = sock.recv(4096)
        if not data:
            break
        buffer += data
        if b"\n" in buffer:
            break
    return buffer.decode().strip()


def get_index(base_path: str) -> dict:
    index_path = os.path.join(base_path, "index.json")
    if os.path.exists(index_path):
        with open(index_path, "r") as f:
            index_data = json.load(f)
    else:
        os.makedirs(os.path.dirname(index_path), exist_ok=True)
        index_data = {"deletions": {}}
    return index_data


def set_index(base_path: str, index_data: dict):
    index_path = os.path.join(base_path, "index.json")
    with open(index_path, "w") as f:
        json.dump(index_data, f, indent=4)

def compare_summaries(summary_local: dict, summary_distant: dict) -> tuple[list[str], list[str]]:
    # summary_local: The summary of the local files (backup side)
    # summary_distant: The summary of the distant files (client side)
    changes = []
    deletions = []
    for file_path, hash in summary_distant.items():
        if file_path not in summary_local or hashes_neq(summary_local[file_path], hash):
            changes.append(file_path)
    for file_path in summary_local.keys():
        if file_path not in summary_distant:
            deletions.append(file_path)
    return changes, deletions

def incremental_check_for_changes(backups_folder: str, service: str, summary_distant: dict, deletions: list[str]) -> tuple[list[str], list[str], list[str]]: # Returns a tuple of (changes, deletions, summary)
    backups_folder = backups_folder.removesuffix(os.sep)  # Remove trailing slash if present
    backup_folder = os.path.join(backups_folder, service, "incremental") # 1 folder for full, 1 folder for incremental
    if not os.path.exists(backup_folder): # No backups yet, so we just return all the file changes
        os.makedirs(backup_folder, exist_ok=True)
        compare_result = compare_summaries({}, summary_distant)
        return compare_result[0], compare_result[1], [path for path in summary_distant.keys()]
    base_backup_folder = os.path.join(backup_folder, "base") # The "base" folder contains the full backup of the service, after that, the incremental backups are stored in 1, 2, 3, ... folders
    summary_local = {}
    all_folders = all_versions_folders(backup_folder)

    # We go through the base folder and all the incremental folders to get a list of all the files that are currently stored in the backup, so we can compare them with the files that are on the client side
    # And only update what changed
    if os.path.exists(base_backup_folder) and os.path.isdir(base_backup_folder):
        all_folders.insert(0, "base") # We add the base folder to the list of folders to check

    for version_folder in all_folders:
        version_folder_path = os.path.join(backup_folder, version_folder)
        for root, _, files_in_dir in os.walk(version_folder_path):
            for file in files_in_dir:
                if not any(is_path_part_of(os.path.join(root, file), deletion) for deletion in deletions): # If the file is marked as deleted, we don't add it to the summary
                    file_path = os.path.join(root, file)
                    summary_local[strip_path(file_path, version_folder_path)] = hash_file(file_path)

    if os.getenv("BACKUP_DUMP") is not None:
        if not os.path.exists("logs"):
            os.makedirs("logs", exist_ok=True)
        with open(f"logs/debug_{int(time.time())}.json", "w") as f:
            json.dump({
                "storage_summary": summary_local,
                "client_summary": summary_distant
            }, f, indent=4)
    compare_result = compare_summaries(summary_local, summary_distant)
    return compare_result[0], compare_result[1], [path for path in summary_local.keys()]

def after_incremential_cleanup(backups_folder: str, service: str, max_size: int, max_age: int, deletions: dict):
    backups_folder = backups_folder.removesuffix(os.sep)  # Remove trailing slash if present
    backup_folder = os.path.join(backups_folder, service, "incremental")

    # We always keep at least one backup, so if there is only the "base" folder, we don't delete anything
    if len(all_versions_folders(backup_folder)) == 0:
        return

    last_folder = os.path.join(backup_folder, oldest_folder(backup_folder))
    while do_folder_needs_deletion(last_folder, max_age, max_size, backup_folder):
        oldest_folder_name = oldest_folder(backup_folder)
        oldest_folder_path = os.path.join(backup_folder, oldest_folder_name)

        deletions_for_current_folder = deletions.get(oldest_folder_name, [])


        # Cleanup deleted files from the base folder
        # Always do it *before* copying. Else it would delete files from the incremental backup.
        for root, dirs, files in os.walk(os.path.join(backup_folder, "base"), topdown=False):
            for file in files:
                for deletion in deletions_for_current_folder:
                    file_full_path = os.path.join(root, file)
                    full_deletion_path = os.path.join(backup_folder, "base", deletion)
                    if is_path_part_of(file_full_path, full_deletion_path):
                        os.unlink(os.path.join(root, file)) # Delete the file to the base folder if it was marked for deletion in the incremental backup
                        break
                # We DON'T want to delete folders, even if a deletion is marked for them!
                # Since deletions are *always* for files, not folders. If a folder is marked a deletion, it's simply a file that was deleted and a folder with the same name was created.

        if os.path.exists(oldest_folder_path) and os.path.isdir(oldest_folder_path):
            # First: we copy the files that are going to be deleted from the incremental backup to the base folder, so we don't lose them
            for root, _, files in os.walk(oldest_folder_path, topdown=False):
                for name in files:
                    full_path = os.path.join(root, name)
                    full_path_base = os.path.join(backup_folder, "base", os.path.relpath(full_path, oldest_folder_path))

                    if os.path.isdir(full_path_base):
                        shutil.rmtree(full_path_base)

                    # A file can become a folder in the backups, we need to handle that case.
                    parts = full_path_base.split(os.sep)
                    for i in range(1, len(parts)):
                        dir_path = os.sep.join(parts[:i])
                        if os.path.isfile(dir_path):
                            os.remove(dir_path)
                        # And also ensure the parent folder exists
                        if not os.path.exists(dir_path):
                            os.makedirs(dir_path, exist_ok=True)
                    # Then copy the file WITH THE METADATA (so the hash still works)
                    # If you OS does not support metadata (mtime, to be precise), you won't be able to use incremental backups CORRECTLY
                    shutil.copy2(full_path, full_path_base)

            # Finally: we delete the oldest incremental backup folder
            shutil.rmtree(oldest_folder_path)

        all_folders = all_versions_folders(backup_folder)
        if len(all_folders) == 0:
            break
        last_folder = os.path.join(backup_folder, all_folders[0])

def oldest_folder(backup_folder: str) -> str:
    all_folders = all_versions_folders(backup_folder)
    if len(all_folders) == 0:
        raise ValueError(f"No backup folders found in {backup_folder}")
    return all_folders[0]

def do_folder_needs_deletion(folder: str, max_age: int, max_size: int, folder_container: str) -> bool:
    return os.path.getctime(folder) < time.time() - max_age or folder_size(folder_container) > max_size # TODO: folder_size can take a verrry long time

def after_full_cleanup(backups_folder: str, service: str, max_size: int, max_age: int):
    backups_folder = backups_folder.removesuffix(os.sep)  # Remove trailing slash if present
    backup_folder = os.path.join(backups_folder, service, "full")

    if len(all_versions_folders(backup_folder)) == 0:  # ALWAYS keep at least one backup
        return
    last = os.path.join(backup_folder, oldest_folder(backup_folder))

    while do_folder_needs_deletion(last, max_age, max_size, backup_folder): # TODO: folder_size can take a verrry long time, folder_size(backup_folder, *last*) since we always want to have at least one backup
        oldest_folder_name = oldest_folder(backup_folder)
        oldest_folder_path = os.path.join(backup_folder, oldest_folder_name)

        if os.path.exists(oldest_folder_path) and os.path.isdir(oldest_folder_path):
            try:
                shutil.rmtree(os.path.join(backup_folder, "base"))
            except OSError as e:
                debug(f"Error deleting folder {oldest_folder_path}: {e}")
            shutil.move(oldest_folder_path, os.path.join(backup_folder, "base"))  # Move the oldest folder to the base folder
        last = os.path.join(backup_folder, oldest_folder(backup_folder))


def send_file(sock: SSLSocket, file_path: str, file_path_to_send: str):
    fsize = os.path.getsize(file_path)
    mtime = os.path.getmtime(file_path)
    permissions = oct(os.stat(file_path).st_mode)[-3:]
    sock.sendall(f"{fsize}?{file_path_to_send}?{mtime}?{permissions}\n".encode())
    send_file_raw(file_path, sock)


def send_file_raw(file_path: str, sock: SSLSocket):
    with open(file_path, "rb") as f:
        while True:
            bytes_read = f.read(4096)
            if not bytes_read:
                break
            sock.sendall(bytes_read)

def recv_file(sock: SSLSocket, save_path: str) -> bool:
    file_infos = buffered_recv(sock)
    if file_infos == "END":
        return True
    if not file_infos:
        return False
    if not file_infos.count("?") == 3:
        debug(f"Invalid file info received: {file_infos}. Protocol violation!")
        return True
    fsize_str, file_path, mod_date, permissions = file_infos.split("?", 3)
    fsize = int(fsize_str)
    full_save_path = os.path.join(save_path, file_path)
    return recv_file_raw(full_save_path, sock, fsize, mod_date, permissions)

def recv_file_raw(full_save_path: str, sock: SSLSocket, fsize: int, mod_date: str, permissions: str) -> bool:
    try:
        bytes_received = 0
        os.makedirs(os.path.dirname(full_save_path), exist_ok=True)
        with open(full_save_path, "wb") as f:
            while bytes_received < fsize:
                chunk = sock.recv(min(4096, fsize - bytes_received))
                if not chunk:
                    break
                f.write(chunk)
                bytes_received += len(chunk)
    except Exception as e:
        debug(f"Error while writing to {full_save_path}: {e}")
        # If we can't write to the file, we still need to read the incoming data to avoid breaking the protocol
        while bytes_received < fsize:
            chunk = sock.recv(min(4096, fsize - bytes_received))
            if not chunk:
                break
            bytes_received += len(chunk)
    try:
        os.utime(full_save_path, (float(mod_date), float(mod_date)))
        os.chmod(full_save_path, int(permissions, 8))
    except Exception as e:
        debug(f"Error setting modification time & permissions for {full_save_path}: {e}")
    return False

def restore_files(files: list, base_path: str, client: SSLSocket, is_incremental: bool = False) -> str:
    messages = ""
    for file_info in files:
        if is_incremental:
            file_path = os.path.join(base_path, file_info["folder"], file_info["path"])
        else:
            file_path = os.path.join(base_path, file_info["path"])
        if os.path.exists(file_path) and os.path.isfile(file_path):
            send_file(client, file_path, file_info["path"])
        else:
            msg = f"File {file_path} does not exist or is not a file."
            debug(msg)
            messages += msg + "\n"
    return messages

def connect_as(transaction_id: str, role: str, ssl_context: SSLContext) -> SSLSocket | None:
    if not AgentConfig.instance:
        return
    client = ssl_context.wrap_socket(socket.socket(socket.AF_INET, socket.SOCK_STREAM), server_hostname=AgentConfig.instance.server["host"])
    client.connect((AgentConfig.instance.server["host"], AgentConfig.instance.backup_relay_port))
    payload = f"temp:{transaction_id}:{role}"
    client.sendall(payload.encode())
    response = client.recv(1024).decode()
    if response == "OK":
        return client
    else:
        client.close()
        return None

def get_restore_needs(files_to_check: list, restore_path: str) -> list:
    files_changed = []
    for file_info in files_to_check:
        file_path = os.path.join(restore_path, file_info["path"])
        if os.path.exists(file_path) and os.path.isfile(file_path):
            try:
                modification_time = os.path.getmtime(file_path)
                hash = str(modification_time)
                if len(hash) != len(file_info["hash"]):
                    smallest = min(len(hash), len(file_info["hash"]))
                    if hash[0:smallest] != file_info["hash"][0:smallest]:
                        files_changed.append(file_info)
                else:
                    if hash != file_info["hash"]:
                        files_changed.append(file_info)
            except OSError:
                files_changed.append(file_info)
        else:
            files_changed.append(file_info)
    return files_changed

def is_deleted(path: str, undeleted_files: list) -> bool:
    i = 0
    immune = False
    while i < len(undeleted_files) and not immune: # So we don't delete files that haven't changed.
        if path == undeleted_files[i]["path"]:
            immune = True
        i += 1
    return not immune
