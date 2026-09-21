use crate::client::Client;

mod client;
mod config;
mod net;

#[tokio::main]
async fn main() {
    let ini_path = "config.ini".to_string();
    let mut client = Client::new(ini_path)
        .map_err(|e| e.to_string())
        .expect("Failed to start client:");
    client
        .run()
        .await
        .map_err(|e| e.to_string())
        .expect("Client runtime error:");
}
