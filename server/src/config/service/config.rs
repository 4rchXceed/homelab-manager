use yaml_rust2::Yaml;

use crate::{
    config::{
        config::ConfigError,
        service::{backup::config::BackupConfig, config_file::config::ConfigFileConfig},
    },
    consts::DEFAULT_SERVICE_DESCRIPTION,
};

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub id: String,
    pub description: String,
    pub data_dirs: Vec<String>,
    pub backup_configs: Vec<BackupConfig>,
    pub config_files: Vec<ConfigFileConfig>,
}

impl ServiceConfig {
    pub fn from_yaml(yaml: &Yaml, key: String) -> Result<Self, ConfigError> {
        let description = yaml["description"]
            .as_str()
            .unwrap_or(DEFAULT_SERVICE_DESCRIPTION);
        let data_dirs = yaml["datas"]
            .as_vec()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or(ConfigError::ServiceConfigDataDirInvalid(key.clone()))
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<String>, ConfigError>>()?;

        let mut backup_configs = Vec::new();
        if !yaml["backup_configs"].is_badvalue() {
            if yaml["backup_configs"].is_array() {
                let backup_configs_raw = yaml["backup_configs"].as_vec().unwrap();
                for backup_config_raw in backup_configs_raw.iter() {
                    let backup_config = BackupConfig::from_yaml(backup_config_raw);
                    if backup_config.is_err() {
                        return Err(backup_config.err().unwrap());
                    }
                    backup_configs.push(backup_config.unwrap());
                }
            } else {
                return Err(ConfigError::ServiceConfigBackupConfigsNotArray(key.clone()));
            }
        }

        let mut config_files = Vec::new();
        if !yaml["config_files"].is_badvalue() {
            if yaml["config_files"].is_array() {
                let config_files_raw = yaml["config_files"].as_vec().unwrap();
                for config_file_raw in config_files_raw.iter() {
                    let config_file = ConfigFileConfig::from_yaml(config_file_raw);
                    if config_file.is_err() {
                        return Err(config_file.err().unwrap());
                    }
                    config_files.push(config_file.unwrap());
                }
            } else {
                return Err(ConfigError::ServiceConfigConfigFilesNotArray(key.clone()));
            }
        }

        return Ok(Self {
            id: key,
            description: String::from(description),
            data_dirs: data_dirs,
            backup_configs: backup_configs,
            config_files: config_files,
        });
    }
}
