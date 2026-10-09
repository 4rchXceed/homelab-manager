use std::collections::HashMap;

use async_trait::async_trait;
use config::generator::config::GeneratorBaseConfig;

#[async_trait]
pub trait GeneratorConfigRegistry: Send + Sync {
    async fn reload(&self, configs: HashMap<String, GeneratorBaseConfig>);

    async fn get_config(&self, name: String) -> Option<GeneratorBaseConfig>;
}
