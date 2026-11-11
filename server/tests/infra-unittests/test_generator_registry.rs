use std::sync::Arc;

use application::generators::repositories::generator_config_registry::GeneratorConfigRegistry;
use config::config::Config;
use infrastructure::generators::inmemory::generator_config_registry::InMemoryGeneratorConfigRegistry;

use crate::common::load_config::load_config;

async fn setup() -> (Arc<InMemoryGeneratorConfigRegistry>, Config) {
    let config = load_config(include_str!("configs/generator_registry_testdatas.yaml"));

    let generator_config_registry = InMemoryGeneratorConfigRegistry::new();

    generator_config_registry
        .reload(config.generators_config.clone())
        .await;

    return (Arc::new(generator_config_registry), config);
}

#[tokio::test]
async fn test_get_by_name() {
    let (generator_config_registry, _config) = setup().await;

    let generator_config = generator_config_registry
        .get_config(String::from("generator1"))
        .await
        .expect("Config not found");

    assert_eq!(generator_config.generator_base_name, "replace");
    assert_eq!(generator_config.id, "generator1");
    assert_eq!(generator_config.require_sample_file, false);
    assert_eq!(generator_config.timeout, 60);
}

#[tokio::test]
async fn test_reload() {
    let (generator_config_registry, config) = setup().await;

    let configs = generator_config_registry.get_configs_testonly().await;

    assert_eq!(configs.len(), 2);

    assert_eq!(configs["generator1"].generator_base_name, "replace");
    assert_eq!(configs["generator1"].id, "generator1");
    assert_eq!(configs["generator1"].require_sample_file, false);
    assert_eq!(configs["generator1"].timeout, 60);

    assert_eq!(configs["generator2"].generator_base_name, "bash");
    assert_eq!(configs["generator2"].id, "generator2");
    assert_eq!(configs["generator2"].require_sample_file, true);
    assert_eq!(configs["generator2"].timeout, 120);
    assert_eq!(
        configs["generator2"]
            .arguments
            .get("commands")
            .expect("'commands' not in arguments (generator2)")
            .as_vec()
            .expect("'commands' is not a vec (generator2)")[0]
            .as_str()
            .expect("'commands[0]' is not a string"),
        "test123"
    );

    let mut generator_configs = config.generators_config;

    generator_configs.insert(
        "generator3".to_string(),
        config::generator::config::GeneratorBaseConfig {
            id: "generator3".to_string(),
            generator_base_name: "replace".to_string(),
            require_sample_file: false,
            timeout: 30,
            arguments: Default::default(),
        },
    );

    generator_config_registry.reload(generator_configs).await;

    let configs = generator_config_registry.get_configs_testonly().await;

    assert_eq!(configs.len(), 3);

    assert_eq!(configs["generator3"].generator_base_name, "replace");
    assert_eq!(configs["generator3"].id, "generator3");
    assert_eq!(configs["generator3"].require_sample_file, false);
    assert_eq!(configs["generator3"].timeout, 30);
}
