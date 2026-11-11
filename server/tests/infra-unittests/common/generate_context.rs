use application::interactions::repositories::user_context::UserContext;
use async_trait::async_trait;
use log::{error, info};

pub struct TestContext {
    input: String,
}

impl TestContext {
    pub fn new(input: String) -> Self {
        Self { input }
    }
}

#[async_trait]
impl UserContext for TestContext {
    async fn print(&self, message: &str) {
        info!("{message}");
    }

    async fn error_print(&self, message: &str) {
        error!("{message}");
    }

    async fn prompt(&self, _message: &str) -> Result<String, String> {
        return Ok(self.input.clone());
    }
}

pub struct PanicContext {}

impl PanicContext {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl UserContext for PanicContext {
    async fn print(&self, message: &str) {
        info!("{message}");
    }

    async fn error_print(&self, message: &str) {
        error!("{message}");
    }

    async fn prompt(&self, _message: &str) -> Result<String, String> {
        panic!("Test failed: asking user input isn't allowed");
    }
}
