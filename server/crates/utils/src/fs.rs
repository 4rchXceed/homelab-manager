use tempdir::TempDir;

use consts::TEMP_DIR_PREFIX;

pub type TempDirCreation = (String, TempDir);

pub fn create_temp_dir() -> Result<TempDirCreation, String> {
    let temp_dir = TempDir::new(TEMP_DIR_PREFIX);
    if let Ok(temp_dir) = temp_dir {
        let path = temp_dir.path().as_os_str().to_str();
        if let Some(path) = path {
            return Ok((String::from(path), temp_dir));
        } else {
            return Err(String::from(
                "Should NOT be happening: TempDir path INVALID!",
            ));
        }
    } else {
        return Err(String::from("Temp dir creation failed :("));
    }
}
