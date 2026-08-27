use std::collections::HashMap;

use yaml_rust2::Yaml;

use crate::{
    config::{
        generator::config::GeneratorConfig, generic::config::GeneralConfig,
        server::config::ServerConfig, service::config::ServiceConfig,
    },
    utils::fs::{FileSizeParseError, TimeParseError},
};

#[derive(Debug, Clone)]
pub enum ConfigError {
    NoConfig,
    NoGeneralConfig,
    NetConfigMissing,
    FileServerAuthMissing,
    StartupTimeoutParseError(TimeParseError),
    KeepAliveIntervalParseError(TimeParseError),
    BackupCheckIntervalParseError(TimeParseError),
    FileServerAuthUsernameMissing,
    FileServerAuthPasswordMissing,
    NoGeneratorsConfig,
    GeneratorsConfigNotObject,
    OneGeneratorKeyNotFound,
    GeneratorConfigBaseMissing(String),
    GeneratorConfigBashCommandsMissing,
    GeneratorConfigBashCommandNotString,
    GeneratorConfigBashTimeoutParseError(TimeParseError),
    InvalidFileServerPort(i64),
    InvalidServerPort(i64),
    InvalidBackupTransferPort(i64),
    ServiceConfigDataDirInvalid(String),
    ServiceConfigBackupConfigsNotArray(String),
    BackupConfigIdMissing,
    BackupConfigBackupTypeMissing(String),
    BackupConfigMaxSizeMissing(String),
    BackupConfigMaxAgeMissing(String),
    BackupConfigScheduleMissing(String),
    BackupConfigMaxSizeParseError(String, FileSizeParseError),
    BackupConfigBackupTypeWrong(String),
    ServiceConfigConfigFilesNotArray(String),
    ConfigFileConfigPathMissing,
    ConfigFileWhenConfigUpdatedCommandNotString(String),
    ConfigFileReloadTimeoutParseError(String, String),
    ServiceConfigGeneratorsNotArray(String),
    RunGeneratorIdMissing,
    RunGeneratorArgsNotArray(String),
    GeneratorArgumentTypeNotFound,
    GeneratorArgumentStringNotFound,
    GeneratorArgumentTypeNotSupported(String),
    ServicesConfigNotObject,
    OneServiceKeyNotFound,
    ServerIpMissing(String),
    ServerIpInvalid(String, String),
    ServerApiKeyMissing(String),
    ServerStorageIdInvalid(String),
    ServerStoragePathMissing(String),
    ServerConfigMissing,
    ServerIdMissing,
}

#[derive(Debug)]
pub struct Config {
    pub config_general: GeneralConfig,
    pub generators_config: HashMap<String, GeneratorConfig>,
    pub services_config: HashMap<String, ServiceConfig>,
    pub servers_config: Vec<ServerConfig>,
}

impl Config {
    pub fn from_yaml(yaml: &Yaml) -> Result<Config, ConfigError> {
        if yaml["config"].is_badvalue() {
            return Err(ConfigError::NoGeneralConfig);
        }
        let config_general = GeneralConfig::from_yaml(&yaml["config"]);
        if config_general.is_err() {
            return Err(config_general.err().unwrap());
        }
        let config_general = config_general.unwrap();

        let mut generators_config: HashMap<String, GeneratorConfig> = HashMap::new();
        if yaml["generators"].is_badvalue() {
            return Err(ConfigError::NoGeneratorsConfig);
        }
        if !yaml["generators"].is_hash() {
            return Err(ConfigError::GeneratorsConfigNotObject);
        }
        let generators_config_raw = yaml["generators"].as_hash().unwrap();
        for (key, value) in generators_config_raw.iter() {
            let key_str = key.as_str();
            if key_str.is_none() {
                return Err(ConfigError::OneGeneratorKeyNotFound);
            }
            let key_str = key_str.unwrap();

            let generator_config = GeneratorConfig::from_yaml(value, String::from(key_str));
            if generator_config.is_err() {
                return Err(generator_config.err().unwrap());
            }
            generators_config.insert(key_str.to_string(), generator_config.unwrap());
        }

        if yaml["services"].is_badvalue() || !yaml["services"].is_hash() {
            return Err(ConfigError::ServicesConfigNotObject);
        }
        let services_config_raw = yaml["services"].as_hash().unwrap();

        let mut services_config: HashMap<String, ServiceConfig> = HashMap::new();
        for (key, value) in services_config_raw.iter() {
            let key_str = key.as_str();
            if key_str.is_none() {
                return Err(ConfigError::OneServiceKeyNotFound);
            }
            let key_str = key_str.unwrap();
            let service_config = ServiceConfig::from_yaml(value, String::from(key_str));
            if service_config.is_err() {
                return Err(service_config.err().unwrap());
            }
            services_config.insert(key_str.to_string(), service_config.unwrap());
        }

        let servers_config_raw = yaml["servers"]
            .as_vec()
            .ok_or(ConfigError::ServerConfigMissing)?;

        let mut servers_config: Vec<ServerConfig> = Vec::new();
        for server_config_raw in servers_config_raw.iter() {
            let server_config = ServerConfig::from_yaml(server_config_raw)?;
            servers_config.push(server_config);
        }

        return Ok(Self {
            config_general: config_general,
            generators_config: generators_config,
            services_config: services_config,
            servers_config: servers_config,
        });
    }
}
