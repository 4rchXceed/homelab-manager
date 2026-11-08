use std::sync::Arc;
use std::time::Duration;

use domain::agent::protocol::message::{AgentToServerMsg, ServerToAgentMsg};
use log::trace;

use crate::net::repositories::agent_connection::AgentConnection;

pub struct ReloadService {}

impl ReloadService {
    pub fn new() -> Self {
        return Self {};
    }

    pub async fn reload_service(
        &self,
        service_id: String,
        net_agent: Arc<dyn AgentConnection>,
        commands: Vec<String>,
        timeout: u64,
    ) -> Result<(), String> {
        trace!(
            "Reloading service {service_id} with {} commands, and a timeout of {timeout} seconds",
            commands.len()
        );

        let result = net_agent
            .send_pingpong(
                ServerToAgentMsg::RunCommands {
                    service: service_id,
                    commands: commands,
                },
                Duration::from_secs(timeout),
            )
            .await;

        match result {
            Ok(msg) => match msg {
                AgentToServerMsg::RunCommandsResponse {
                    success,
                    error_message,
                } => {
                    if success {
                        trace!("Service reloaded successfully!");
                        Ok(())
                    } else {
                        Err(error_message
                            .unwrap_or(String::from("Unknown error while reloading service")))
                    }
                }
                _ => Err(String::from("Unexpected response from agent")),
            },
            Err(e) => Err(e),
        }
    }
}
