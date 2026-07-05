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
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS local_settings (
                key TEXT PRIMARY KEY,
                value TEXT
            );
        """)
        conn.commit()
        cursor.close()
        conn.close()

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

    def get_local_setting(self, key: str) -> str | None:
        with self.conn_lock:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            cursor.execute("SELECT value FROM local_settings WHERE key = ?", (key,))
        res = cursor.fetchone()
        return res[0] if res else None

    def set_local_setting(self, key: str, value: str) -> None:
        with self.conn_lock:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            cursor.execute("INSERT OR REPLACE INTO local_settings (key, value) VALUES (?, ?)", (key, value))
            conn.commit()
