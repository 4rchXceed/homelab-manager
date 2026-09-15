use diesel::{SqliteConnection, r2d2::ConnectionManager};
use r2d2::Pool;

pub struct Context {
    pub db_pool: Pool<ConnectionManager<SqliteConnection>>,
    pub command_context: CommandContext,
    pub thread_context: ThreadContext,
}

pub struct ThreadContext {
    pub connection: Option<SqliteConnection>,
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
