use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum TimeParseError {
    #[error("No number found before unit")]
    NoNumberBeforeUnit,
    #[error("General parse error: {0}")]
    GeneralParseError(String),
}

#[derive(Debug, Clone, Error)]
pub enum FileSizeParseError {
    #[error("Invalid number")]
    NumberInvalid,
    #[error("Invalid unit")]
    InvalidUnit,
    #[error("Negative size")]
    NegativeSize,
    #[error("Size too large")]
    SizeTooLarge,
    #[error("Size is not an integer")]
    SizeIsNotInteger,
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
        ("kb", 1024),
        ("mb", 1024 * 1024),
        ("gb", 1024 * 1024 * 1024),
        ("tb", 1024 * 1024 * 1024 * 1024),
        ("b", 1), // Put the b at the end to avoid matching b in kb, mb, gb, tb
    ];
    let value = value.trim().to_lowercase();

    for (unit, multiplier) in SIZE_MAP.iter() {
        if value.ends_with(unit) {
            let number_str = value.trim_end_matches(unit).trim();
            let number: f32 = number_str
                .parse()
                .map_err(|_| FileSizeParseError::NumberInvalid)?;
            let result = number * *multiplier as f32;
            if result < 0.0 {
                return Err(FileSizeParseError::NegativeSize);
            }
            if result > usize::MAX as f32 {
                return Err(FileSizeParseError::NegativeSize);
            }
            if result % 1.0 != 0.0 {
                return Err(FileSizeParseError::SizeIsNotInteger);
            }
            return Ok(result as usize);
        }
    }
    return Err(FileSizeParseError::InvalidUnit);
}

#[cfg(test)]
mod tests {}
