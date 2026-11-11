use std::collections::HashMap;
use std::sync::Arc;

use application::generators::repositories::generator_config_registry::GeneratorConfigRegistry;
use async_trait::async_trait;
use config::generator::config::GeneratorBaseConfig;
use tokio::sync::RwLock;

pub struct InMemoryGeneratorConfigRegistry {
    pub configs: Arc<RwLock<HashMap<String, GeneratorBaseConfig>>>,
}

impl InMemoryGeneratorConfigRegistry {
    pub fn new() -> Self {
        return Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
        };
    }
}

#[async_trait]
impl GeneratorConfigRegistry for InMemoryGeneratorConfigRegistry {
    async fn reload(&self, configs: HashMap<String, GeneratorBaseConfig>) {
        let mut configs_lock = self.configs.write().await;

        *configs_lock = configs;
    }

    async fn get_config(&self, name: String) -> Option<GeneratorBaseConfig> {
        let configs_lock = self.configs.read().await;

        return configs_lock.get(&name).cloned();
    }
}
