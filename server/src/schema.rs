// @generated automatically by Diesel CLI.

diesel::table! {
    backup_config (id) {
        id -> Integer,
        service_id -> Integer,
        id_str -> Text,
        last -> Nullable<Timestamp>,
        disabled -> Bool,
    }
}

diesel::table! {
    ip_needs_update (id) {
        id -> Integer,
        service_trigger_id -> Integer,
        service_updated_id -> Integer,
        last_ip -> Text,
    }
}

diesel::table! {
    server (id) {
        id -> Integer,
        id_str -> Text,
        description -> Nullable<Text>,
        ip -> Text,
        api_key -> Nullable<Text>,
        disabled -> Bool,
        reverse_api_key -> Text,
    }
}

diesel::table! {
    service (id) {
        id -> Integer,
        sync_server_id -> Nullable<Integer>,
        server_id -> Nullable<Integer>,
        id_str -> Text,
        name -> Text,
        last_config -> Nullable<Text>,
        sync_storage_id_str -> Nullable<Text>,
        last_sync -> Nullable<Timestamp>,
        sync_time -> Integer,
        disabled -> Bool,
    }
}

diesel::table! {
    user_var_needs_update (id) {
        id -> Integer,
        service_id -> Integer,
        user_variable_id -> Integer,
        last_value -> Text,
    }
}

diesel::table! {
    user_variable (id) {
        id -> Integer,
        id_str -> Text,
        value -> Nullable<Text>,
    }
}

diesel::joinable!(backup_config -> service (service_id));
diesel::joinable!(user_var_needs_update -> service (service_id));
diesel::joinable!(user_var_needs_update -> user_variable (user_variable_id));

diesel::allow_tables_to_appear_in_same_query!(
    backup_config,
    ip_needs_update,
    server,
    service,
    user_var_needs_update,
    user_variable,
);
