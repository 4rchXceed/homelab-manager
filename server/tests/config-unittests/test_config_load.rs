use config::loader::ConfigLoader;
use utils::fs::create_temp_dir;

static CONFIG_SAMPLE: &str = include_str!("../documents/config_test.yaml");

#[test]
fn test_load_config() {
    let tmp_res = create_temp_dir();
    assert!(tmp_res.is_ok());
    let (tmp_path, tmp_folder) = tmp_res.unwrap();

    let config_file_path = format!("{}/config_test.yaml", tmp_path);
    std::fs::write(&config_file_path, CONFIG_SAMPLE)
        .expect("Failed to write config sample to temp file");

    let config_loader = ConfigLoader::with_path(config_file_path);
    let config_result = config_loader
        .load()
        .map_err(|e| e.to_string())
        .expect("Failed to load config");
    println!("Config result: {:?}", config_result);
    let _ = tmp_folder.close();
}
