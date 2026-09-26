use config::service::config::ServiceConfig;

use crate::{
    backup::backup::Backup,
    services::{config_file::ConfigFile, sync_config::SyncConfig},
};

pub struct Service {
    pub id: String,
    pub description: String,
    pub datas: Vec<String>,
    pub backups: Vec<Backup>,
    pub config_files: Vec<ConfigFile>,
    pub agent_host_id: Option<String>,
    pub sync_config: Option<SyncConfig>,
}

impl Service {
    pub fn from_config_and_db(
        config: ServiceConfig,
        agent_host: Option<String>,
        sync_config: Option<SyncConfig>,
        backups: Vec<Backup>,
    ) -> Self {
        let config_files = config
            .config_files
            .iter()
            .map(|c| ConfigFile::from_config(c))
            .collect();

        return Service {
            id: config.id,
            description: config.description,
            datas: config.data_dirs,
            backups: backups,
            config_files: config_files,
            agent_host_id: agent_host,
            sync_config: sync_config,
        };
    }
}
