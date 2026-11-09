use std::sync::Arc;

use async_trait::async_trait;

use crate::var_provider::repositories::var_provider::VarProvider;

#[async_trait]
pub trait VarProviderRegistry: Send + Sync {
    async fn var_provider_for_type(&self, name: String) -> Result<Arc<dyn VarProvider>, String>;
}
