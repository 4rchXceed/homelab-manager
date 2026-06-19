from typing import Optional

from sqlalchemy import Boolean, ForeignKeyConstraint, Index, Integer, PrimaryKeyConstraint, String, text
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column, relationship

class Base(DeclarativeBase):
    pass


class Server(Base):
    __tablename__ = 'server'
    __table_args__ = (
        PrimaryKeyConstraint('id', name='server_pk'),
        Index('api_key_server_unique', 'api_key', unique=True)
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    id_str: Mapped[str] = mapped_column(String(100), nullable=False)
    name: Mapped[str] = mapped_column(String(100), nullable=False)
    description: Mapped[str] = mapped_column(String(1000), nullable=False, server_default=text("'No description'::character varying"))
    ip: Mapped[str] = mapped_column(String(15), nullable=False)
    api_key: Mapped[Optional[str]] = mapped_column(String(36))

    service: Mapped[list['Service']] = relationship('Service', back_populates='server')


class UserVariable(Base):
    __tablename__ = 'user_variable'
    __table_args__ = (
        PrimaryKeyConstraint('id', name='user_variable_pk'),
        Index('user_variable_id_str_unique', 'id_str', unique=True)
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    id_str: Mapped[str] = mapped_column(String(100), nullable=False)
    value: Mapped[Optional[str]] = mapped_column(String)


class Service(Base):
    __tablename__ = 'service'
    __table_args__ = (
        ForeignKeyConstraint(['server_id'], ['server.id'], name='server_service_fk'),
        PrimaryKeyConstraint('id', name='service_pk'),
        Index('service_id_str_unique', 'id_str', unique=True)
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    id_str: Mapped[str] = mapped_column(String(100), nullable=False)
    name: Mapped[str] = mapped_column(String(100), nullable=False)
    disabled: Mapped[bool] = mapped_column(Boolean, nullable=False)
    server_id: Mapped[Optional[int]] = mapped_column(Integer)
    last_config: Mapped[Optional[str]] = mapped_column(String)

    server: Mapped[Optional['Server']] = relationship('Server', back_populates='service')
    needs_update_service_trigger: Mapped[list['NeedsUpdate']] = relationship('NeedsUpdate', foreign_keys='[NeedsUpdate.service_trigger_id]', back_populates='service_trigger')
    needs_update_service_updated: Mapped[list['NeedsUpdate']] = relationship('NeedsUpdate', foreign_keys='[NeedsUpdate.service_updated_id]', back_populates='service_updated')


class NeedsUpdate(Base):
    __tablename__ = 'needs_update'
    __table_args__ = (
        ForeignKeyConstraint(['service_trigger_id'], ['service.id'], name='service_needs_update_fk'),
        ForeignKeyConstraint(['service_updated_id'], ['service.id'], name='service_needs_update_fk1'),
        PrimaryKeyConstraint('id', name='needs_update_pk')
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    service_trigger_id: Mapped[int] = mapped_column(Integer, nullable=False)
    service_updated_id: Mapped[int] = mapped_column(Integer, nullable=False)
    last_ip: Mapped[str] = mapped_column(String(15), nullable=False)

    service_trigger: Mapped['Service'] = relationship('Service', foreign_keys=[service_trigger_id], back_populates='needs_update_service_trigger')
    service_updated: Mapped['Service'] = relationship('Service', foreign_keys=[service_updated_id], back_populates='needs_update_service_updated')
