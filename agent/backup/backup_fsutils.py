import os

def hash_file(file_path: str) -> str:
    try:
        modification_time = os.path.getmtime(file_path)
        return str(modification_time)
    except OSError:
        return ""

def hashes_neq(hash1: str, hash2: str) -> bool:
    if len(hash1) != len(hash2):
        smallest = min(len(hash1), len(hash2))
        return hash1[0:smallest] != hash2[0:smallest]
    return hash1 != hash2


def is_path_part_of(path: str, parent_path: str) -> bool:
    # Check if path is part of parent_path
    path = os.path.abspath(path)
    parent_path = os.path.abspath(parent_path)
    return os.path.commonpath([path]) == os.path.commonpath([path, parent_path])

def strip_path(path: str, parent_path: str) -> str:
    # Strip parent_path from path
    path = os.path.abspath(path)
    parent_path = os.path.abspath(parent_path)
    if path.startswith(parent_path):
        return path.removeprefix(parent_path).strip(os.sep)
    return path

def all_versions_folders(backup_folder: str) -> list[str]: # Returns folder in order from oldest to newest, except "base"
    all_versions_folders = [f for f in os.listdir(backup_folder) if os.path.isdir(os.path.join(backup_folder, f)) and f.isdigit()]
    all_versions_folders.sort(key=lambda x: int(x))
    return all_versions_folders

def folder_size(path: str, except_folder: str) -> float:
    total_size = 0
    for folder in os.listdir(path):
        if folder != except_folder:
            for dirpath, _, filenames in os.walk(os.path.join(path, folder)):
                for f in filenames:
                    fp = os.path.join(dirpath, f)
                    if os.path.exists(fp):
                        total_size += os.path.getsize(fp) # Size in B
    return total_size
