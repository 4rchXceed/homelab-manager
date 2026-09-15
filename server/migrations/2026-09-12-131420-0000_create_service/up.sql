CREATE TABLE service (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    sync_server_id INTEGER REFERENCES server(id),
    server_id INTEGER REFERENCES server(id),
    id_str VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    last_config TEXT,
    sync_storage_id_str VARCHAR(100),
    last_sync TIMESTAMP,
    sync_time INTEGER NOT NULL,
    disabled BOOLEAN NOT NULL DEFAULT FALSE
);
