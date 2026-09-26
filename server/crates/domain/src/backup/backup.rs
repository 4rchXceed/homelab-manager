use config::service::backup::config::{BackupConfig, BackupType};

pub struct Backup {
    pub id: String,
    pub backup_type: BackupType,
    pub max_size: usize,
    pub max_age: usize,
    pub schedule_every: usize,
    pub no_backup_on_creation: bool,
    pub last_backup_time: Option<u64>,
}

impl Backup {
    pub fn from_config_and_db(config: BackupConfig, last_backup_time: Option<u64>) -> Self {
        return Self {
            id: config.id,
            backup_type: config.backup_type,
            max_size: config.max_size,
            max_age: config.max_age,
            schedule_every: config.schedule,
            no_backup_on_creation: config.no_backup_on_creation,
            last_backup_time: last_backup_time,
        };
    }
}
