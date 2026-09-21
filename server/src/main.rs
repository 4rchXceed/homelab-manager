mod config;
mod consts;
mod context;
mod database;
mod logger;
mod models;
mod net;
mod protocol;
mod schema;
mod server;
mod utils;

#[tokio::main]
async fn main() {
    let mut server = server::HomelabServer::new()
        .map_err(|e| e.to_string())
        .expect("Failed to create the server :( ");
    server
        .run()
        .await
        .map_err(|e| e.to_string())
        .expect("Failed to run the server :( ");
}
