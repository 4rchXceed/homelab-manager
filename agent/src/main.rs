use crate::client::Client;

mod client;
mod config;
mod db;
mod errors;
mod net;

#[tokio::main]
async fn main() {
    let ini_path = "config.ini".to_string();
    Client::new(ini_path)
        .await
        .map_err(|e| e.to_string())
        .expect("Failed to start client")
        .run()
        .await
        .map_err(|e| e.to_string())
        .expect("Client runtime error");
}
