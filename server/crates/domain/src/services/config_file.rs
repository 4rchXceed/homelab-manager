use config::service::config_file::config::ConfigFileConfig;

use crate::services::generator::service_generator::ServiceGenerator;

pub struct ConfigFile {
    pub path: String,
    pub commands_when_updated: Vec<String>,
    pub reload_timeout: u64,
    pub service_generators: Vec<ServiceGenerator>,
}

impl ConfigFile {
    pub fn from_config(config: &ConfigFileConfig) -> Self {
        let service_generators = config
            .generators_config
            .iter()
            .map(|g| ServiceGenerator::from_config(g))
            .collect();

        return ConfigFile {
            path: config.path.clone(),
            commands_when_updated: config.when_config_updated.clone(),
            reload_timeout: config.reload_timeout as u64,
            service_generators: service_generators,
        };
    }
}
