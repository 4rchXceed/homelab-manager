use which::which;

pub fn check_requirements() -> Result<(), String> {
    if which("rclone").is_err() {
        return Err(String::from(
            "Requirement Error: rclone is not installed. Please install it",
        ));
    }

    return Ok(());
}
