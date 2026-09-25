use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    fs::{File, read_to_string},
    io::AsyncWriteExt,
};

#[derive(Debug, Error)]
pub enum DatabaseOpenError {
    #[error("Failed to open db file: {0}")]
    FileOpen(std::io::Error),
    #[error("Failed to parse db file: {0}")]
    Parse(serde_json::error::Error),
    #[error("Failed to save the file, since it's the first time and it doesn't exists: {0}")]
    FirstTimeSaveError(DatabaseSaveError),
}

#[derive(Debug, Error)]
pub enum DatabaseSaveError {
    #[error("Failed to convert db to json: {0}")]
    Serialize(serde_json::error::Error),
    #[error("Failed to open file: {0}")]
    FileOpenError(std::io::Error),
    #[error("Failed to write to file: {0}")]
    FileWriteError(std::io::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Database {
    pub reverse_api_key: Option<String>,
}

impl Database {
    pub async fn load(file: String) -> Result<Self, DatabaseOpenError> {
        if !Path::new(&file).exists() {
            Database::default()
                .save(file.clone())
                .await
                .map_err(|e| DatabaseOpenError::FirstTimeSaveError(e))?;
        }

        let content = read_to_string(file)
            .await
            .map_err(|e| DatabaseOpenError::FileOpen(e))?;

        return serde_json::from_str(&content).map_err(|e| DatabaseOpenError::Parse(e));
    }

    pub async fn save(&self, file: String) -> Result<(), DatabaseSaveError> {
        let json = serde_json::to_string(self).map_err(|e| DatabaseSaveError::Serialize(e))?;

        let mut file = File::create(file)
            .await
            .map_err(|e| DatabaseSaveError::FileOpenError(e))?;

        file.write(json.as_bytes())
            .await
            .map_err(|e| DatabaseSaveError::FileWriteError(e))?;

        return Ok(());
    }
}

impl Default for Database {
    fn default() -> Self {
        return Self {
            reverse_api_key: None,
        };
    }
}
