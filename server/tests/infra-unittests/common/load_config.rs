use std::{fs::File, io::Write};

use config::{config::Config, loader::ConfigLoader};
use tempdir::TempDir;

pub fn load_config(config_text: &str) -> Config {
    let tempdir =
        TempDir::new("homelab-mgr-tests-cfg").expect("Failed to create TempDir in load()");

    let cfg_path = tempdir.path().join("config.yaml");
    let cfg_path_str = cfg_path.to_string_lossy().to_string();

    let mut cfg_file = File::create(cfg_path).expect("Failed to open TempDir config");
    cfg_file
        .write_all(config_text.as_bytes())
        .expect("Failed to write config");
    cfg_file.flush().expect("Failed to flush");

    let loader = ConfigLoader::with_path(cfg_path_str);

    let config = loader
        .load()
        .map_err(|e| e.to_string())
        .expect("Failed to load the config");

    return config.0;
}
