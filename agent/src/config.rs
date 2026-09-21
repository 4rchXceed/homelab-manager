use ini::ini;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to open ini file: {0}")]
    FailedToOpenIniFile(String),
    #[error("Missing required section: {0} in ini file")]
    MissingSection(String),
    #[error("Missing required key: {0} in section: {1} of ini file")]
    MissingKey(String, String),
}

pub struct Config {
    // Auth
    pub id: String,                   // Required
    pub api_key: String,              // Required
    pub file_server_username: String, // Required
    pub file_server_password: String, // Required
    // Paths
    pub service_folder: String,   // Optional
    pub certificate_path: String, // Required
    // Connection
    pub server_host: String,     // Required
    pub server_port: u16,        // Optional
    pub file_server_port: u16,   // Optional
    pub backup_relay_port: u16,  // Optional
    pub keepalive_max_time: u64, // Optional
}

impl Config {
    pub fn from_ini(ini_path: String) -> Result<Self, ConfigError> {
        let ini = ini!(safe ini_path).map_err(|e| ConfigError::FailedToOpenIniFile(e))?;

        let auth_section = ini
            .get("auth")
            .ok_or(ConfigError::MissingSection("auth".to_string()))?
            .iter()
            .filter(|(_, v)| v.is_some())
            .map(|(k, v)| (k.clone(), v.as_ref().unwrap().clone()))
            .collect::<std::collections::HashMap<String, String>>();

        let id = auth_section.get("id").ok_or(ConfigError::MissingKey(
            "id".to_string(),
            "auth".to_string(),
        ))?;

        let api_key = auth_section.get("api_key").ok_or(ConfigError::MissingKey(
            "api_key".to_string(),
            "auth".to_string(),
        ))?;

        let file_server_username =
            auth_section
                .get("file_server_username")
                .ok_or(ConfigError::MissingKey(
                    "file_server_username".to_string(),
                    "auth".to_string(),
                ))?;

        let file_server_password =
            auth_section
                .get("file_server_password")
                .ok_or(ConfigError::MissingKey(
                    "file_server_password".to_string(),
                    "auth".to_string(),
                ))?;

        let paths_section = ini
            .get("paths")
            .ok_or(ConfigError::MissingSection("paths".to_string()))?
            .iter()
            .filter(|(_, v)| v.is_some())
            .map(|(k, v)| (k.clone(), v.as_ref().unwrap().clone()))
            .collect::<std::collections::HashMap<String, String>>();

        let service_folder = paths_section
            .get("service_folder")
            .unwrap_or(&"services".to_string())
            .to_string();

        let certificate_path =
            paths_section
                .get("certificate_path")
                .ok_or(ConfigError::MissingKey(
                    "certificate_path".to_string(),
                    "paths".to_string(),
                ))?;

        let connection_section = ini
            .get("connection")
            .ok_or(ConfigError::MissingSection("connection".to_string()))?
            .iter()
            .filter(|(_, v)| v.is_some())
            .map(|(k, v)| (k.clone(), v.as_ref().unwrap().clone()))
            .collect::<std::collections::HashMap<String, String>>();

        let server_host = connection_section
            .get("server_host")
            .ok_or(ConfigError::MissingKey(
                "server_host".to_string(),
                "connection".to_string(),
            ))?;

        let server_port = connection_section
            .get("server_port")
            .unwrap_or(&"4398".to_string())
            .parse::<u16>()
            .map_err(|_| {
                ConfigError::MissingKey("server_port".to_string(), "connection".to_string())
            })?;

        let file_server_port = connection_section
            .get("file_server_port")
            .unwrap_or(&"4399".to_string())
            .parse::<u16>()
            .map_err(|_| {
                ConfigError::MissingKey("file_server_port".to_string(), "connection".to_string())
            })?;

        let backup_server_port = connection_section
            .get("backup_server_port")
            .unwrap_or(&"4397".to_string())
            .parse::<u16>()
            .map_err(|_| {
                ConfigError::MissingKey("backup_server_port".to_string(), "connection".to_string())
            })?;

        let keepalive_max_time = connection_section
            .get("keepalive_max_time")
            .unwrap_or(&"30".to_string())
            .parse::<u64>()
            .map_err(|_| {
                ConfigError::MissingKey("keepalive_max_time".to_string(), "connection".to_string())
            })?;

        return Ok(Self {
            id: id.clone(),
            api_key: api_key.clone(),
            file_server_username: file_server_username.clone(),
            file_server_password: file_server_password.clone(),
            service_folder: service_folder.clone(),
            server_host: server_host.clone(),
            server_port: server_port,
            file_server_port: file_server_port,
            backup_relay_port: backup_server_port,
            keepalive_max_time: keepalive_max_time,
            certificate_path: certificate_path.clone(),
        });
    }
}
