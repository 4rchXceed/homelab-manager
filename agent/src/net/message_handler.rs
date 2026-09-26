use std::{sync::Arc, time::Duration};

use log::{error, trace};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{RwLock, RwLockWriteGuard},
    time::sleep,
};
use tokio_rustls::client::TlsStream;

use crate::{
    commands::handle_command::handle_command,
    context::AgentContext,
    net::{message::FromServerMessage, send_queue::SendQueue},
};

pub struct MessageHandler;

impl MessageHandler {
    async fn wait_until(queue: SendQueue) {
        while queue.queue.read().await.is_empty() {
            sleep(Duration::from_millis(100)).await;
        }
    }

    async fn handle_send(stream: Arc<RwLock<TlsStream<TcpStream>>>, to_queue: SendQueue) {
        let mut stream = stream.write().await;

        for message in to_queue.queue.write().await.drain(..) {
            let serialized_message = serde_json::to_string(&message);

            match serialized_message {
                Ok(serialized_message) => {
                    trace!(
                        "Sending message: {}...",
                        serialized_message.chars().take(100).collect::<String>()
                    );

                    let bytes = serialized_message.as_bytes();

                    let len = bytes.len() as u32;
                    let len_bytes = len.to_be_bytes();

                    if let Err(e) = stream.write_all(&len_bytes).await {
                        error!("Failed to send message length: {}", e);
                        continue;
                    }

                    if let Err(e) = stream.write_all(bytes).await {
                        error!("Failed to send message: {}", e);
                        continue;
                    }
                }
                Err(e) => {
                    error!("Failed to serialize message: {}", e);
                }
            }
        }
    }

    async fn handle_recv(
        len: u32,
        stream: &mut RwLockWriteGuard<'_, TlsStream<TcpStream>>,
        context: AgentContext,
        mut queue: SendQueue,
    ) {
        let mut message_buf = vec![0u8; len as usize];

        if let Err(e) = stream.read_exact(&mut message_buf).await {
            error!("Failed to read message: {}", e);
            return;
        }

        let message_str = String::from_utf8_lossy(&message_buf);

        match serde_json::from_str::<FromServerMessage>(&message_str) {
            Ok(message) => {
                trace!(
                    "Receiving message: {}... (id: {})",
                    message_str.chars().take(100).collect::<String>(),
                    message.id
                );

                queue.msg_id = Some(message.id.clone());

                tokio::spawn(async move {
                    handle_command(message, context.clone(), queue).await;
                });
            }
            Err(e) => {
                error!("Failed to deserialize message: {}", e);
            }
        }
    }

    pub async fn handle_messages(
        stream_locked: Arc<RwLock<TlsStream<TcpStream>>>,
        to_queue: SendQueue,
        context: AgentContext,
    ) {
        loop {
            let mut buf = [0u8; 4];

            let mut stream = stream_locked.write().await;

            tokio::select! {
                _ = Self::wait_until(to_queue.clone()) => {
                    drop(stream);

                    Self::handle_send(stream_locked.clone(), to_queue.clone()).await;
                }
                len = stream.read(&mut buf) => {
                    match len {
                        Ok(_) => Self::handle_recv(u32::from_be_bytes(buf), &mut stream, context.clone(), to_queue.clone()).await,
                        Err(e) => {
                            error!("Failed to handle received message: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    }
}
