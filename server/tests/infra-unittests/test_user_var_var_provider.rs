use std::sync::Arc;

use application::{
    database::repositories::user_vars_db::UserVarsDb,
    var_provider::{context::VarProviderContext, repositories::var_provider::VarProvider},
};
use infrastructure::{
    database::turso::user_vars::TursoUserVarsDb, var_provider::types::user_var::UserVarProvider,
};
use tempdir::TempDir;
use yaml_rust2::Yaml;

use crate::{
    common::generate_context::{PanicContext, TestContext},
    common::generate_db::{generate_db, load_yaml},
};

const YAML_OK: &str = "
id: id_test
name: TestName
";

const YAML_DEFAULT: &str = "
id: id_test_def
name: TestName
default_value: TestVal
";

const YAML_ERR: &str = "
id: id_error
";

async fn setup(yaml: &str) -> (TempDir, UserVarProvider, Yaml, Arc<dyn UserVarsDb>) {
    let (db, tempdir) = generate_db().await;

    let conn = Arc::new(db);

    let user_vars_db = Arc::new(TursoUserVarsDb::new(conn));

    let user_vars = UserVarProvider::new(user_vars_db.clone());

    let args = load_yaml(yaml);

    return (tempdir, user_vars, args, user_vars_db);
}

#[tokio::test]
async fn test_success() {
    let test_context = TestContext::new(String::from("TestVal"));
    let test_context = Arc::new(test_context);

    let panic_context = PanicContext::new();
    let panic_context = Arc::new(panic_context);

    let (_tempdir, user_vars, args, user_vars_db) = setup(YAML_OK).await;

    let result = user_vars
        .process(
            args.clone(),
            String::from("test-service"),
            VarProviderContext {
                user_context: test_context,
            },
        )
        .await
        .expect("Error while processing the variable");

    assert_eq!(result, "TestVal");

    let value = user_vars_db
        .get_value(String::from("id_test"))
        .await
        .expect("Failed to get the value")
        .expect("Var with id 'id_test' not found");

    assert_eq!(value, "TestVal");

    let result = user_vars
        .process(
            args,
            String::from("test-service"),
            VarProviderContext {
                user_context: panic_context,
            },
        )
        .await
        .expect("Error while processing the variable");

    assert_eq!(result, "TestVal");
}

#[tokio::test]
async fn test_missing_field() {
    let test_context = TestContext::new(String::from("TestVal"));
    let test_context = Arc::new(test_context);

    let (_tempdir, user_vars, args, _user_vars_db) = setup(YAML_ERR).await;

    let result = user_vars
        .process(
            args,
            String::from("test-service"),
            VarProviderContext {
                user_context: test_context,
            },
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_default_value() {
    let test_context = TestContext::new(String::from("ErrorVal"));
    let test_context = Arc::new(test_context);

    let (_tempdir, user_vars, args, user_vars_db) = setup(YAML_DEFAULT).await;

    let result = user_vars
        .process(
            args,
            String::from("test-service"),
            VarProviderContext {
                user_context: test_context,
            },
        )
        .await
        .expect("Failed to create variable");

    assert_eq!(result, "TestVal");

    let value = user_vars_db
        .get_value(String::from("id_test_def"))
        .await
        .expect("Failed to get the value")
        .expect("Var with id 'id_test_def' not found");

    assert_eq!(value, "TestVal");
}
