from typing import Optional

from sqlalchemy import (
    ForeignKeyConstraint,
    Index,
    Integer,
    PrimaryKeyConstraint,
    String,
    text,
)
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column, relationship


class Base(DeclarativeBase):
    pass


class Server(Base):
    __tablename__ = "server"
    __table_args__ = (PrimaryKeyConstraint("id", name="server_pk"),)

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    name: Mapped[str] = mapped_column(String(100), nullable=False)
    description: Mapped[str] = mapped_column(
        String(1000),
        nullable=False,
        server_default=text("'No description'::character varying"),
    )
    ip: Mapped[str] = mapped_column(String(15), nullable=False)

    service: Mapped[list["Service"]] = relationship("Service", back_populates="server")


class UserVariable(Base):
    __tablename__ = "user_variable"
    __table_args__ = (
        PrimaryKeyConstraint("id", name="user_variable_pk"),
        Index("user_variable_id_str_unique", "id_str", unique=True),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    id_str: Mapped[str] = mapped_column(String(100), nullable=False)
    value: Mapped[Optional[str]] = mapped_column(String)


class Service(Base):
    __tablename__ = "service"
    __table_args__ = (
        ForeignKeyConstraint(["server_id"], ["server.id"], name="server_service_fk"),
        PrimaryKeyConstraint("id", name="service_pk"),
        Index("service_id_str_unique", "id_str", unique=True),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    id_str: Mapped[str] = mapped_column(String(100), nullable=False)
    name: Mapped[str] = mapped_column(String(100), nullable=False)
    server_id: Mapped[Optional[int]] = mapped_column(Integer)
    last_config: Mapped[Optional[str]] = mapped_column(String)

    server: Mapped[Optional["Server"]] = relationship(
        "Server", back_populates="service"
    )
