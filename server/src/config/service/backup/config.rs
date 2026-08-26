use yaml_rust2::Yaml;

use crate::{config::config::ConfigError, utils::fs::parse_file_size};

#[derive(Debug)]
pub enum BackupType {
    Full,
    Incremental,
}

#[derive(Debug)]
pub struct BackupConfig {
    pub id: String,
    pub backup_type: BackupType,
    pub max_size: usize,
    pub max_age: usize,
    pub schedule: usize,
    pub no_backup_on_creation: bool,
}

impl BackupConfig {
    pub fn from_yaml(yaml: &Yaml) -> Result<Self, ConfigError> {
        let id = yaml["id"]
            .as_str()
            .ok_or(ConfigError::BackupConfigIdMissing)?;
        let backup_type_str = yaml["type"]
            .as_str()
            .ok_or(ConfigError::BackupConfigBackupTypeMissing(String::from(id)))?;
        let max_size_raw = yaml["max_size"]
            .as_str()
            .ok_or(ConfigError::BackupConfigMaxSizeMissing(String::from(id)))?;
        let max_age_raw = yaml["max_age"]
            .as_str()
            .ok_or(ConfigError::BackupConfigMaxAgeMissing(String::from(id)))?;
        let schedule_raw = yaml["schedule"]
            .as_str()
            .ok_or(ConfigError::BackupConfigScheduleMissing(String::from(id)))?;
        let max_size = parse_file_size(max_size_raw);
        if max_size.is_err() {
            return Err(ConfigError::BackupConfigMaxSizeParseError(
                String::from(id),
                max_size.err().unwrap(),
            ));
        }
        let max_size = max_size.unwrap();

        let max_age = parse_file_size(max_age_raw);
        if max_age.is_err() {
            return Err(ConfigError::BackupConfigMaxSizeParseError(
                String::from(id),
                max_age.err().unwrap(),
            ));
        }
        let max_age = max_age.unwrap();

        let schedule = parse_file_size(schedule_raw);
        if schedule.is_err() {
            return Err(ConfigError::BackupConfigMaxSizeParseError(
                String::from(id),
                schedule.err().unwrap(),
            ));
        }
        let schedule = schedule.unwrap();

        let no_backup_on_creation = yaml["no_backup_on_creation"].as_bool().unwrap_or(false);

        let backup_type = match backup_type_str {
            "full" => BackupType::Full,
            "incremental" => BackupType::Incremental,
            _ => {
                return Err(ConfigError::BackupConfigBackupTypeWrong(String::from(id)));
            }
        };

        return Ok(Self {
            id: String::from(id),
            backup_type,
            max_size,
            max_age,
            schedule,
            no_backup_on_creation,
        });
    }
}
