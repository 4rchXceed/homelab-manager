
CREATE SEQUENCE user_variable_id_seq;

CREATE TABLE user_variable (
                id INTEGER NOT NULL DEFAULT nextval('user_variable_id_seq'),
                id_str VARCHAR(100) NOT NULL,
                value VARCHAR,
                CONSTRAINT user_variable_pk PRIMARY KEY (id)
);


ALTER SEQUENCE user_variable_id_seq OWNED BY user_variable.id;

CREATE UNIQUE INDEX user_variable_id_str_unique
 ON user_variable
 ( id_str );

CREATE SEQUENCE server_id_seq;

CREATE TABLE server (
                id INTEGER NOT NULL DEFAULT nextval('server_id_seq'),
                id_str VARCHAR(100) NOT NULL,
                name VARCHAR(100) NOT NULL,
                description VARCHAR(1000) DEFAULT 'No description' NOT NULL,
                ip VARCHAR(15) NOT NULL,
                api_key VARCHAR(36),
                disabled BOOLEAN NOT NULL,
                reverse_api_key VARCHAR(36),
                CONSTRAINT server_pk PRIMARY KEY (id)
);


ALTER SEQUENCE server_id_seq OWNED BY server.id;

CREATE UNIQUE INDEX api_key_server_unique
 ON server
 ( api_key );

CREATE SEQUENCE service_id_seq;

CREATE TABLE service (
                id INTEGER NOT NULL DEFAULT nextval('service_id_seq'),
                sync_server_id INTEGER,
                server_id INTEGER,
                id_str VARCHAR(100) NOT NULL,
                name VARCHAR(100) NOT NULL,
                last_config VARCHAR,
                sync_storage_id_str VARCHAR(100),
                last_sync TIMESTAMP,
                sync_time INTEGER,
                disabled BOOLEAN NOT NULL,
                CONSTRAINT service_pk PRIMARY KEY (id)
);


ALTER SEQUENCE service_id_seq OWNED BY service.id;

CREATE UNIQUE INDEX service_id_str_unique
 ON service
 ( id_str );

CREATE SEQUENCE backup_config_id_seq;

CREATE TABLE backup_config (
                id INTEGER NOT NULL DEFAULT nextval('backup_config_id_seq'),
                service_id INTEGER NOT NULL,
                id_str VARCHAR(100) NOT NULL,
                last TIMESTAMP,
                disabled BOOLEAN NOT NULL,
                CONSTRAINT backup_config_pk PRIMARY KEY (id)
);


ALTER SEQUENCE backup_config_id_seq OWNED BY backup_config.id;

CREATE SEQUENCE user_var_needs_update_id_seq;

CREATE TABLE user_var_needs_update (
                id INTEGER NOT NULL DEFAULT nextval('user_var_needs_update_id_seq'),
                service_id INTEGER NOT NULL,
                user_variable_id INTEGER NOT NULL,
                last_value VARCHAR,
                CONSTRAINT user_var_needs_update_pk PRIMARY KEY (id)
);


ALTER SEQUENCE user_var_needs_update_id_seq OWNED BY user_var_needs_update.id;

CREATE SEQUENCE ip_needs_update_id_seq;

CREATE TABLE ip_needs_update (
                id INTEGER NOT NULL DEFAULT nextval('ip_needs_update_id_seq'),
                service_trigger_id INTEGER NOT NULL,
                service_updated_id INTEGER NOT NULL,
                last_ip VARCHAR(15) NOT NULL,
                CONSTRAINT ip_needs_update_pk PRIMARY KEY (id)
);


ALTER SEQUENCE ip_needs_update_id_seq OWNED BY ip_needs_update.id;

ALTER TABLE user_var_needs_update ADD CONSTRAINT user_variable_user_var_needs_update_fk
FOREIGN KEY (user_variable_id)
REFERENCES user_variable (id)
ON DELETE NO ACTION
ON UPDATE NO ACTION
NOT DEFERRABLE;

ALTER TABLE service ADD CONSTRAINT server_service_fk
FOREIGN KEY (server_id)
REFERENCES server (id)
ON DELETE NO ACTION
ON UPDATE NO ACTION
NOT DEFERRABLE;

ALTER TABLE service ADD CONSTRAINT server_service_fk1
FOREIGN KEY (sync_server_id)
REFERENCES server (id)
ON DELETE NO ACTION
ON UPDATE NO ACTION
NOT DEFERRABLE;

ALTER TABLE ip_needs_update ADD CONSTRAINT service_needs_update_fk
FOREIGN KEY (service_trigger_id)
REFERENCES service (id)
ON DELETE NO ACTION
ON UPDATE NO ACTION
NOT DEFERRABLE;

ALTER TABLE ip_needs_update ADD CONSTRAINT service_needs_update_fk1
FOREIGN KEY (service_updated_id)
REFERENCES service (id)
ON DELETE NO ACTION
ON UPDATE NO ACTION
NOT DEFERRABLE;

ALTER TABLE user_var_needs_update ADD CONSTRAINT service_user_var_needs_update_fk
FOREIGN KEY (service_id)
REFERENCES service (id)
ON DELETE NO ACTION
ON UPDATE NO ACTION
NOT DEFERRABLE;

ALTER TABLE backup_config ADD CONSTRAINT service_backup_config_fk
FOREIGN KEY (service_id)
REFERENCES service (id)
ON DELETE NO ACTION
ON UPDATE NO ACTION
NOT DEFERRABLE;
