use std::sync::Arc;

use application::generators::repositories::{
    generator_bases_registry::GeneratorBaseRegistry,
    generator_config_registry::GeneratorConfigRegistry,
};
use infrastructure::generators::{
    bases::{bash::BashGeneratorBase, replace::ReplaceFileGeneratorBase},
    inmemory::{
        generator_base_registry::InMemoryGeneratorRegistry,
        generator_config_registry::InMemoryGeneratorConfigRegistry,
    },
};

use crate::common::{load_config::load_config, testing_generator::TestingGenerator};

async fn setup() -> InMemoryGeneratorRegistry {
    let config = load_config(include_str!(
        "configs/generator_base_registry_testdatas.yaml"
    ));

    let generator_config_registry = Arc::new(InMemoryGeneratorConfigRegistry::new());

    generator_config_registry
        .reload(config.generators_config.clone())
        .await;

    let generator_base_registry = InMemoryGeneratorRegistry::new(generator_config_registry);

    let replace_base_registry = Arc::new(ReplaceFileGeneratorBase::new());
    let bash_generator_registry = Arc::new(BashGeneratorBase::new());

    generator_base_registry.add(bash_generator_registry).await;
    generator_base_registry.add(replace_base_registry).await;
    generator_base_registry
        .add(Arc::new(TestingGenerator::new(String::from("TestOk"))))
        .await;

    return generator_base_registry;
}

#[tokio::test]
async fn test_get_generator() {
    let generator_config_registry = setup().await;

    let (generator, generator_config) = generator_config_registry
        .get_generator(String::from("generator1"))
        .await
        .expect("Failed to get replace generator");

    assert_eq!(generator_config.generator_base_name, "replace");

    assert_eq!(generator.get_name(), "replace");

    let (generator, generator_config) = generator_config_registry
        .get_generator(String::from("generator3"))
        .await
        .expect("Failed to get replace generator");

    assert_eq!(generator_config.generator_base_name, "test");

    assert_eq!(
        generator
            .generate_commands(
                &generator_config,
                &vec![String::from("TestArgOk")],
                &String::new()
            )
            .await
            .expect("Failed to generate commands")
            .0[0],
        "TestOk"
    );
}
