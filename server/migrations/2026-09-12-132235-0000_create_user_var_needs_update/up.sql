CREATE TABLE user_var_needs_update (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    service_id INTEGER NOT NULL REFERENCES service(id) ON DELETE CASCADE,
    user_variable_id INTEGER NOT NULL REFERENCES user_variable(id) ON DELETE CASCADE,
    last_value TEXT NOT NULL
);
