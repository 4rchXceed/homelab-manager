use yaml_rust2::Yaml;

use crate::{
    config::config::ConfigError,
    consts::{DEFAULT_BACKUP_TRANSFER_PORT, DEFAULT_SERVER_PORT, MAX_PORT_NBR},
};

#[derive(Debug)]
pub struct NetConfig {
    pub server_port: usize,
    pub file_server_port: usize,
    pub backup_transfer_port: usize,
}

impl NetConfig {
    pub fn from_yaml(yaml: &Yaml) -> Result<NetConfig, ConfigError> {
        let server_port = yaml["server_port"].as_i64().unwrap_or(DEFAULT_SERVER_PORT);
        if server_port < 0 || server_port > MAX_PORT_NBR {
            return Err(ConfigError::InvalidServerPort(server_port));
        }
        let file_server_port = yaml["file_server_port"]
            .as_i64()
            .unwrap_or(DEFAULT_SERVER_PORT);
        if file_server_port < 0 || file_server_port > MAX_PORT_NBR {
            return Err(ConfigError::InvalidFileServerPort(file_server_port));
        }
        let backup_transfer_port = yaml["backup_transfer_port"]
            .as_i64()
            .unwrap_or(DEFAULT_BACKUP_TRANSFER_PORT);
        if backup_transfer_port < 0 || backup_transfer_port > MAX_PORT_NBR {
            return Err(ConfigError::InvalidBackupTransferPort(backup_transfer_port));
        }

        return Ok(NetConfig {
            server_port: server_port as usize,
            file_server_port: file_server_port as usize,
            backup_transfer_port: backup_transfer_port as usize,
        });
    }
}
