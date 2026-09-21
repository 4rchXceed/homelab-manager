use yaml_rust2::Yaml;

use crate::{
    config::{
        config::ConfigError,
        generic::{fileserver_auth::FileServerAuthConfig, net::NetConfig},
    },
    consts::{
        DEFAULT_BACKUP_CHECK_INTERVAL, DEFAULT_DATABASE_FILE_PATH, DEFAULT_KEEPALIVE_INTERVAL,
        DEFAULT_SERVICES_FOLDER, DEFAULT_STARTUP_TIMEOUT, DEFAULT_UNIX_SOCKET_PATH,
    },
    utils::fs::parse_time,
};

#[derive(Debug, Clone)]
pub struct GeneralConfig {
    pub services_folder: String,
    pub database_file: String,
    pub net_config: NetConfig,
    pub unix_socket_path: String,
    pub startup_timeout: usize,
    pub keepalive_interval: usize,
    pub notifications_urls: Vec<String>,
    pub file_server_auth: FileServerAuthConfig,
    pub backup_check_interval: usize,
}

impl GeneralConfig {
    pub fn from_yaml(yaml: &Yaml) -> Result<GeneralConfig, ConfigError> {
        let services_folder = yaml["services_folder"]
            .as_str()
            .unwrap_or(DEFAULT_SERVICES_FOLDER);
        let database_file = yaml["database_file"]
            .as_str()
            .unwrap_or(DEFAULT_DATABASE_FILE_PATH);

        let net_config = NetConfig::from_yaml(&yaml["net"]);
        if net_config.is_err() {
            return Err(net_config.err().unwrap());
        }
        let net_config = net_config.unwrap();

        let unix_socket_path = yaml["unix_socket_path"]
            .as_str()
            .unwrap_or(DEFAULT_UNIX_SOCKET_PATH);

        let startup_timeout_raw = yaml["startup_timeout"]
            .as_str()
            .unwrap_or(DEFAULT_STARTUP_TIMEOUT);
        let startup_timeout = parse_time(startup_timeout_raw);
        if startup_timeout.is_err() {
            return Err(ConfigError::StartupTimeoutParseError(
                startup_timeout.err().unwrap(),
            ));
        }
        let startup_timeout = startup_timeout.unwrap();

        let keepalive_interval_raw = yaml["keepalive_interval"]
            .as_str()
            .unwrap_or(DEFAULT_KEEPALIVE_INTERVAL);
        let keepalive_interval = parse_time(keepalive_interval_raw);
        if keepalive_interval.is_err() {
            return Err(ConfigError::KeepAliveIntervalParseError(
                keepalive_interval.err().unwrap(),
            ));
        }
        let keepalive_interval = keepalive_interval.unwrap();

        let notifications_urls: Vec<String> = yaml["notifications_urls"]
            .as_vec()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|url| url.as_str().map(|s| s.to_string()))
            .collect();

        let file_server_auth_raw = &yaml["file_server_auth"];
        if file_server_auth_raw.is_badvalue() {
            return Err(ConfigError::FileServerAuthMissing);
        }

        let file_server_auth = FileServerAuthConfig::from_yaml(file_server_auth_raw);
        if file_server_auth.is_err() {
            return Err(file_server_auth.err().unwrap());
        }
        let file_server_auth = file_server_auth.unwrap();

        let backup_check_interval_raw = yaml["backup_check_interval"]
            .as_str()
            .unwrap_or(DEFAULT_BACKUP_CHECK_INTERVAL);
        let backup_check_interval = parse_time(backup_check_interval_raw);
        if backup_check_interval.is_err() {
            return Err(ConfigError::BackupCheckIntervalParseError(
                backup_check_interval.err().unwrap(),
            ));
        }
        let backup_check_interval = backup_check_interval.unwrap();

        return Ok(Self {
            services_folder: String::from(services_folder),
            database_file: String::from(database_file),
            net_config: net_config,
            unix_socket_path: String::from(unix_socket_path),
            startup_timeout: startup_timeout,
            keepalive_interval: keepalive_interval,
            notifications_urls: notifications_urls,
            file_server_auth: file_server_auth,
            backup_check_interval: backup_check_interval,
        });
    }
}
