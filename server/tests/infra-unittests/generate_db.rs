use infrastructure::database::turso::connection::TursoDbConnection;
use tempdir::TempDir;
use yaml_rust2::{Yaml, YamlLoader};

pub async fn generate_db() -> (TursoDbConnection, TempDir) {
    let tempdir = TempDir::new("homelab-mgr-tests").expect("Failed to create db's tempdir");

    let path = tempdir.path().join("test.db");

    let db_connection = TursoDbConnection::new(path.to_string_lossy().to_string())
        .await
        .expect("Failed to create database");

    return (db_connection, tempdir);
}

pub fn load_yaml(yaml: &str) -> Yaml {
    let documents = YamlLoader::load_from_str(yaml)
        .map_err(|e| e.to_string())
        .expect("Invalid YAML");

    let yaml_parsed = documents.first().expect("No documents");

    return yaml_parsed.clone();
}
