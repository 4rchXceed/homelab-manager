import threading

from sqlalchemy import create_engine
from sqlalchemy.orm import Session, sessionmaker


class DatabaseEngine:
    """PostgreSQL database engine"""

    def __init__(self, options: dict) -> None:
        self.host = options.get("host")
        self.port = options.get("port")
        self.username = options.get("username")
        self.password = options.get("password")
        self.database = options.get("database")
        self.engine = create_engine(
            f"postgresql://{self.username}:{self.password}@{self.host}:{self.port}/{self.database}"
        )
        self.SessionLocal = sessionmaker(self.engine, autocommit=False, autoflush=False)
        self.thread_data = threading.local()

    @property
    def session(self) -> Session:
        if not hasattr(self.thread_data, "session"):
            self.thread_data.session = self.SessionLocal()
        return self.thread_data.session
