use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, insert_into};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, RwLock},
    thread::Thread,
};
use thiserror::Error;
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::{
    config::{
        config::Config,
        server::{config::ServerConfig, storage_config::StorageConfig},
    },
    consts::{DEFAULT_FALLBACK_STORAGE, DEFAULT_SMALL_OPERATION_TIMEOUT, SEND_SLEEP_DELAY},
    context::{CommandContext, Context},
    logger::{log_error, log_recv_mismatch},
    models::Server,
    protocol::{
        message::{
            FromAgentMessage, FromAgentMessageWrapper, ToAgentMessage, ToAgentMessageWrapper,
        },
        storage::Storage,
    },
    schema,
};

macro_rules! agent_take_write {
    ($me: expr) => {
        $me.write()
            .map_err(|e| AgentCommunicationError::AgentUnlockFailed(e.to_string()))?
    };
}

macro_rules! agent_take_read {
    ($me: expr) => {
        $me.read()
            .map_err(|e| AgentCommunicationError::AgentUnlockFailed(e.to_string()))?
    };
}

pub type AgentLocked = Arc<RwLock<Agent>>;

#[derive(Error, Debug)]
pub enum MessageInvalidity {
    #[error("Invalid UTF-8 message received: {0}")]
    InvalidUtf8(std::string::FromUtf8Error),
    #[error("Invalid JSON message received: {0}")]
    InvalidJson(serde_json::Error),
}

#[derive(Error, Debug)]
pub enum AgentCommunicationError {
    #[error("Agent is not connected (maybe disconnected / not initialized)")]
    AgentNotConnected,
    #[error("Failed to send message to agent: {0}")]
    FailedToSendMessage(std::io::Error),
    #[error("Failed to receive message from agent: {0}")]
    FailedToReceiveMessage(std::io::Error),
    #[error("Auth failed: {0}")]
    AuthFailed(String),
    #[error("Client tried to connect with wrong UUID, connection rejected")]
    AuthRejected,
    #[error("Agent did not acknowledge reverse API key")]
    ReverseApiKeyNotAcknowledged,
    #[error("Failed to unlock agent for writing (you should not see this error, please report it)")]
    AgentUnlockFailed(String),
    #[error("Failed to read from socket: {0}")]
    ReadError(std::io::Error),
    #[error("Failed to write to socket: {0}")]
    WriteError(std::io::Error),
    #[error("Invalid message received: {0}")]
    InvalidMessage(MessageInvalidity),
    #[error("Ping-Pong timed out")]
    PingPongTimeout,
}

pub struct Agent {
    id: String,
    keep_alive: Option<Thread>,
    api_key: String,
    ip: Option<IpAddr>,
    storages_configs: HashMap<String, StorageConfig>,
    storages: Vec<Storage>,
    send_queue: HashMap<u32, ToAgentMessage>,
    recv_queue: HashMap<u32, FromAgentMessage>,
    current_message_id: u32,
    reverse_api_key: String,
    connected: bool,
}

impl Agent {
    pub fn new(
        id: String,
        api_key: String,
        new_config: &Config,
        context: &mut Context,
    ) -> Result<AgentLocked, AgentCommunicationError> {
        let agent = Agent {
            id: id,
            keep_alive: None,
            api_key: api_key,
            ip: None,
            storages_configs: HashMap::new(),
            storages: Vec::new(),
            send_queue: HashMap::new(),
            recv_queue: HashMap::new(),
            current_message_id: 0,
            reverse_api_key: Uuid::new_v4().to_string(),
            connected: false,
        };

        let agent = Arc::new(RwLock::new(agent));

        Self::reload(agent.clone(), new_config, context)?;

        return Ok(agent);
    }

    pub fn reload(
        me: AgentLocked,
        new_config: &Config,
        context: &mut Context,
    ) -> Result<(), AgentCommunicationError> {
        let mut agent = agent_take_write!(me);

        let server = new_config.servers_config.iter().find(|s| s.id == agent.id);

        if let Some(server) = server {
            Self::init_server_storage(me.clone(), Some(server));

            agent.id = server.id.clone();
            agent.api_key = server.api_key.clone();
            agent.ip = Some(server.ip);

            if let Some(conn) = &mut context.thread_context.connection {
                let db_server = schema::server::table
                    .filter(schema::server::id_str.eq(agent.id.clone()))
                    .first::<Server>(conn);

                if let Ok(db_server) = db_server {
                    agent.update_server(&mut context.command_context, db_server, conn, server);
                } else if let Err(e) = db_server {
                    match e {
                        diesel::result::Error::NotFound => {
                            agent.insert_server(&context.command_context, conn, server);
                        }

                        _ => {
                            log_error(
                                format!(
                                    "Error while running server db sync query: {}",
                                    e.to_string()
                                )
                                .as_str(),
                                &context.command_context,
                            );
                        }
                    }
                }
            } else {
                log_error("Database context not available", &context.command_context);
            }
        } else {
            // TODO: Server deleted
        }

        return Ok(());
    }

    pub fn send_message(
        &mut self,
        message: ToAgentMessage,
    ) -> Result<u32, AgentCommunicationError> {
        if !self.connected {
            return Err(AgentCommunicationError::AgentNotConnected);
        }

        let message_id = self.generate_message_id();

        self.send_queue.insert(message_id, message);

        return Ok(message_id);
    }

    pub fn send_message_pingpong(
        message: ToAgentMessage,
        timeout: Option<std::time::Duration>,
        me: AgentLocked,
    ) -> Result<FromAgentMessage, AgentCommunicationError> {
        let message_id = agent_take_write!(me).send_message(message)?;

        let start_time = std::time::Instant::now();
        loop {
            if let Some(response) = agent_take_write!(me).recv_queue.remove(&message_id) {
                return Ok(response);
            }
            if let Some(timeout) = timeout {
                if start_time.elapsed() > timeout {
                    return Err(AgentCommunicationError::PingPongTimeout);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(SEND_SLEEP_DELAY));
        }
    }

    fn check_storage(me: AgentLocked, path: String, do_not_create: bool) -> bool {
        match Self::send_message_pingpong(
            ToAgentMessage::CheckStorage(path.clone(), !do_not_create),
            Some(DEFAULT_SMALL_OPERATION_TIMEOUT),
            me,
        ) {
            Ok(FromAgentMessage::StorageCheckResult(result)) => {
                return result;
            }
            Err(e) => {
                log_error(
                    format!("Error while checking storage: {}", e.to_string()).as_str(),
                    &CommandContext::server_logger(),
                );

                return false;
            }
            _ => {
                log_recv_mismatch(&CommandContext::server_logger(), "StorageCheckResult");
            }
        }
        return true;
    }

    fn init_server_storage(
        me: AgentLocked,
        server: Option<&ServerConfig>,
    ) -> Result<(), AgentCommunicationError> {
        if let Some(server) = server {
            agent_take_write!(me).storages_configs = server.storages.clone();
        }

        if agent_take_read!(me).connected {
            agent_take_write!(me).storages =
                std::mem::take(&mut agent_take_write!(me).storages_configs)
                    .iter()
                    .map(|(id, c)| {
                        Storage::new(
                            id.clone(),
                            c.path.clone(),
                            Self::check_storage(me.clone(), c.path.clone(), c.do_not_create),
                            c.fallback.clone(),
                            c.do_not_create,
                        )
                    })
                    .collect();
        }

        return Ok(());
    }

    pub fn send_raw(bytes: Vec<u8>, stream: &TcpStream) -> Result<(), AgentCommunicationError> {
        match stream.try_write(&bytes) {
            Ok(_) => Ok(()),
            Err(e) => Err(AgentCommunicationError::FailedToSendMessage(e)),
        }
    }

    pub fn recv_raw(length: usize, stream: &TcpStream) -> Result<Vec<u8>, AgentCommunicationError> {
        let mut buffer = vec![0; length];

        match stream.try_read(&mut buffer) {
            Ok(_) => Ok(buffer),
            Err(e) => Err(AgentCommunicationError::FailedToReceiveMessage(e)),
        }
    }

    pub fn init_connection(
        me: AgentLocked,
        stream: &TcpStream,
    ) -> Result<(), AgentCommunicationError> {
        let agent = agent_take_read!(me);

        Self::send_raw(b"AUTH".to_vec(), stream)?;

        let uuid = Self::recv_raw(36, stream)?;

        let uuid_str = String::from_utf8(uuid)
            .map_err(|e| AgentCommunicationError::AuthFailed(e.to_string()))?;

        if uuid_str != agent.id {
            return Err(AgentCommunicationError::AuthRejected);
        }

        Self::send_raw(b"OK".to_vec(), stream)?;

        Self::send_raw(agent.reverse_api_key.as_bytes().to_vec(), stream)?;

        let ack = Self::recv_raw(2, stream)?;

        if ack != b"OK" {
            return Err(AgentCommunicationError::ReverseApiKeyNotAcknowledged);
        }

        return Ok(());
    }

    pub fn generate_message_id(&mut self) -> u32 {
        self.current_message_id += 1;
        return self.current_message_id;
    }

    pub fn connect(socket: &TcpStream, me: AgentLocked) -> Result<(), AgentCommunicationError> {
        Self::init_connection(me.clone(), socket)?;

        agent_take_write!(me).connected = true;

        return Ok(());
    }

    pub fn process_messages(
        me: AgentLocked,
        socket: Arc<TcpStream>,
    ) -> Result<(), AgentCommunicationError> {
        loop {
            let mut buf = [0; 4];

            socket
                .try_read(&mut buf)
                .map_err(|e| AgentCommunicationError::ReadError(e))?;

            let message_length = u32::from_be_bytes(buf) as usize;

            let message_bytes = Self::recv_raw(message_length, &socket)?;

            let json = String::from_utf8(message_bytes).map_err(|e| {
                AgentCommunicationError::InvalidMessage(MessageInvalidity::InvalidUtf8(e))
            })?;

            let message: FromAgentMessageWrapper = serde_json::from_str(&json).map_err(|e| {
                AgentCommunicationError::InvalidMessage(MessageInvalidity::InvalidJson(e))
            })?;

            let mut agent = me
                .write()
                .map_err(|e| AgentCommunicationError::AgentUnlockFailed(e.to_string()))?;

            agent.recv_queue.insert(message.id, message.message);
        }
    }

    pub fn process_sends(
        me: AgentLocked,
        socket: Arc<TcpStream>,
    ) -> Result<(), AgentCommunicationError> {
        loop {
            let agent = me
                .read()
                .map_err(|e| AgentCommunicationError::AgentUnlockFailed(e.to_string()))?;

            for (id, message) in agent.send_queue.iter() {
                let message_wrapper = ToAgentMessageWrapper {
                    id: *id,
                    message: message.clone(),
                };

                let json = serde_json::to_string(&message_wrapper).map_err(|e| {
                    AgentCommunicationError::InvalidMessage(MessageInvalidity::InvalidJson(e))
                })?;

                let message_bytes = json.as_bytes();

                let message_length = (message_bytes.len() as u32).to_be_bytes();

                Self::send_raw(message_length.to_vec(), &socket)?;
                Self::send_raw(message_bytes.to_vec(), &socket)?;
            }
            std::thread::sleep(std::time::Duration::from_millis(SEND_SLEEP_DELAY));
        }
    }

    pub fn get_storage(&self, searching_storage: String) -> Option<&Storage> {
        let storage = self.storages.iter().find(|s| s.id == searching_storage);

        if let Some(storage) = storage {
            if storage.is_invalid && storage.id != DEFAULT_FALLBACK_STORAGE {
                return self.get_storage(storage.fallback.clone());
            } else {
                return Some(storage);
            }
        } else if searching_storage != DEFAULT_FALLBACK_STORAGE {
            return self.get_storage(DEFAULT_FALLBACK_STORAGE.to_string());
        } else {
            return None;
        }
    }

    fn insert_server(
        &mut self,
        context: &CommandContext,
        conn: &mut diesel::prelude::SqliteConnection,
        server: &ServerConfig,
    ) {
        let insert = insert_into(schema::server::dsl::server)
            .values((
                schema::server::id_str.eq(self.id.clone()),
                schema::server::description.eq(server.description.clone()),
                schema::server::ip.eq(server.ip.to_string()),
                schema::server::api_key.eq(server.api_key.clone()),
                schema::server::reverse_api_key.eq(self.reverse_api_key.clone()),
            ))
            .execute(conn);

        if let Err(error) = insert {
            log_error(
                format!(
                    "Error while adding server into database: {}",
                    error.to_string()
                )
                .as_str(),
                context,
            );
        }
    }

    pub fn sync_to_db(&mut self, server: &ServerConfig) {
        self.id = server.id.clone();
        self.ip = Some(server.ip);
    }

    fn update_server(
        &self,
        context: &mut CommandContext,
        db_server: Server,
        conn: &mut diesel::prelude::SqliteConnection,
        server: &ServerConfig,
    ) {
        let update = diesel::update(schema::server::dsl::server);

        if db_server.ip != server.ip.to_string()
            || db_server.description != Some(server.description.clone())
        {
            let update = update
                .set((
                    schema::server::ip.eq(server.ip.to_string()),
                    schema::server::description.eq(server.description.clone()),
                ))
                .execute(conn);

            if let Err(error) = update {
                log_error(
                    format!(
                        "Error while updating server in database: {}",
                        error.to_string()
                    )
                    .as_str(),
                    &context,
                );
            }
        }
    }
}
