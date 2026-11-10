use std::sync::Arc;

use application::var_provider::{
    context::VarProviderContext, repositories::var_provider::VarProvider,
};
use infrastructure::{
    interactions::cli::user_context::CliUserContext, var_provider::types::string::StringVarProvider,
};
use yaml_rust2::YamlLoader;

const YAML_NORMAL: &str = "
value: SomeString
";

const YAML_INVALID: &str = "
notvalue: SomeString
";

#[tokio::test]
pub async fn test_string_var_fail() {
    let documents = YamlLoader::load_from_str(YAML_INVALID)
        .map_err(|e| e.to_string())
        .expect("Invalid YAML");

    let yaml_parsed = documents.first().expect("No documents");

    let var_provider = StringVarProvider::new();

    let value = var_provider
        .process(
            yaml_parsed.clone(),
            String::new(),
            VarProviderContext {
                user_context: Arc::new(CliUserContext::new()),
            },
        )
        .await;

    assert!(value.is_err());
}

#[tokio::test]
pub async fn test_string_var_normal() {
    let documents = YamlLoader::load_from_str(YAML_NORMAL)
        .map_err(|e| e.to_string())
        .expect("Invalid YAML");

    let yaml_parsed = documents.first().expect("No documents");

    let var_provider = StringVarProvider::new();

    let value = var_provider
        .process(
            yaml_parsed.clone(),
            String::new(),
            VarProviderContext {
                user_context: Arc::new(CliUserContext::new()),
            },
        )
        .await
        .expect("Error: string var provider failed");

    assert_eq!(value, "SomeString");
}
