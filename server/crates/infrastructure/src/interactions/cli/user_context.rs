use application::interactions::repositories::user_context::UserContext;
use async_trait::async_trait;
use log::{error, info};
use std::io::{self, Write};

pub struct CliUserContext {}

#[async_trait]
impl UserContext for CliUserContext {
    async fn print(&self, message: &str) {
        info!("{}", message);
    }

    async fn error_print(&self, message: &str) {
        error!("Error: {}", message);
    }

    async fn prompt(&self, message: &str) -> Result<String, String> {
        print!("{}: ", message);
        io::stdout()
            .flush()
            .map_err(|e| format!("Failed to flush stdout: {e}"))?;

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("Failed to read stdin: {e}"))?;

        return Ok(input.trim().to_string());
    }
}
