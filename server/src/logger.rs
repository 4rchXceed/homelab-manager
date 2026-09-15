use chrono::Local;

use crate::{context::CommandContext, protocol::message::FromAgentMessage};

pub fn log(message: &str, level: &str, context: &CommandContext) {
    let time = Local::now();
    let message = format!("[{}] at {}: {}", level, time.format("%H:%M:%S"), message);
    context.print(message.as_str());
}

pub fn log_info(message: &str, context: &CommandContext) {
    log(message, "INFO", context);
}

pub fn log_warn(message: &str, context: &CommandContext) {
    log(message, "WARN", context);
}

pub fn log_error(message: &str, context: &CommandContext) {
    log(message, "ERROR", context);
}

// Specific case errors:

pub fn log_recv_mismatch(context: &CommandContext, expected: &str) {
    log_error(
        format!(
            "Wanted FromAgentMessage::{} but got another message (agent sending back)",
            expected
        )
        .as_str(),
        context,
    );
}
