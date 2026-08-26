use std::path::MAIN_SEPARATOR_STR;

use tempdir::TempDir;

use crate::consts::TEMP_DIR_PREFIX;

pub type TempDirCreation = (String, TempDir);

pub fn create_temp_dir() -> Result<TempDirCreation, String> {
    let temp_dir = TempDir::new(TEMP_DIR_PREFIX);
    if temp_dir.is_ok() {
        let temp_dir = temp_dir.unwrap();
        let path = temp_dir.path().as_os_str().to_str();
        if path.is_some() {
            return Ok((String::from(path.unwrap()), temp_dir));
        } else {
            return Err(String::from(
                "Should NOT be happening: TempDir path INVALID!",
            ));
        }
    } else {
        return Err(String::from("Temp dir creation failed :("));
    }
}

pub fn join_path(path1: &str, path2: &str) -> String {
    let sep = MAIN_SEPARATOR_STR;
    let mut path = String::from(path1);
    if !path.ends_with(sep) {
        path.push_str(sep);
    }
    path.push_str(path2);
    return path;
}

#[derive(Debug, Clone)]
pub enum TimeParseError {
    NoNumberBeforeUnit,
    GeneralParseError(String),
}

#[derive(Debug, Clone)]
pub enum FileSizeParseError {
    NoNumberBeforeUnit,
    InvalidUnit,
}

pub fn parse_time(value: &str) -> Result<usize, TimeParseError> {
    const TIME_MAP: [(&str, usize); 7] = [
        ("y", 365 * 24 * 60 * 60),
        ("mo", 30 * 24 * 60 * 60),
        ("w", 7 * 24 * 60 * 60),
        ("d", 24 * 60 * 60),
        ("h", 60 * 60),
        ("m", 60),
        ("s", 1),
    ];
    let value = value.trim().to_lowercase();

    let mut total: usize = 0;

    let mut remaining = value.as_str();
    let mut stop = false;
    while !remaining.is_empty() && !stop {
        let mut found = false;
        for (unit, multiplier) in TIME_MAP.iter() {
            if remaining.ends_with(unit) {
                let remaining_str = remaining.trim_end_matches(unit).trim();
                let mut i = remaining_str.len();
                while i > 0 && remaining_str[i - 1..i].chars().all(|c| c.is_digit(10)) {
                    i -= 1;
                }
                let number_str = remaining_str[i..].trim();
                if number_str.is_empty() {
                    return Err(TimeParseError::NoNumberBeforeUnit);
                }
                let number: usize = number_str
                    .parse()
                    .map_err(|_| TimeParseError::NoNumberBeforeUnit)?;
                total += number * multiplier;
                remaining = remaining_str[..i].trim();
                found = true;
                break;
            }
        }
        if !found {
            stop = true;
        }
    }

    if !remaining.is_empty() {
        return Err(TimeParseError::GeneralParseError(String::from(value)));
    }

    return Ok(total);
}

pub fn parse_file_size(value: &str) -> Result<usize, FileSizeParseError> {
    const SIZE_MAP: [(&str, usize); 5] = [
        ("b", 1),
        ("kb", 1024),
        ("mb", 1024 * 1024),
        ("gb", 1024 * 1024 * 1024),
        ("tb", 1024 * 1024 * 1024 * 1024),
    ];
    let value = value.trim().to_lowercase();

    for (unit, multiplier) in SIZE_MAP.iter() {
        if value.ends_with(unit) {
            let number_str = value.trim_end_matches(unit).trim();
            let number: usize = number_str
                .parse()
                .map_err(|_| FileSizeParseError::NoNumberBeforeUnit)?;
            return Ok(number * multiplier);
        }
    }
    return Err(FileSizeParseError::InvalidUnit);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parse_time() {
        let test_cases = vec![
            ("1y", 365 * 24 * 60 * 60),
            ("2mo", 2 * 30 * 24 * 60 * 60),
            ("3w", 3 * 7 * 24 * 60 * 60),
            ("4d", 4 * 24 * 60 * 60),
            ("5h", 5 * 60 * 60),
            ("6m", 6 * 60),
            ("7s", 7),
            (
                "1y2mo3w4d5h6m7s",
                365 * 24 * 60 * 60
                    + 2 * 30 * 24 * 60 * 60
                    + 3 * 7 * 24 * 60 * 60
                    + 4 * 24 * 60 * 60
                    + 5 * 60 * 60
                    + 6 * 60
                    + 7,
            ),
        ];

        for (input, expected) in test_cases {
            let result = parse_time(input);
            println!("Testing parse_time('{}') => {:?}", input, result);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), expected);
        }
    }
}
