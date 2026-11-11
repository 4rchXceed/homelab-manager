use application::generators::repositories::generator_base::GeneratorBase;
use config::generator::config::GeneratorBaseConfig;
use domain::agent::protocol::message::GeneratorMode;
use infrastructure::generators::bases::bash::BashGeneratorBase;
use tempdir::TempDir;

use crate::common::{generate_db::load_yaml, run_docker::run_commands_in_docker};

const YAML_OK: &str = r#"
base: bash
commands:
    - echo "$ARG0"
based_on_sample: false
timeout: 60s
base_image: debian:bookworm
"#;

const YAML_ERROR: &str = r#"
base: bash
based_on_sample: false
timeout: 60s
base_image: testerror:latest
"#;

#[tokio::test]
async fn test_bash_generator_fail() {
    let bash_generator = BashGeneratorBase::new();

    let config_yaml = load_yaml(YAML_ERROR);

    let generator_base_config =
        GeneratorBaseConfig::from_yaml(&config_yaml, String::from("generator1"))
            .map_err(|e| e.to_string())
            .expect("Failed to parse config");

    let args = vec![String::from("TestOk$PWD")];
    let fname = String::new();

    let error = bash_generator
        .generate_commands(&generator_base_config, &args, &fname)
        .await;

    assert!(error.is_err());
}

#[tokio::test]
async fn test_bash_generator_ok() {
    let bash_generator = BashGeneratorBase::new();

    let config_yaml = load_yaml(YAML_OK);

    let generator_base_config =
        GeneratorBaseConfig::from_yaml(&config_yaml, String::from("generator1"))
            .map_err(|e| e.to_string())
            .expect("Failed to parse config");

    assert_eq!(bash_generator.get_name(), "bash");

    let args = vec![String::from("TestOk$PWD")];
    let fname = String::new();

    let (commands, mode, image) = bash_generator
        .generate_commands(&generator_base_config, &args, &fname)
        .await
        .map_err(|e| e.to_string())
        .expect("Failed to generate commands");

    assert_eq!(
        image
            .clone()
            .expect("Custom image should exist, but doesn't"),
        "debian:bookworm"
    );
    assert_eq!(mode, GeneratorMode::Sandboxed);

    let tempdir = TempDir::new("homelab-manager-test").expect("Failed to create tempdir");

    let result = run_commands_in_docker(
        tempdir.path().to_string_lossy().to_string(),
        commands,
        image,
    )
    .await;

    assert!(result.contains("TestOk$PWD"));
}
