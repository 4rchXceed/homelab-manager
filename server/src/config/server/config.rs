use std::{collections::HashMap, net::IpAddr};

use crate::config::server::storage_config::StorageConfig;

#[derive(Debug)]
pub struct ServerConfig {
    pub id: String,
    pub description: String,
    pub ip: IpAddr,
    pub api_key: String,
    pub storages: HashMap<String, StorageConfig>,
}
