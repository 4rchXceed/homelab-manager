use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use rustls::pki_types::{IpAddr, ServerName};

use crate::{
    builders::{message_builder::ProtocolMessage, server_socket::ServerSocket},
    config::SSLConfig,
    errors::HLMCommunicationError,
};

pub type ErrorCallback = Box<dyn Fn(HLMCommunicationError) + Send + Sync>;
pub type AuthCallback = Box<dyn Fn(&str) -> bool + Send + Sync>;

pub enum ProtocolSide {
    Client,
    Server,
}

pub struct ProtocolBuilder {
    side: ProtocolSide,
    config: Option<SSLConfig>,
    on_error: Option<ErrorCallback>,
    auth_callback: Option<AuthCallback>,
    allowed_addresses: Option<Vec<String>>,
    on_accept: Option<Box<dyn Fn(ServerSocket) -> bool + Send + Sync>>,
    message_generator: Option<Box<dyn Fn() -> ProtocolMessage + Send + Sync>>,
    on_listen_callback: Option<Box<dyn Fn() + Send + Sync>>,
}

impl ProtocolBuilder {
    pub fn client() -> Self {
        ProtocolBuilder {
            side: ProtocolSide::Client,
            config: None,
            on_error: None,
            auth_callback: None,
            message_generator: None,
            allowed_addresses: None,
            on_accept: None,
            on_listen_callback: None,
        }
    }

    pub fn server() -> Self {
        ProtocolBuilder {
            side: ProtocolSide::Server,
            config: None,
            on_error: None,
            auth_callback: None,
            message_generator: None,
            allowed_addresses: None,
            on_accept: None,
            on_listen_callback: None,
        }
    }

    pub fn on_listen<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_listen_callback = Some(Box::new(callback));
        self
    }

    pub fn with_server_accept<F>(mut self, callback: F) -> Self
    where
        F: Fn(ServerSocket) -> bool + Send + Sync + 'static,
    {
        self.on_accept = Some(Box::new(callback));
        self
    }

    pub fn with_ssl_config(mut self, config: SSLConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn on_error<F>(mut self, callback: F) -> Self
    where
        F: Fn(HLMCommunicationError) + Send + Sync + 'static,
    {
        self.on_error = Some(Box::new(callback));
        self
    }

    pub fn with_auth<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.auth_callback = Some(Box::new(callback));
        self
    }

    pub fn with_protocol_message<T>(mut self, message: T) -> Self
    where
        T: Fn() -> ProtocolMessage + Send + Sync + 'static,
    {
        self.message_generator = Some(Box::new(message));
        self
    }

    pub fn with_whitelist(mut self, address: Vec<&str>) -> Self {
        self.allowed_addresses = Some(address.into_iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn connect(&mut self, address: &str, port: u16) -> Result<(), HLMCommunicationError> {
        match self.side {
            ProtocolSide::Client => {
                let config = self
                    .config
                    .as_ref()
                    .ok_or(HLMCommunicationError::SSLConfigMissing)?;
                let ssl_config = config
                    .generate_client_config()
                    .map_err(|_| HLMCommunicationError::FailedToCreateClientSSLConfig)?;
                let stream = TcpStream::connect(format!("{}:{}", address, port))
                    .map_err(|_| HLMCommunicationError::FailedToConnect)?;
                let ip = IpAddr::try_from(address)
                    .map_err(|_| HLMCommunicationError::InvalidMainAddr(String::from(address)))?;
                let client =
                    rustls::ClientConnection::new(Arc::new(ssl_config), ServerName::IpAddress(ip))
                        .map_err(|_| HLMCommunicationError::FailedToCreateClientSSLConnection)?;
                let mut tls_stream = rustls::StreamOwned::new(client, stream);
                self.process_stream_client(&mut tls_stream);
                Ok(())
            }
            ProtocolSide::Server => Err(HLMCommunicationError::LaunchedClientWhenServer)?,
        }
    }

    pub fn listen(
        &mut self,
        port: u16,
        bind_addr: Option<&str>,
    ) -> Result<(), HLMCommunicationError> {
        match self.side {
            ProtocolSide::Server => {
                let config = self
                    .config
                    .as_ref()
                    .ok_or(HLMCommunicationError::SSLConfigMissing)?;
                let ssl_config = config.generate_server_config().map_err(|_| {
                    HLMCommunicationError::FailedToCreateServerConnection(rustls::Error::General(
                        "Failed to create server connection".to_string(),
                    ))
                })?;
                let mut binding_addr = String::from("[::]");
                if let Some(addr) = bind_addr {
                    binding_addr = String::from(addr);
                }
                binding_addr.push(':');
                binding_addr.push_str(&port.to_string());
                let listener = TcpListener::bind(binding_addr);
                if listener.is_err() {
                    return Err(HLMCommunicationError::FailedToBindAddress);
                }
                let listener = listener.unwrap();
                if self.on_listen_callback.is_some() {
                    let callback = self.on_listen_callback.as_ref().unwrap();
                    callback();
                }
                for stream in listener.incoming() {
                    if stream.is_ok() {
                        let stream = stream.unwrap();
                        let ip = stream
                            .peer_addr()
                            .map_err(|_| HLMCommunicationError::FailedToGetPeerAddress)?;
                        if let Some(allowed) = &self.allowed_addresses {
                            if !allowed.contains(&ip.ip().to_string()) {
                                self.raise_error(HLMCommunicationError::UnauthorizedConnection);
                                continue;
                            }
                        }
                        let server = rustls::ServerConnection::new(Arc::new(ssl_config.clone()))
                            .map_err(|e| {
                                HLMCommunicationError::FailedToCreateServerConnection(e)
                            })?;
                        let tls_stream = rustls::StreamOwned::new(server, stream);
                        if self.on_accept.is_some() {
                            let callback = self.on_accept.as_ref().unwrap();
                            if self.message_generator.is_none() {
                                self.raise_error(HLMCommunicationError::NoProtocolMessageDefined);
                                continue;
                            }
                            let stop = callback(ServerSocket::new(
                                tls_stream,
                                self.message_generator.as_ref().unwrap()(),
                            ));
                            if stop {
                                break;
                            }
                        } else {
                            self.raise_error(HLMCommunicationError::NoAcceptCallbackDefined);
                        }
                    } else {
                        self.raise_error(HLMCommunicationError::FailedToAcceptConnection);
                    }
                }
                Ok(())
            }
            ProtocolSide::Client => Err(HLMCommunicationError::LaunchedServerWhenClient)?,
        }
    }

    fn write_response(
        &self,
        stream: &mut rustls::StreamOwned<rustls::ClientConnection, std::net::TcpStream>,
        response: Vec<u8>,
    ) {
        let len_bytes = (response.len() as u32).to_be_bytes();
        let res = stream.write_all(&len_bytes);
        if res.is_err() {
            self.raise_error(HLMCommunicationError::SendIOError(
                res.err().unwrap().to_string(),
            ));
            return;
        }

        let res = stream.write_all(&response);
        if res.is_err() {
            self.raise_error(HLMCommunicationError::SendIOError(
                res.err().unwrap().to_string(),
            ));
            return;
        }
        let res = stream.flush();
        if res.is_err() {
            self.raise_error(HLMCommunicationError::SendIOError(
                res.err().unwrap().to_string(),
            ));
            return;
        }
        let mut ack_buf = [0u8; 1];
        let res = stream.read_exact(&mut ack_buf);
        if res.is_err() {
            self.raise_error(HLMCommunicationError::SendIOError(
                res.err().unwrap().to_string(),
            ));
            return;
        }
        if ack_buf[0] != 0x01 {
            self.raise_error(HLMCommunicationError::SendIOError(
                "Did not receive acknowledgment from client".to_string(),
            ));
            return;
        }
    }

    fn process_stream_client(
        &mut self,
        stream: &mut rustls::StreamOwned<rustls::ClientConnection, std::net::TcpStream>,
    ) {
        if self.message_generator.is_none() {
            self.raise_error(HLMCommunicationError::NoProtocolMessageDefined);
            return;
        }
        let mut protocol_message = self.message_generator.as_ref().unwrap()();
        loop {
            let mut buf_length = [0u8; 4];
            match stream.read_exact(&mut buf_length) {
                Ok(_) => {
                    let length = u32::from_be_bytes(buf_length) as u32;
                    let mut buf = vec![0u8; length as usize];
                    match stream.read_exact(&mut buf) {
                        Ok(_) => {
                            let response = protocol_message.parse_from(buf);
                            if response.is_err() {
                                self.raise_error(response.as_ref().err().unwrap().clone());
                            }
                            let (response, is_end) = response.unwrap();

                            let res = stream.write_all(&[0x01]);
                            if res.is_err() {
                                self.raise_error(HLMCommunicationError::SendIOError(
                                    res.err().unwrap().to_string(),
                                ));
                            }
                            let res = stream.flush();
                            if res.is_err() {
                                self.raise_error(HLMCommunicationError::SendIOError(
                                    res.err().unwrap().to_string(),
                                ));
                            }
                            if response.is_some() {
                                self.write_response(stream, response.unwrap());
                            }
                            if is_end {
                                break;
                            }
                        }
                        Err(_) => {
                            self.raise_error(HLMCommunicationError::FailedToReadMessage);
                        }
                    }
                }
                Err(_) => {
                    self.raise_error(HLMCommunicationError::FailedToReadLength);
                }
            }
        }
    }

    fn raise_error(&self, error: HLMCommunicationError) -> HLMCommunicationError {
        if self.on_error.is_some() {
            let callback = self.on_error.as_ref().unwrap();
            callback(error.clone());
        }
        return error;
    }
}

mod test {
    use crate::builders::{
        message_parts::{
            binary::BinaryPart,
            parsable_part::{ParsedMessageParts, ProtocolMessageData},
            string::StringPart,
        },
        send::message::MessageBuilder,
    };

    use super::*;

    #[allow(dead_code)]
    pub struct HelloWorldPart {
        message: String,
    }

    impl ProtocolMessageData for HelloWorldPart {
        fn build() -> ParsedMessageParts {
            return ParsedMessageParts::new().add_part("message", StringPart::until('\n'));
        }
        fn when_parsed(parts: &ParsedMessageParts) -> Option<Self> {
            let value = parts.get("message").unwrap();
            let content = value.as_str().unwrap();
            return Some(HelloWorldPart {
                message: String::from(content),
            });
        }
    }

    #[allow(dead_code)]
    pub struct ResponsePart {
        sucess: bool,
    }

    impl ProtocolMessageData for ResponsePart {
        fn build() -> ParsedMessageParts {
            return ParsedMessageParts::new().add_part("status", BinaryPart::size(1));
        }
        fn when_parsed(parts: &ParsedMessageParts) -> Option<Self> {
            let value = parts.get("status").unwrap();
            let content = value.as_bytes().unwrap();
            return Some(ResponsePart {
                sucess: content[0] == 0x01,
            });
        }
    }

    #[allow(dead_code)]
    fn build_protocol_message() -> ProtocolMessage {
        let protocol_message = ProtocolMessage::new().add_type_part(StringPart::until('\n'));
        let protocol_message = protocol_message
            .for_type(b"hello_message", |msg| {
                let content_op = msg.parse::<HelloWorldPart>();
                if content_op.is_ok() {
                    let content = content_op.unwrap();
                    assert_eq!(content.message, "Hello, World!");
                    return MessageBuilder::new(StringPart::new("hello_response", '\n'))
                        .add_part(BinaryPart::new(&[0x01]))
                        .build();
                } else {
                    println!("Failed to parse message: {:?}", content_op.err());
                    return MessageBuilder::new(StringPart::new("hello_response", '\n'))
                        .add_part(BinaryPart::new(&[0x00]))
                        .build();
                }
            })
            .for_type(b"hello_response", |msg| {
                let content_op = msg.parse::<ResponsePart>();
                if content_op.is_ok() {
                    let content = content_op.unwrap();
                    println!("Received response: {:?}", content.sucess);
                    assert!(content.sucess);
                    return None;
                } else {
                    println!("Failed to parse message: {:?}", content_op.err());
                    return None;
                }
            })
            .with_end_message(StringPart::new("end", '\n'));
        return protocol_message;
    }

    #[allow(dead_code)]
    fn on_server_accept(mut socket: ServerSocket) -> bool {
        println!("Server accepted connection from: {:?}", socket.ip());
        socket
            .send_pingpong(
                MessageBuilder::new(StringPart::new("hello_message", '\n'))
                    .add_part(StringPart::new("Hello, World!", '\n'))
                    .build()
                    .unwrap(),
            )
            .unwrap();
        socket.end().unwrap();
        return true;
    }

    #[allow(dead_code)]
    fn build_server() -> ProtocolBuilder {
        let server = ProtocolBuilder::server()
            .on_error(|e| {
                panic!("Error: {:?}", e);
            })
            .with_ssl_config(
                SSLConfig::new_generated_server(
                    String::from("/tmp/srv_cert.crt"),
                    String::from("/tmp/srv.key"),
                    String::from("/tmp/ca_cert.crt"),
                    vec![String::from("127.0.0.1")],
                )
                .unwrap(),
            )
            .with_server_accept(on_server_accept)
            .with_protocol_message(build_protocol_message);
        return server;
    }

    #[allow(dead_code)]
    fn build_client() -> ProtocolBuilder {
        let client = ProtocolBuilder::client()
            .on_error(|e| {
                panic!("Error occurred (client): {:?}", e);
            })
            .with_ssl_config(SSLConfig::new_client(String::from("/tmp/ca_cert.crt")))
            // .with_whitelist(vec!["127.0.0.1"])
            .with_protocol_message(build_protocol_message);
        return client;
        // println!("{:?}", client.connect("192.168.1.1"));
        // panic!("Test completed");
    }

    #[test]
    fn test_server_client_communication() {
        let mut server = build_server().on_listen(|| {
            std::thread::spawn(|| {
                let mut client = build_client();
                let r = client.connect("127.0.0.1", 12345);
                if r.is_err() {
                    println!("Client failed to connect: {:?}", r.err());
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            });
        });
        let r = server.listen(12345, Some("127.0.0.1"));
        if r.is_err() {
            panic!("Server failed to listen: {:?}", r.err());
        }
    }
}
