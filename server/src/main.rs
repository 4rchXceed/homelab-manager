use std::sync::Arc;

use application::{
    agent::{
        repositories::agents_repository::AgentsRepository,
        usecases::authenticate::AuthenticateAgent,
    },
    file_server::usecases::starter::FileServerStarter,
    net::repositories::net_manager::NetworkManager,
};
use config::loader::ConfigLoader;
use infrastructure::{
    agent::inmemory::agent_repository::InMemoryAgentRepository,
    database::turso::{agent::TursoAgentsDb, connection::TursoDbConnection},
    file_server::rclone::file_server::RCloneFileServer,
    init_app,
    logger::simple_logger::init::init_logger,
    net::rustls::network_manager::RustlsNetworkManager,
};
use log::error;
use log::info;

#[tokio::main]
async fn main() {
    // Test implementation
    // Init
    init_app().expect("Failed to init app");

    // The config
    let config = ConfigLoader::new_from_env()
        .load()
        .map_err(|e| e.to_string())
        .expect("Failed to load config")
        .0;

    // The logger
    init_logger(config.config_general.log_level).expect("Failed to set log level");

    // The db
    let db = TursoDbConnection::new(config.config_general.database_file.clone())
        .await
        .map_err(|e| e.to_string())
        .expect("Failed to create db");

    // The agent's db
    let agents_db = Arc::new(TursoAgentsDb::new(Arc::new(db)));

    // The agent's repository
    let agent_repo = InMemoryAgentRepository::new(agents_db, config.agents_config.clone());

    agent_repo
        .ensure_agents()
        .await
        .map_err(|e| e.to_string())
        .expect("Failed to ensure agents");

    // Test file server
    let file_server = Arc::new(RCloneFileServer::new(&config.config_general));

    let starter = FileServerStarter::new(file_server);

    starter
        .start_and_wait()
        .await
        .expect("Failed to start file server");

    // The connection
    let nm = RustlsNetworkManager::listen(
        config.config_general.net_config.server_port,
        config.config_general.net_config.clone(),
    )
    .await
    .map_err(|e| e.to_string())
    .expect("Failed to start server");

    info!(
        "Server listening on port {}",
        config.config_general.net_config.server_port
    );

    // The connection authentication
    let auth = AuthenticateAgent::new(Arc::new(agent_repo.clone()));

    // The accept loop
    loop {
        let connection = nm.accept_new().await;

        match connection {
            Ok(connection) => {
                let result = auth.authenticate(connection).await;
                match result {
                    Ok(agent) => {
                        info!("Agent authenticated: {:?}", agent.id);
                    }
                    Err(e) => {
                        error!("Error authenticating agent: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
            }
        }
    }
}
