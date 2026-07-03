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
    disabled: Mapped[bool] = mapped_column(Boolean, nullable=False)
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

    user_var_needs_update: Mapped[list['UserVarNeedsUpdate']] = relationship('UserVarNeedsUpdate', back_populates='user_variable')


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
    ip_needs_update_service_trigger: Mapped[list['IpNeedsUpdate']] = relationship('IpNeedsUpdate', foreign_keys='[IpNeedsUpdate.service_trigger_id]', back_populates='service_trigger')
    ip_needs_update_service_updated: Mapped[list['IpNeedsUpdate']] = relationship('IpNeedsUpdate', foreign_keys='[IpNeedsUpdate.service_updated_id]', back_populates='service_updated')
    user_var_needs_update: Mapped[list['UserVarNeedsUpdate']] = relationship('UserVarNeedsUpdate', back_populates='service')


class IpNeedsUpdate(Base):
    __tablename__ = 'ip_needs_update'
    __table_args__ = (
        ForeignKeyConstraint(['service_trigger_id'], ['service.id'], name='service_needs_update_fk'),
        ForeignKeyConstraint(['service_updated_id'], ['service.id'], name='service_needs_update_fk1'),
        PrimaryKeyConstraint('id', name='ip_needs_update_pk')
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    service_trigger_id: Mapped[int] = mapped_column(Integer, nullable=False)
    service_updated_id: Mapped[int] = mapped_column(Integer, nullable=False)
    last_ip: Mapped[str] = mapped_column(String(15), nullable=False)

    service_trigger: Mapped['Service'] = relationship('Service', foreign_keys=[service_trigger_id], back_populates='ip_needs_update_service_trigger')
    service_updated: Mapped['Service'] = relationship('Service', foreign_keys=[service_updated_id], back_populates='ip_needs_update_service_updated')


class UserVarNeedsUpdate(Base):
    __tablename__ = 'user_var_needs_update'
    __table_args__ = (
        ForeignKeyConstraint(['service_id'], ['service.id'], name='service_user_var_needs_update_fk'),
        ForeignKeyConstraint(['user_variable_id'], ['user_variable.id'], name='user_variable_user_var_needs_update_fk'),
        PrimaryKeyConstraint('id', name='user_var_needs_update_pk')
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    service_id: Mapped[int] = mapped_column(Integer, nullable=False)
    user_variable_id: Mapped[int] = mapped_column(Integer, nullable=False)
    last_value: Mapped[Optional[str]] = mapped_column(String)

    service: Mapped['Service'] = relationship('Service', back_populates='user_var_needs_update')
    user_variable: Mapped['UserVariable'] = relationship('UserVariable', back_populates='user_var_needs_update')
