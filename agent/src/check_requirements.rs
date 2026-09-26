use which::which;

use crate::errors::RequirementMissing;

pub fn check_requirements() -> Result<(), RequirementMissing> {
    if which("rclone").is_err() {
        return Err(RequirementMissing::RCloneNotFound);
    }

    return Ok(());
}
