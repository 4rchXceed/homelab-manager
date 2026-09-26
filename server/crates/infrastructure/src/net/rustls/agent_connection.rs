use std::{sync::Arc, time::Duration};

use application::net::repositories::agent_connection::{AgentConnection, MessageId};
use async_trait::async_trait;
use domain::agent::protocol::message::{AgentToServerMsg, ServerToAgentMsg};
use log::error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::RwLock,
};
use tokio_rustls::server::TlsStream;
use uuid::Uuid;

use crate::net::rustls::message::agent::{FromAgentMessage, ToAgentMessage};

pub struct RustTlsAgentConnection {
    stream: Arc<RwLock<TlsStream<TcpStream>>>,
    send_queue: Arc<RwLock<Vec<ToAgentMessage>>>,
    receive_queue: Arc<RwLock<Vec<FromAgentMessage>>>,
}

impl RustTlsAgentConnection {
    pub fn new(stream: TlsStream<TcpStream>) -> Self {
        return Self {
            stream: Arc::new(RwLock::new(stream)),
            send_queue: Arc::new(RwLock::new(Vec::new())),
            receive_queue: Arc::new(RwLock::new(Vec::new())),
        };
    }

    async fn wait_for_message(
        send_queue: Arc<RwLock<Vec<ToAgentMessage>>>,
    ) -> Result<ToAgentMessage, String> {
        // DO NOT LOCK THE RECEIVE QUEUE HERE
        while send_queue.read().await.is_empty() {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        let mut receive_queue = send_queue.write().await;

        return Ok(receive_queue.remove(0));
    }

    async fn send_raw_message(
        stream: Arc<RwLock<TlsStream<TcpStream>>>,
        message: ToAgentMessage,
    ) -> Result<(), String> {
        let mut stream = stream.write().await;

        let message_bytes = serde_json::to_vec(&message).map_err(|e| e.to_string())?;
        let length = message_bytes.len() as u32;
        let length_bytes = length.to_be_bytes();

        stream
            .write_all(&length_bytes)
            .await
            .map_err(|e| e.to_string())?;
        stream
            .write_all(&message_bytes)
            .await
            .map_err(|e| e.to_string())?;

        return Ok(());
    }

    pub async fn handle_raw_message(
        receive_queue: Arc<RwLock<Vec<FromAgentMessage>>>,
        message_bytes: Vec<u8>,
    ) -> Result<(), String> {
        let message: Result<FromAgentMessage, serde_json::Error> =
            serde_json::from_slice(&message_bytes);

        match message {
            Ok(msg) => {
                let mut receive_queue = receive_queue.write().await;

                receive_queue.push(msg);

                Ok(())
            }
            Err(e) => Err(format!("Error deserializing message: {}", e)),
        }
    }
}

#[async_trait]
impl AgentConnection for RustTlsAgentConnection {
    async fn close(&self) -> Result<(), String> {
        let mut stream = self.stream.write().await;

        stream.shutdown().await.map_err(|e| e.to_string())?;

        return Ok(());
    }

    async fn start_processing(&self) {
        // From now on only communicate with the agent using the send_queue and receive_queue, since stream's locked
        let stream_locked = self.stream.clone();
        let send_queue = self.send_queue.clone();
        let receive_queue = self.receive_queue.clone();

        tokio::spawn(async move {
            loop {
                let mut buf = [0u8; 4];
                let mut stream = stream_locked.write().await;
                tokio::select! {
                    _ = stream.read(&mut buf) => {
                        let length = u32::from_be_bytes(buf) as usize;
                        let mut message_buf = vec![0u8; length];
                        match stream.read_exact(&mut message_buf).await.map_err(|e| e.to_string()) {
                            Ok(_) => {
                                if let Err(e) = Self::handle_raw_message(receive_queue.clone(), message_buf).await {
                                    error!("Error handling raw message: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Error reading from stream: {}", e);
                                break;
                            }
                        }
                    },
                    message = Self::wait_for_message(send_queue.clone()) => {
                        drop(stream); // Release the lock on the stream
                        match message {
                            Ok(msg) => {
                                match Self::send_raw_message(stream_locked.clone(), msg).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        error!("Error sending message: {}", e);
                                        break;
                                    }
                                }
                            },
                            Err(e) => {
                                error!("Error waiting for message: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    async fn send_message(&self, message: ServerToAgentMsg) -> Result<MessageId, String> {
        let message_id = Uuid::new_v4().to_string();

        self.send_queue
            .write()
            .await
            .push(ToAgentMessage::from_domain_msg(message_id.clone(), message));

        return Ok(message_id);
    }

    async fn receive_message(&self, id: MessageId) -> Result<AgentToServerMsg, String> {
        let mut receive_queue = self.receive_queue.write().await;

        if let Some(pos) = receive_queue.iter().position(|msg| msg.id == id) {
            let msg = receive_queue.remove(pos);

            return Ok(msg.to_domain_msg());
        } else {
            return Err(format!("Message with id {} not found", id));
        }
    }

    async fn send_pingpong(
        &self,
        message: ServerToAgentMsg,
        timeout: Duration,
    ) -> Result<AgentToServerMsg, String> {
        let message_id = self.send_message(message).await?;

        let start_time = tokio::time::Instant::now();

        loop {
            if start_time.elapsed() > timeout {
                return Err("Timeout waiting for pingpong response".to_string());
            }

            match self.receive_message(message_id.clone()).await {
                Ok(response) => {
                    return Ok(response);
                }
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }
}
