use std::time::Duration;

pub const TEMP_DIR_PREFIX: &str = "homelab-manager-";
pub const DEFAULT_CONFIG_FILE_PATH: &str = "./config/config.yaml";
pub const DEFAULT_SERVICES_FOLDER: &str = "services";
pub const DEFAULT_DATABASE_FILE_PATH: &str = "database.db";
pub const DEFAULT_UNIX_SOCKET_PATH: &str = "/tmp/homelab-manager.sock";
pub const DEFAULT_SERVICE_DESCRIPTION: &str = "No description provided.";
pub const DEFAULT_BASH_GENERATOR_TIMEOUT: &str = "60s";
pub const DEFAULT_STARTUP_TIMEOUT: &str = "120s";
pub const DEFAULT_KEEPALIVE_INTERVAL: &str = "5s";
pub const DEFAULT_BACKUP_CHECK_INTERVAL: &str = "1m";
pub const DEFAULT_SERVER_PORT: i64 = 4398;
pub const DEFAULT_FILE_SERVER_PORT: i64 = 4399;
pub const DEFAULT_BACKUP_TRANSFER_PORT: i64 = 4397;
pub const MAX_PORT_NBR: i64 = 65535;
pub const DEFAULT_FALLBACK_STORAGE: &str = "fallback";
pub const SEND_SLEEP_DELAY: u64 = 50; // in milliseconds
pub const DEFAULT_SMALL_OPERATION_TIMEOUT: Duration = Duration::from_secs(5);
pub const DEFAULT_LOG_LEVEL: &str = "INFO";
pub const DEFAULT_FILE_SERVER_TIMEOUT: Duration = Duration::from_secs(1000);
pub const DEFAULT_FILE_SERVER_LOCAL_PORT: u16 = 4339;
