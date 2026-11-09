use std::{sync::Arc, time::Duration};

use application::{
    agent::repositories::agents_repository::AgentsRepository,
    file_server::usecases::starter::FileServerStarter,
    generators::{
        repositories::generator_config_registry::GeneratorConfigRegistry,
        usecases::generate::GenerateConfig,
    },
    net::repositories::net_manager::NetworkManager,
    services::repositories::services_repository::ServicesRepository,
};
use config::loader::ConfigLoader;
use domain::agent::protocol::message::ServerToAgentMsg;
use infrastructure::{
    agent::inmemory::agent_repository::InMemoryAgentRepository,
    database::turso::{
        agents::TursoAgentsDb, backups::TursoBackupsDb, connection::TursoDbConnection,
        services::TursoServicesDb, user_vars::TursoUserVarsDb,
    },
    file_server::rclone::file_server::RCloneFileServer,
    generators::{
        bases::{bash::BashGeneratorBase, replace::ReplaceFileGeneratorBase},
        inmemory::{
            generator_base_registry::InMemoryGeneratorRegistry,
            generator_config_registry::InMemoryGeneratorConfigRegistry,
        },
    },
    init_app,
    interactions::cli::user_context::CliUserContext,
    logger::simple_logger::init::init_logger,
    net::rustls::network_manager::RustlsNetworkManager,
    services::inmemory::services_repository::InMemoryServicesRepository,
    var_provider::{
        inmemeory::var_provider_registry::InMemoryVarProviderRegistry,
        types::{string::StringVarProvider, user_var::UserVarProvider},
    },
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

    // The default user context
    let user_context = Arc::new(CliUserContext::new());

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
    let services_db = Arc::new(TursoServicesDb::new(db.clone(), backups_db));

    // The user vars db
    let user_vars_db = Arc::new(TursoUserVarsDb::new(db.clone()));

    // The variable provider registry
    let var_provider_registry = Arc::new(InMemoryVarProviderRegistry::new());

    // The "string" var provider
    let string_var_provider = Arc::new(StringVarProvider::new());
    var_provider_registry.add(string_var_provider).await;

    // The "user_var" var provider
    let user_var_provider = Arc::new(UserVarProvider::new(user_vars_db.clone()));
    var_provider_registry.add(user_var_provider).await;

    // The generators repository
    let generator_registry = Arc::new(InMemoryGeneratorConfigRegistry::new());
    generator_registry
        .reload(config.generators_config.clone())
        .await;

    // The generators base repository
    let generator_base_registry =
        Arc::new(InMemoryGeneratorRegistry::new(generator_registry.clone()));

    // Adding the generators
    generator_base_registry
        .add(Arc::new(BashGeneratorBase::new()))
        .await;

    generator_base_registry
        .add(Arc::new(ReplaceFileGeneratorBase::new()))
        .await;

    // Generate usecase
    let generate_usecase = GenerateConfig::new(
        generator_base_registry.clone(),
        var_provider_registry.clone(),
    );

    // The services repository
    let service_repo = Arc::new(InMemoryServicesRepository::new(
        agent_repo.clone(),
        services_db,
        generate_usecase,
        user_vars_db,
    ));

    service_repo
        .ensure_services(config.services_config)
        .await
        .expect("Failed to ensure services into db");

    // Test file server
    let file_server = Arc::new(
        RCloneFileServer::new(&config.config_general)
            .map_err(|e| e.to_string())
            .expect("Failed to create the file server"),
    );

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

                let result = service_repo
                    .assign_to(
                        String::from("service_id"),
                        String::from("main"),
                        user_context.clone(),
                    )
                    .await;

                match result {
                    Ok(_) => {
                        info!("Service assigned to agent successfully");
                    }
                    Err(e) => {
                        error!("Failed to assign service to agent: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
            }
        }
    }
}
