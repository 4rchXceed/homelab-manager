use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::server)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Server {
    pub id: i32,
    pub id_str: String,
    pub description: Option<String>,
    pub ip: String,
    pub api_key: Option<String>,
    pub disabled: bool,
    pub reverse_api_key: String,
}

#[derive(Queryable, Selectable, Associations)]
#[diesel(table_name = crate::schema::service)]
#[diesel(belongs_to(Server))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Service {
    pub id: i32,
    pub sync_server_id: Option<i32>,
    pub server_id: Option<i32>,
    pub id_str: String,
    pub name: String,
    pub last_config: Option<String>,
    pub sync_storage_id_str: Option<String>,
    pub last_sync: Option<chrono::NaiveDateTime>,
    pub sync_time: i32,
    pub disabled: bool,
}

#[derive(Queryable, Selectable, Associations)]
#[diesel(table_name = crate::schema::backup_config)]
#[diesel(belongs_to(Service))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct BackupConfig {
    pub id: i32,
    pub service_id: i32,
    pub id_str: String,
    pub last: Option<chrono::NaiveDateTime>,
    pub disabled: bool,
}

#[derive(Queryable, Selectable, Associations)]
#[diesel(table_name = crate::schema::ip_needs_update)]
#[diesel(belongs_to(Service, foreign_key = service_trigger_id))]
// Cannot add 2 relationships to the same table, so I'll have to handle this manually in the code
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct IpNeedsUpdate {
    pub id: i32,
    pub service_trigger_id: i32,
    pub service_updated_id: i32,
    pub last_ip: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::user_variable)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserVariable {
    pub id: i32,
    pub id_str: String,
    pub value: Option<String>,
}

#[derive(Queryable, Selectable, Associations)]
#[diesel(table_name = crate::schema::user_var_needs_update)]
#[diesel(belongs_to(Service))]
#[diesel(belongs_to(UserVariable))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserVarNeedsUpdate {
    pub id: i32,
    pub service_id: i32,
    pub user_variable_id: i32,
    pub last_value: String,
}
