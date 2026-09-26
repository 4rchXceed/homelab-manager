use std::{sync::Arc, time::Duration};

use application::{
    agent::repositories::agents_repository::AgentsRepository,
    file_server::usecases::starter::FileServerStarter,
    net::repositories::net_manager::NetworkManager,
    services::repositories::services_repository::ServicesRepository,
};
use config::loader::ConfigLoader;
use domain::agent::protocol::message::ServerToAgentMsg;
use infrastructure::{
    agent::inmemory::agent_repository::InMemoryAgentRepository,
    database::turso::{
        agents::TursoAgentsDb, backups::TursoBackupsDb, connection::TursoDbConnection,
        services::TursoServicesDb,
    },
    file_server::rclone::file_server::RCloneFileServer,
    init_app,
    logger::simple_logger::init::init_logger,
    net::rustls::network_manager::RustlsNetworkManager,
    services::inmemory::services_repository::InMemoryServicesRepository,
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
    let db = Arc::new(
        TursoDbConnection::new(config.config_general.database_file.clone())
            .await
            .map_err(|e| e.to_string())
            .expect("Failed to create db"),
    );

    // The agent's db
    let agents_db = Arc::new(TursoAgentsDb::new(db.clone()));

    // The agent's repository
    let agent_repo = Arc::new(InMemoryAgentRepository::new(
        agents_db,
        config.agents_config.clone(),
    ));

    agent_repo
        .ensure_agents()
        .await
        .map_err(|e| e.to_string())
        .expect("Failed to ensure agents");

    // The backups db
    let backups_db = Arc::new(TursoBackupsDb::new(db.clone()));

    // The services db
    let services_db = Arc::new(TursoServicesDb::new(db, backups_db));

    // The services repository
    let service_repo = Arc::new(InMemoryServicesRepository::new(
        agent_repo.clone(),
        services_db,
    ));

    service_repo
        .ensure_services(config.services_config)
        .await
        .expect("Failed to ensure services into db");

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
        agent_repo.clone(),
    )
    .await
    .map_err(|e| e.to_string())
    .expect("Failed to start server");

    info!(
        "Server listening on port {}",
        config.config_general.net_config.server_port
    );

    // The accept loop
    loop {
        let connection = nm.accept_new_auth_agent().await;

        match connection {
            Ok(connection) => {
                connection.start_processing().await;

                info!("New agent connection accepted and processing started");

                let response = connection
                    .send_pingpong(ServerToAgentMsg::Void, Duration::from_secs(10))
                    .await;

                match response {
                    Ok(_) => {
                        info!("Ping-pong successful with agent");
                    }
                    Err(e) => {
                        error!("Ping-pong failed with agent: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
            }
        }
    }
}
