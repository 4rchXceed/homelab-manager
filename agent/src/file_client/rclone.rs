use std::path::Path;

use log::{info, trace};
use tokio::process::Command;

use crate::file_client::errors::RCloneCommandError;

pub async fn sync_rclone(
    address: String,
    path: String,
    auth: String,
    cert_file: String,
) -> Result<(), RCloneCommandError> {
    info!("Syncing files from {} to {} using rclone...", address, path);

    if !Path::new(&path).exists() {
        info!("Services directory {} does not exist. Creating it...", path);

        std::fs::create_dir_all(&path)
            .map_err(|e| RCloneCommandError::DirectoryCreationError(e.to_string()))?;
    }

    let mut command = Command::new("rclone");
    command.arg("copy");
    command.arg("--ignore-times");
    command.arg(":http:/");
    command.arg("--http-url");
    command.arg(format!("https://{}@{}", auth, address));
    command.arg(path.clone());
    #[cfg(not(target_os = "windows"))]
    command.env("SSL_CERT_FILE", cert_file);
    #[cfg(target_os = "windows")]
    command.env("RCLONE_CA_CERT", cert_file); // WHY IS WINDOWS ALREADY DOING THIS DIFFERENTLY :(

    trace!(
        "Running rclone: rclone copy --ignore-times :http:/ --http-url https://CENSORED@{} {}",
        address, path
    );

    let result = command
        .output()
        .await
        .map_err(|e| RCloneCommandError::CommandExecutionError(e.to_string()))?;

    if !result.status.success() {
        return Err(RCloneCommandError::CommandExecutionError(
            String::from_utf8_lossy(&result.stderr).to_string(),
        ));
    }

    info!("Syncing completed successfully!");

    return Ok(());
}
