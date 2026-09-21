use std::sync::RwLock;

use diesel::{SqliteConnection, r2d2::ConnectionManager};
use r2d2::{Pool, PooledConnection};

use crate::config::config::Config;

pub struct LockedContext {
    pub context: std::sync::Arc<RwLock<Context>>,
    pub command_context: CommandContext,
}

impl LockedContext {
    pub fn new(context: std::sync::Arc<RwLock<Context>>, command_context: CommandContext) -> Self {
        return LockedContext {
            context: context,
            command_context: command_context,
        };
    }

    pub fn new_server(context: std::sync::Arc<RwLock<Context>>) -> Self {
        return LockedContext {
            context: context,
            command_context: CommandContext::server_logger(),
        };
    }
}

pub struct Context {
    pub db_pool: Pool<ConnectionManager<SqliteConnection>>,
    pub config: Config,
}

impl Context {
    pub fn generate_connection(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, r2d2::Error> {
        return self.db_pool.get();
    }
}

pub struct CommandContext {
    print: Box<dyn Fn(&str) + Send + Sync>,
    input: Box<dyn FnMut() -> Result<String, std::io::Error> + Send + Sync>,
}

impl CommandContext {
    pub fn new(
        print: Box<dyn Fn(&str) + Send + Sync>,
        input: Box<dyn FnMut() -> Result<String, std::io::Error> + Send + Sync>,
    ) -> Self {
        return CommandContext { print, input };
    }

    pub fn server_logger() -> Self {
        CommandContext {
            print: Box::new(|message| println!("{}", message)),
            input: Box::new(|| {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                return Ok(input.trim().to_string());
            }),
        }
    }

    pub fn print(&self, message: &str) {
        (self.print)(message);
    }

    pub fn input(&mut self) -> Result<String, std::io::Error> {
        (self.input)()
    }
}
