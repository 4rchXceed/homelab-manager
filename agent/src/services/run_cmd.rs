use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use tempdir::TempDir;
use thiserror::Error;
use tokio::fs::{File, set_permissions};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::context::AgentContext;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Failed to create temporary directory: {0}")]
    TempDirCreationError(String),
    #[error("Failed to create script bash file: {0}")]
    FileCreationError(String),
    #[error("Failed to write into bash file: {0}")]
    FileWriteError(String),
    #[error("Failed to set file permissions: {0}")]
    SetFilePermissionsError(String),
    #[error("Service folder not found: {0}, please check the file server status")]
    ServiceFolderNotFoundError(String),
    #[error("Failed to run command (return code: {1}): {0}")]
    RunningError(String, i32),
    #[error("Failed to run command in docker (return code: {1}): {0}")]
    RunningDockerError(String, i32),
    #[error("Failed to create the command: {0}")]
    CreateCommandError(String),
    #[error("Could not make the path absolute: {0}")]
    CouldNotMakeAbsolutePath(String),
}

pub async fn run_commands(
    commands: Vec<String>,
    service: String,
    sandboxed: bool,
    context: AgentContext,
    image: Option<String>,
) -> Result<(), CommandError> {
    let script_dir = TempDir::new("homelab_mgr_agent")
        .map_err(|e| CommandError::TempDirCreationError(e.to_string()))?;
    let script_path = script_dir.path().join("script.sh");

    let mut file = File::create(&script_path)
        .await
        .map_err(|e| CommandError::FileCreationError(e.to_string()))?;

    let script_content = format!("#!/bin/bash\nset -e\n{}", commands.join("\n"));

    file.write_all(script_content.as_bytes())
        .await
        .map_err(|e| CommandError::FileWriteError(e.to_string()))?;

    set_permissions(&script_path, Permissions::from_mode(0o755))
        .await
        .map_err(|e| CommandError::SetFilePermissionsError(e.to_string()))?;

    let folder_path = {
        let config = context.config.read().await;

        Path::new(&config.service_folder).join(service)
    };

    if !folder_path.exists() {
        return Err(CommandError::ServiceFolderNotFoundError(
            folder_path.to_string_lossy().to_string(),
        ));
    }

    if sandboxed {
        let default_image = {
            let config = context.config.read().await;

            config.sandbox_base_image.clone()
        };

        let image = image.unwrap_or(default_image);

        let absolute_folder_path = folder_path
            .canonicalize()
            .map_err(|e| CommandError::CouldNotMakeAbsolutePath(e.to_string()))?;

        let output = Command::new("docker")
            .arg("run")
            .arg("--rm")
            .arg("-v")
            .arg(format!("{}:/script", script_dir.path().to_string_lossy()))
            .arg("-v")
            .arg(format!(
                "{}:/service",
                absolute_folder_path.to_string_lossy()
            ))
            .arg(image)
            .arg("bash")
            .arg("/script/script.sh")
            .output()
            .await
            .map_err(|e| CommandError::CreateCommandError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            return Err(CommandError::RunningDockerError(
                stderr.to_string(),
                output.status.code().unwrap_or(-1),
            ));
        } else {
            return Ok(());
        }
    } else {
        let output = Command::new("bash")
            .arg(script_path)
            .current_dir(folder_path)
            .output()
            .await
            .map_err(|e| CommandError::CreateCommandError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            return Err(CommandError::RunningError(
                stderr.to_string(),
                output.status.code().unwrap_or(-1),
            ));
        } else {
            return Ok(());
        }
    }
}
