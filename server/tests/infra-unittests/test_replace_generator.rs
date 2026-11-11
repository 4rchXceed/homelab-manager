use application::generators::repositories::generator_base::GeneratorBase;
use config::generator::config::GeneratorBaseConfig;
use domain::agent::protocol::message::GeneratorMode;
use infrastructure::generators::bases::replace::ReplaceFileGeneratorBase;
use tempdir::TempDir;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::common::{generate_db::load_yaml, run_docker::run_commands_in_docker};

const YAML_OK: &str = r#"
base: replace
string: TestError
based_on_sample: false
timeout: 60s
"#;

#[tokio::test]
async fn test_replace_generator_ok() {
    let bash_generator = ReplaceFileGeneratorBase::new();

    let config_yaml = load_yaml(YAML_OK);

    let generator_base_config =
        GeneratorBaseConfig::from_yaml(&config_yaml, String::from("generator1"))
            .map_err(|e| e.to_string())
            .expect("Failed to parse config");

    assert_eq!(bash_generator.get_name(), "replace");

    let args = vec![String::from("TestOk")];
    let fname = String::from("testfile.txt");

    let (commands, mode, image) = bash_generator
        .generate_commands(&generator_base_config, &args, &fname)
        .await
        .map_err(|e| e.to_string())
        .expect("Failed to generate commands");

    assert_eq!(mode, GeneratorMode::Sandboxed);

    let tempdir = TempDir::new("homelab-manager-test").expect("Failed to create tempdir");

    let replace_file_path = tempdir.path().join(&fname);

    let mut file = File::create(&replace_file_path)
        .await
        .expect("Failed to create test file");

    let content = "TEST=TestError\n";
    file.write_all(content.as_bytes())
        .await
        .expect("Failed to write to test file");
    file.flush().await.expect("Failed to flush test file");

    run_commands_in_docker(
        tempdir.path().to_string_lossy().to_string(),
        commands,
        image,
    )
    .await;

    let mut file = File::open(&replace_file_path)
        .await
        .expect("Failed to open test file");

    let mut content = String::new();
    file.read_to_string(&mut content)
        .await
        .expect("Failed to read test file");

    assert_eq!(content, "TEST=TestOk\n");
}
