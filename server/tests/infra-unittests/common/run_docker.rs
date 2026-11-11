#[cfg(not(windows))]
use std::fs::Permissions;
#[cfg(not(windows))]
use std::os::unix::fs::PermissionsExt;

use tempdir::TempDir;
use tokio::fs::File;
#[cfg(not(windows))]
use tokio::fs::set_permissions;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub async fn run_commands_in_docker(
    folder_path: String,
    commands: Vec<String>,
    image: Option<String>,
) -> String {
    let script_dir = TempDir::new("homelab_mgr_agent").expect("Failed to create temp dir");

    #[cfg(windows)]
    let script_path = script_dir.path().join(match sandboxed {
        true => "script.sh",
        false => "script.bat",
    });
    #[cfg(not(windows))]
    let script_path = script_dir.path().join("script.sh");

    let mut file = File::create(&script_path)
        .await
        .expect("Failed to create script file");

    let script_content = format!("#!/bin/bash\nset -e\ncd /service\n{}", commands.join("\n"));

    file.write_all(script_content.as_bytes())
        .await
        .expect("Failed to write to script file");

    #[cfg(not(windows))]
    set_permissions(&script_path, Permissions::from_mode(0o755))
        .await
        .expect("Failed to set script file permissions");

    let default_image = "debian:bookworm".to_string();

    let image = image.unwrap_or(default_image);

    // let relative_folder_path = folder_path
    //     .canonicalize()
    //     .map_err(|e| CommandError::CouldNotMakeAbsolutePath(e.to_string()))?;

    let relative_folder_path = if folder_path.starts_with("./") || folder_path.starts_with("/") {
        folder_path
    } else {
        format!("./{}", folder_path)
    };

    let output = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(format!("{}:/script", script_dir.path().to_string_lossy()))
        .arg("-v")
        .arg(format!("{}:/service", relative_folder_path))
        .arg(image)
        .arg("bash")
        .arg("/script/script.sh")
        .output()
        .await
        .expect("Failed to run command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        println!("Command crashed!");
        println!("Command stdout: {stdout}");
        panic!("Failed to run command: {stderr}");
    } else {
        return String::from_utf8_lossy(&output.stdout).to_string();
    }
}
