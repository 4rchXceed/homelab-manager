use crate::{
    common::net::init::init_rustls, file_server::rclone::check_requirements::check_requirements,
};

pub mod common;
pub mod database;
pub mod file_server;
pub mod logger;
pub mod net;

/// Initiate the dependencies for the infrastructure layer.
pub fn init_app() -> Result<(), String> {
    init_rustls();
    check_requirements()?;

    return Ok(());
}
