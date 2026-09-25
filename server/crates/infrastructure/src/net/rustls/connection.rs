use std::sync::Arc;

use application::net::repositories::net_connection::NetworkConnection;
use async_trait::async_trait;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::RwLock,
};
use tokio_rustls::server::TlsStream;

pub struct RustTlsNetworkConnection {
    stream: Arc<RwLock<TlsStream<TcpStream>>>,
}

impl RustTlsNetworkConnection {
    pub fn new(stream: TlsStream<TcpStream>) -> Self {
        return Self {
            stream: Arc::new(RwLock::new(stream)),
        };
    }
}

#[async_trait]
impl NetworkConnection for RustTlsNetworkConnection {
    async fn send_raw(&self, data: Vec<u8>) -> Result<(), String> {
        let mut stream = self.stream.write().await;

        stream.write_all(&data).await.map_err(|e| e.to_string())?;

        return Ok(());
    }

    async fn recv_raw(&self, length: usize) -> Result<Vec<u8>, String> {
        let mut stream = self.stream.write().await;

        let mut buffer = vec![0u8; length];
        stream
            .read_exact(&mut buffer)
            .await
            .map_err(|e| e.to_string())?;

        return Ok(buffer);
    }

    async fn recv_until(&self, delimiter: u8) -> Result<Vec<u8>, String> {
        let mut stream = self.stream.write().await;

        let mut buffer = Vec::new();
        stream
            .read_until(delimiter, &mut buffer)
            .await
            .map_err(|e| e.to_string())?;

        return Ok(buffer);
    }

    async fn close(&self) -> Result<(), String> {
        let mut stream = self.stream.write().await;

        stream.shutdown().await.map_err(|e| e.to_string())?;

        return Ok(());
    }
}
