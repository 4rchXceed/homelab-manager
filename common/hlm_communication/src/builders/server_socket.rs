use std::io::{Read, Write};

use rustls::StreamOwned;

use crate::{builders::message_builder::ProtocolMessage, errors::HLMCommunicationError};

pub struct ServerSocket {
    socket: StreamOwned<rustls::ServerConnection, std::net::TcpStream>,
    message: ProtocolMessage,
}

impl ServerSocket {
    pub fn new(
        socket: StreamOwned<rustls::ServerConnection, std::net::TcpStream>,
        message: ProtocolMessage,
    ) -> Self {
        ServerSocket {
            socket: socket,
            message: message,
        }
    }

    pub fn ip(&self) -> Option<String> {
        let peer_addr = self.socket.get_ref().peer_addr();
        if peer_addr.is_ok() {
            return Some(peer_addr.unwrap().ip().to_string());
        }
        return None;
    }

    pub fn only_send(&mut self, data: Vec<u8>) -> Result<(), HLMCommunicationError> {
        let len_bytes = (data.len() as u32).to_be_bytes();
        let res = self.socket.write_all(&len_bytes);
        if res.is_err() {
            return Err(HLMCommunicationError::SendIOError(
                res.err().unwrap().to_string(),
            ));
        }
        let res = self.socket.write_all(data.as_slice());
        if res.is_err() {
            return Err(HLMCommunicationError::SendIOError(
                res.err().unwrap().to_string(),
            ));
        }
        let res = self.socket.flush();
        if res.is_err() {
            return Err(HLMCommunicationError::SendIOError(
                res.err().unwrap().to_string(),
            ));
        }

        let mut ack_buf = [0u8; 1];
        let r = self.socket.read_exact(&mut ack_buf);
        if r.is_err() {
            return Err(HLMCommunicationError::ReceiveIOError(
                r.err().unwrap().to_string(),
            ));
        }
        if ack_buf[0] != 0x01 {
            return Err(HLMCommunicationError::InvalidAckReceived);
        }
        return Ok(());
    }

    pub fn send_pingpong(&mut self, message: Vec<u8>) -> Result<(), HLMCommunicationError> {
        self.only_send(message)?;
        return self.receive();
    }

    pub fn receive(&mut self) -> Result<(), HLMCommunicationError> {
        let mut buf_length = [0u8; 4];
        match self.socket.read_exact(&mut buf_length) {
            Ok(_) => {
                let length = u32::from_be_bytes(buf_length);
                let mut buf = vec![0u8; length as usize];
                match self.socket.read_exact(&mut buf) {
                    Ok(_) => {
                        let (parse_res, is_end) = self.message.parse_from(buf)?;
                        if is_end {
                            return Err(HLMCommunicationError::EndMessageReceivedOnServer); // Only the server should send the end message, not receive it. If the server receives it, it means the client is misbehaving.
                        }
                        let ack_res = self.socket.write_all(&[0x01]);
                        if ack_res.is_err() {
                            return Err(HLMCommunicationError::SendIOError(
                                ack_res.err().unwrap().to_string(),
                            ));
                        }
                        if parse_res.is_some() {
                            let response = parse_res.unwrap();
                            self.only_send(response)?;
                        }
                        return Ok(());
                    }
                    Err(_) => {
                        return Err(HLMCommunicationError::FailedToReadMessage);
                    }
                }
            }
            Err(_) => {
                return Err(HLMCommunicationError::FailedToReadLength);
            }
        }
    }

    pub fn end(&mut self) -> Result<(), HLMCommunicationError> {
        let end_type = self.message.end_message();
        if end_type.is_none() {
            return Err(HLMCommunicationError::NoEndMessageDefined);
        }
        let end_type = end_type.unwrap();
        self.only_send(end_type)?;
        let res = self.socket.flush();
        if res.is_err() {
            return Err(HLMCommunicationError::SendIOError(
                res.err().unwrap().to_string(),
            ));
        }

        let res = self.socket.get_ref().shutdown(std::net::Shutdown::Both);
        if res.is_err() {
            return Err(HLMCommunicationError::SocketShutdownError(
                res.err().unwrap().to_string(),
            ));
        }
        return Ok(());
    }
}
