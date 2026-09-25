use std::collections::HashMap;

use config::agent::storage_config::StorageConfig;

use crate::backup::storage::Storage;

pub struct Agent {
    pub id: String,
    pub api_key: String,
    pub ip: String,
    pub storages_configs: HashMap<String, StorageConfig>,
    pub storages: Vec<Storage>,
    pub reverse_api_key: String,
}
