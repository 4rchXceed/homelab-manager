use log::{Level, SetLoggerError};

pub fn init_logger(level: Level) -> Result<(), SetLoggerError> {
    return simple_logger::init_with_level(level);
}
