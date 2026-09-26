use std::sync::Arc;

use config::generic::net::NetConfig;
use http_body_util::{BodyExt, Empty, Full};
use hyper::{Request, Response, body::Bytes, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use log::error;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};
use tokio_rustls::TlsAcceptor;

use crate::common::net::create_connection::create_connection;

pub struct HttpToHttpsProxy {
    http_port: u16,
    tcp_listener: Arc<RwLock<TcpListener>>,
    tls_acceptor: TlsAcceptor,
}

impl HttpToHttpsProxy {
    pub async fn new(from: u16, to: u16, netconf: NetConfig) -> Result<Self, String> {
        let (tls_acceptor, tcp_listener) = create_connection(to, netconf)
            .await
            .map_err(|e| e.to_string())?;

        return Ok(Self {
            http_port: from,
            tcp_listener: Arc::new(RwLock::new(tcp_listener)),
            tls_acceptor: tls_acceptor,
        });
    }

    pub fn start(&mut self) {
        let http_port = self.http_port;
        let tls_acceptor = self.tls_acceptor.clone();
        let tcp_listener = self.tcp_listener.clone();

        tokio::spawn(
            async move { Self::threaded_server(http_port, tls_acceptor, tcp_listener).await },
        );
    }

    async fn process_request(
        r: Request<impl hyper::body::Body>,
        http_port: u16,
    ) -> Result<Response<Full<Bytes>>, String> {
        // URI:
        let uri = r.uri().path_and_query().map(|x| x.as_str()).unwrap_or("/");

        // Basic AUTH:
        let auth_header = r
            .headers()
            .get("Authorization")
            .map(|x| x.to_str().unwrap_or(""));
        let content_type_header = r
            .headers()
            .get("Content-Type")
            .map(|x| x.to_str().unwrap_or(""));

        // Connect to the HTTP server:
        let stream = TcpStream::connect(format!("localhost:{http_port}"))
            .await
            .map_err(|e| e.to_string())?;

        let io = TokioIo::new(stream);

        // Http 2 handshake
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
            .await
            .map_err(|e| e.to_string())?;

        tokio::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("HTTP/2 connection error: {:?}", err);
            }
        });

        // Request to the HTTP server:
        let request = Request::builder()
            .method(r.method())
            .uri(uri)
            .header("Authorization", auth_header.unwrap_or(""))
            .header("Content-Type", content_type_header.unwrap_or(""))
            .header("Host", format!("localhost:{http_port}"))
            .body(Empty::<Bytes>::new())
            .map_err(|e| e.to_string())?;

        // Stream the response
        let mut response = sender
            .send_request(request)
            .await
            .map_err(|e| e.to_string())?;

        let mut buffer = Vec::new();

        while let Some(next) = response.frame().await {
            let frame = next.map_err(|e| e.to_string())?;
            if let Some(chunk) = frame.data_ref() {
                buffer.extend_from_slice(chunk);
            }
        }

        let content_type_response = response
            .headers()
            .get("Content-Type")
            .map(|x| x.to_str().unwrap_or(""))
            .unwrap_or("");

        // Return
        Ok(Response::builder()
            .status(response.status())
            .header("Content-Type", content_type_response)
            .body(Full::from(buffer))
            .map_err(|e| e.to_string())?)
    }

    async fn threaded_server(
        http_port: u16,
        tls_acceptor: TlsAcceptor,
        tcp_listener: Arc<RwLock<TcpListener>>,
    ) -> ! {
        let listener = tcp_listener.read().await;
        loop {
            let accept = listener.accept().await;
            match accept {
                Ok((tcp_stream, _)) => {
                    let tls_acceptor = tls_acceptor.clone();
                    tokio::spawn(async move {
                        match Self::handle_new_request(http_port, tcp_stream, tls_acceptor).await {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Error handling new request: {e}");
                            }
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {e}");
                }
            }
        }
    }

    async fn handle_new_request(
        http_port: u16,
        tcp_stream: TcpStream,
        tls_acceptor: TlsAcceptor,
    ) -> Result<(), String> {
        let tls_stream = tls_acceptor
            .accept(tcp_stream)
            .await
            .map_err(|e| format!("Error accepting TLS connection: {e}"))?;
        let service = service_fn(move |r| Self::process_request(r, http_port));

        let io = TokioIo::new(tls_stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                error!("Error serving connection: {err}");
            }
        });

        return Ok(());
    }
}
