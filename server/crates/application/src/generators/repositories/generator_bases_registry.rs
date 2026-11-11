use std::sync::Arc;

use async_trait::async_trait;
use config::generator::config::GeneratorBaseConfig;

use crate::generators::repositories::generator_base::GeneratorBase;

#[async_trait]
pub trait GeneratorBaseRegistry: Send + Sync {
    async fn get_generator(
        &self,
        name: String,
    ) -> Result<(Arc<dyn GeneratorBase>, GeneratorBaseConfig), String>;

    async fn add(&self, generator_base: Arc<dyn GeneratorBase>);
}
