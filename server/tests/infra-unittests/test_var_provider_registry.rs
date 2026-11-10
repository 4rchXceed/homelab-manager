use std::sync::Arc;

use application::var_provider::repositories::var_provider_registry::VarProviderRegistry;
use infrastructure::var_provider::{
    inmemeory::var_provider_registry::InMemoryVarProviderRegistry, types::string::StringVarProvider,
};

#[tokio::test]
pub async fn test_integration() {
    let var_provider_registry = InMemoryVarProviderRegistry::new();

    var_provider_registry
        .add(Arc::new(StringVarProvider::new()))
        .await;

    let result = var_provider_registry
        .var_provider_for_type("not-a-type".to_string())
        .await;

    assert!(result.is_err());

    var_provider_registry
        .var_provider_for_type("string".to_string())
        .await
        .expect("Failed to get 'string' var provider");
}
