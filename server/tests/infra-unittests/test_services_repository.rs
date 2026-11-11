use std::sync::Arc;

use application::{
    agent::repositories::agents_repository::AgentsRepository,
    generators::{
        repositories::{
            generator_bases_registry::GeneratorBaseRegistry,
            generator_config_registry::GeneratorConfigRegistry,
        },
        usecases::generate::GenerateConfig,
    },
    services::repositories::services_repository::ServicesRepository,
};
use config::config::Config;
use domain::agent::protocol::message::{GeneratorMode, ServerToAgentMsg};
use infrastructure::{
    agent::inmemory::agent_repository::InMemoryAgentRepository,
    database::turso::{
        agents::TursoAgentsDb, backups::TursoBackupsDb, connection::TursoDbConnection,
        services::TursoServicesDb, user_vars::TursoUserVarsDb,
    },
    generators::inmemory::{
        generator_base_registry::InMemoryGeneratorRegistry,
        generator_config_registry::InMemoryGeneratorConfigRegistry,
    },
    services::inmemory::services_repository::InMemoryServicesRepository,
    var_provider::{
        inmemeory::var_provider_registry::InMemoryVarProviderRegistry,
        types::string::StringVarProvider,
    },
};
use tempdir::TempDir;

use crate::common::{
    generate_context::PanicContext, generate_db::generate_db, load_config::load_config,
    testing_agent_conn::TestingAgentConnection, testing_generator::TestingGenerator,
};

async fn setup() -> (
    Arc<dyn ServicesRepository>,
    Arc<TursoDbConnection>,
    Arc<TestingAgentConnection>,
    Config,
    TempDir,
) {
    // Config setup
    let config = load_config(include_str!("configs/services_repo_testdatas.yaml"));

    // Db setup
    let (db, tempdir) = generate_db().await;
    let db = Arc::new(db);

    // DBs setups
    let agents_db = Arc::new(TursoAgentsDb::new(db.clone()));
    let backups_db = Arc::new(TursoBackupsDb::new(db.clone()));
    let services_db = Arc::new(TursoServicesDb::new(db.clone(), backups_db));
    let user_vars_db = Arc::new(TursoUserVarsDb::new(db.clone()));

    // "Fake" agent net
    let agent_net = Arc::new(TestingAgentConnection::new());

    // Agent setup
    let agent_repo = Arc::new(InMemoryAgentRepository::new(
        agents_db,
        config.agents_config.clone(),
    ));

    agent_repo
        .ensure_agents()
        .await
        .expect("Failed to ensure agents");
    agent_repo
        .set_agents_net(String::from("test-agent"), agent_net.clone())
        .await
        .expect("Failed to set agent's net");

    // GeneratorConfig repo setup
    let generator_config_registries = Arc::new(InMemoryGeneratorConfigRegistry::new());
    generator_config_registries
        .reload(config.generators_config.clone())
        .await;

    // GeneratorBase repo setup
    let generator_bases_registry =
        Arc::new(InMemoryGeneratorRegistry::new(generator_config_registries));

    generator_bases_registry
        .add(Arc::new(TestingGenerator::new(String::from("TestOk"))))
        .await;

    // VarProvider repo setup
    let var_proider_registry = Arc::new(InMemoryVarProviderRegistry::new());

    var_proider_registry
        .add(Arc::new(StringVarProvider::new()))
        .await;

    // GenerateConfig usecase
    let generate_config = GenerateConfig::new(generator_bases_registry, var_proider_registry);

    // Service repository
    let service_repo = InMemoryServicesRepository::new(
        agent_repo,
        services_db.clone(),
        generate_config,
        user_vars_db,
    );

    return (Arc::new(service_repo), db, agent_net, config, tempdir);
}

#[tokio::test]
async fn test_ensure_services() {
    let (service_repo, db, _test_agent, config, _tempdir) = setup().await;

    service_repo
        .ensure_services(config.services_config)
        .await
        .expect("Failed to call 'Ensure Services': ");

    let mut results = db
        .fetch("SELECT id_str FROM service", ())
        .await
        .map_err(|e| e.to_string())
        .expect("Failed to run query");

    let row = results
        .next()
        .await
        .expect("Failed to get first row")
        .expect("Service not added to database");

    let service_id = row
        .get::<String>(0)
        .expect("Failed to convert value to String");

    assert_eq!(service_id, "test_service")
}

// TODO: Test reload() fn

#[tokio::test]
async fn test_assign_to() {
    let (service_repo, _db, test_agent, config, _tempdir) = setup().await;

    service_repo
        .ensure_services(config.services_config)
        .await
        .expect("Failed to call 'Ensure Services': ");

    service_repo
        .assign_to(
            String::from("test_service"),
            String::from("test-agent"),
            Arc::new(PanicContext::new()),
        )
        .await
        .expect("Failed to run assign / reload service");

    let sent_messages = test_agent.get_sent().await;

    assert_eq!(sent_messages.len(), 1);

    match sent_messages[0].clone() {
        ServerToAgentMsg::GenerateConfig {
            commands,
            generator_mode,
            service,
            image,
        } => {
            assert_eq!(commands, vec!["TestOk"]);
            assert_eq!(generator_mode, GeneratorMode::Unsandboxed);
            assert_eq!(service, "test_service");
            assert_eq!(image, None);
        }
        _ => panic!(
            "Invalid message: {:?}. Wanted a GenerateConfig",
            sent_messages
        ),
    }
}
