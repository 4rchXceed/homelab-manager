use std::sync::Arc;

use application::generators::repositories::generator_base::GeneratorBase;
use application::generators::repositories::generator_bases_registry::GeneratorBaseRegistry;
use application::generators::repositories::generator_config_registry::GeneratorConfigRegistry;
use async_trait::async_trait;
use config::generator::config::GeneratorBaseConfig;
use tokio::sync::RwLock;

pub struct InMemoryGeneratorRegistry {
    pub generators: Arc<RwLock<Vec<Arc<dyn GeneratorBase>>>>,
    pub configs: Arc<dyn GeneratorConfigRegistry>,
}

impl InMemoryGeneratorRegistry {
    pub fn new(configs: Arc<dyn GeneratorConfigRegistry>) -> Self {
        return Self {
            generators: Arc::new(RwLock::new(Vec::new())),
            configs: configs,
        };
    }

    pub async fn add(&self, generator_base: Arc<dyn GeneratorBase>) {
        let mut generators = self.generators.write().await;

        generators.push(generator_base);
    }
}

#[async_trait]
impl GeneratorBaseRegistry for InMemoryGeneratorRegistry {
    async fn generator_for_name(
        &self,
        name: String,
    ) -> Result<(Arc<dyn GeneratorBase>, GeneratorBaseConfig), String> {
        let config = self.configs.get_config(name.clone()).await.ok_or(format!(
            "Generator config not found for name: {}",
            name
        ))?;

        let generators = self.generators.read().await;

        let generator = generators
            .iter()
            .find(|g| g.get_name() == config.generator_base_name)
            .cloned().ok_or(format!(
                "Generator base not found for name: {}",
                config.generator_base_name
            ))?;

        return Ok((generator, config));
    }
}
