import hashlib
import os
import sqlite3
from threading import Lock


class Database:
    def __init__(self, db_path: str):
        self.conn_lock = Lock()
        self.db_path = db_path
        self.ensure_tables()

    def ensure_tables(self):
        conn = sqlite3.connect(self.db_path)

        cursor = conn.cursor()
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS services (
                id INTEGER PRIMARY KEY,
                id_str TEXT,
                folder_hash TEXT
            );
        """)
        conn.commit()

    def get_service(self, id_str: str, do_not_create=False):
        with self.conn_lock:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            cursor.execute("SELECT * FROM services WHERE id_str = ?", (id_str,))
        res = cursor.fetchone()
        if not res and not do_not_create:
            self.ensure_service(id_str)
            return self.get_service(id_str)
        return res

    def ensure_service(self, id_str: str):
        if self.get_service(id_str, do_not_create=True):
            return
        with self.conn_lock:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            cursor.execute("INSERT INTO services (id_str) VALUES (?)", (id_str,))
            conn.commit()
        return cursor.lastrowid

    # Removed
    # def check_folder_change(
    #     self, service_folder: str, service_id: str, except_folders_or_files: list[str]
    # ) -> bool:
    #     files_found = []
    #     for root, dirs, files in os.walk(os.path.join(service_folder, service_id)):
    #         for file in files:
    #             files_found.append(os.path.join(root, file))
    #     files_safe = []
    #     for file in files_found:
    #         safe = True
    #         for except_folder_or_file in except_folders_or_files:
    #             if file.startswith(
    #                 os.path.join(
    #                     service_folder,
    #                     service_id,
    #                     except_folder_or_file.removeprefix("./"),
    #                 )
    #             ):
    #                 safe = False
    #                 break
    #         if safe:
    #             files_safe.append(file)
    #     files_safe.sort()
    #     hashes = []
    #     for file in files_safe:
    #         with open(file, "rb") as f:
    #             hashes.append(hashlib.sha256(f.read()).hexdigest())
    #     folder_hash = hashlib.sha256(str(hashes).encode()).hexdigest()
    #     old_hash = self.get_service(service_id)[2]
    #     with self.conn_lock:
    #         conn = sqlite3.connect(self.db_path)
    #         cursor = conn.cursor()
    #         cursor.execute(
    #             "UPDATE services SET folder_hash = ? WHERE id_str = ?",
    #             (folder_hash, service_id),
    #         )
    #         conn.commit()
    #     print(folder_hash, old_hash)
    #     return folder_hash != old_hash
