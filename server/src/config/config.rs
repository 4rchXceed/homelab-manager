use std::collections::HashMap;

use yaml_rust2::Yaml;

use thiserror::Error;

use crate::{
    config::{
        generator::config::GeneratorConfig, generic::config::GeneralConfig,
        server::config::ServerConfig, service::config::ServiceConfig,
    },
    utils::fs::{FileSizeParseError, TimeParseError},
};

#[derive(Debug, Clone, Error)]
pub enum ConfigError {
    #[error("Empty config file (no config found)")]
    NoConfig,
    #[error("No general config found at the top level of the config file (/general)")]
    NoGeneralConfig,
    #[error("The network config is missing (/general/net)")]
    NetConfigMissing,
    #[error("The file server auth config is missing (/general/file_server_auth)")]
    FileServerAuthMissing,
    #[error("The file server auth username is missing (/general/file_server_auth/user)")]
    FileServerAuthUsernameMissing,
    #[error("The file server auth password is missing (/general/file_server_auth/password)")]
    FileServerAuthPasswordMissing,
    #[error(
        "The startup timeout config is in an invalid format: {0} (/general/startup_timeout) [ex: 120s]"
    )]
    StartupTimeoutParseError(TimeParseError),
    #[error(
        "The keepalive interval config is in an invalid format: {0} (/general/keepalive_interval) [ex: 5s]"
    )]
    KeepAliveIntervalParseError(TimeParseError),
    #[error(
        "The backup check interval config is in an invalid format: {0} (/general/backup_check_interval) [ex: 5m]"
    )]
    BackupCheckIntervalParseError(TimeParseError),
    #[error("The generators config is missing (/generators)")]
    NoGeneratorsConfig,
    #[error("The generators config is not an object (/generators)")]
    GeneratorsConfigNotObject,
    #[error("The generators config's key is not a string (/generators)")]
    OneGeneratorKeyNotFound,
    #[error("A generator's \"base\" property is missing (/generators/{0}/base)")]
    GeneratorConfigBaseMissing(String),
    #[error("Bash generator: \"commands\" property is missing (/generators/N/commands)")]
    GeneratorConfigBashCommandsMissing,
    #[error("Bash generator: one of the commands is not a string (/generators/N/commands/N)")]
    GeneratorConfigBashCommandNotString,
    #[error(
        "Bash generator: \"timeout\" property is in an invalid format: {0} (/generators/N/timeout) [ex: 30s]"
    )]
    GeneratorConfigBashTimeoutParseError(TimeParseError),
    #[error(
        "The file server's port is invalid: {0} (/general/file_server_port) [valid range: 1-65535]"
    )]
    InvalidFileServerPort(i64),
    #[error("The main server port is invalid: {0} (/general/server_port) [valid range: 1-65535]")]
    InvalidServerPort(i64),
    #[error(
        "The backup transfer port is invalid: {0} (/general/backup_transfer_port) [valid range: 1-65535]"
    )]
    InvalidBackupTransferPort(i64),
    #[error("The backup config path is invalid (missing / not a string) (/services/{0}/datas/N)")]
    ServiceConfigDataDirInvalid(String),
    #[error("The backup configs is not an array (/services/{0}/backups)")]
    ServiceConfigBackupConfigsNotArray(String),
    #[error("The backup config is missing the \"id\" property (/services/backups/N/id)")]
    BackupConfigIdMissing,
    #[error("The backup config is missing the \"type\" property (/services/{0}/backups/N/type)")]
    BackupConfigBackupTypeMissing(String),
    #[error(
        "The backup config is missing the \"max_size\" property (/services/{0}/backups/N/max_size)"
    )]
    BackupConfigMaxSizeMissing(String),
    #[error(
        "The backup config is missing the \"max_age\" property (/services/{0}/backups/N/max_age)"
    )]
    BackupConfigMaxAgeMissing(String),
    #[error(
        "The backup config is missing the \"schedule\" property (/services/{0}/backups/N/schedule)"
    )]
    BackupConfigScheduleMissing(String),
    #[error(
        "The backup config's \"max_age\" property is in an invalid format: {1} (/services/{0}/backups/N/max_age) [ex: 30d]"
    )]
    BackupConfigMaxSizeParseError(String, FileSizeParseError),
    #[error(
        "The backup config's \"type\" property can only be full or incremental (/services/{0}/backups/N/type) [ex: 30d]"
    )]
    BackupConfigBackupTypeWrong(String),
    #[error("The service \"config_files\" property is not an array (/services/{0}/config_files)")]
    ServiceConfigConfigFilesNotArray(String),
    #[error(
        "The service \"config_files\" property is missing the \"path\" property (/services/N/config_files/N/path)"
    )]
    ConfigFileConfigPathMissing,
    #[error(
        "The service \"config_files\" property is missing the \"when_config_updated\" property or isn't a string (/services/{0}/config_files/N/when_config_updated)"
    )]
    ConfigFileWhenConfigUpdatedCommandNotString(String),
    #[error(
        "The service \"config_files\"'s \"reload_timeout\" property is in an invalid format (ex. 5) (/services/{0}/config_files/N/reload_timeout)"
    )]
    ConfigFileReloadTimeoutParseError(String, String),
    #[error(
        "The service \"config_files\"'s \"generators\" property is not an array (at config_path with path: {0}) (/services/N/config_files/N/generators)"
    )]
    ServiceConfigGeneratorsNotArray(String),
    #[error(
        "One SERVICE generator's \"name\" property is missing (/services/N/config_files/N/generators/N/name)"
    )]
    RunGeneratorNameMissing,
    #[error(
        "One SERVICE generator's \"args\" property is missing or isn't an array (generator with type: {0}) (/services/N/config_files/N/generators/N/args)"
    )]
    RunGeneratorArgsNotArray(String),
    #[error(
        "One SERVICE generator's argument is missing the \"type\" property (/services/N/config_files/N/generators/N/args/N/type)"
    )]
    GeneratorArgumentTypeNotFound,
    #[error(
        "One SERVICE generator's argument with the type \"string\" is missing the \"value\" property (/services/N/config_files/N/generators/N/args/N/value)"
    )]
    GeneratorArgumentStringNotFound,
    #[error(
        "The SERVICE generator's argument type: {0} is not supported (/services/N/config_files/N/generators/N/args/N/type)"
    )]
    GeneratorArgumentTypeNotSupported(String),
    #[error("The services config is missing or isn't an object (/services)")]
    ServicesConfigNotObject,
    #[error("One SERVICE config's key is not a string (/services)")]
    OneServiceKeyNotFound,
    #[error("One server's \"ip\" property is missing (/servers/{0})")]
    ServerIpMissing(String),
    #[error("One server's \"ip\" property is an invalid IPv4/v6 IP: {1} (/servers/{0})")]
    ServerIpInvalid(String, String),
    #[error("One server's \"api_key\" property is missing (/servers/{0})")]
    ServerApiKeyMissing(String),
    #[error("One server's \"storages\"'s key is missing (/servers/{0})")]
    ServerStorageIdInvalid(String),
    #[error("One server's \"storages\"'s path is missing (/servers/{0}/storages/N/path)")]
    ServerStoragePathMissing(String),
    #[error("The servers config is missing or isn't an array (/servers")]
    ServerConfigMissing,
    #[error("One server's key is missing (/servers/N/)")]
    ServerIdMissing,
    #[error(
        "One SERVICE generator's argument with the type \"user_var\" is missing the \"name\" property (/services/N/config_files/N/generators/N/args/N/name)"
    )]
    GeneratorArgumentUserVarNameNotFound,
    #[error(
        "One SERVICE generator's argument with the type \"user_var\" is missing the \"id\" property (/services/N/config_files/N/generators/N/args/N/id)"
    )]
    GeneratorArgumentUserVarIdNotFound,
    #[error(
        "At least one bind address must be specified in the network config (/general/net/binds)"
    )]
    NoBindAddresses,
    #[error(
        "Invalid log level, must be either: debug, error, trace, warn, info, got: {0} (/general/log_level"
    )]
    InvalidLogLevel(String),
}

#[derive(Debug, Clone)]
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
