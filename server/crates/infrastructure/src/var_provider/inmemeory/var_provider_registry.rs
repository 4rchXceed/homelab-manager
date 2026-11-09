use std::sync::Arc;

use application::var_provider::repositories::{
    var_provider::VarProvider, var_provider_registry::VarProviderRegistry,
};
use async_trait::async_trait;
use tokio::sync::RwLock;

pub struct InMemoryVarProviderRegistry {
    providers: Arc<RwLock<Vec<Arc<dyn VarProvider>>>>,
}

impl InMemoryVarProviderRegistry {
    pub fn new() -> Self {
        return Self {
            providers: Arc::new(RwLock::new(Vec::new())),
        };
    }

    pub async fn add(&self, provider: Arc<dyn VarProvider>) {
        let mut providers = self.providers.write().await;

        providers.push(provider);
    }
}

#[async_trait]
impl VarProviderRegistry for InMemoryVarProviderRegistry {
    async fn var_provider_for_type(&self, name: String) -> Result<Arc<dyn VarProvider>, String> {
        let providers = self.providers.read().await;

        let provider = providers
            .iter()
            .find(|p| p.get_name() == name)
            .ok_or(format!("Var provider not found for name: {}", name))?;

        return Ok(provider.clone());
    }
}
