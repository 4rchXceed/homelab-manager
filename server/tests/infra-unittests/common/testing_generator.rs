use application::generators::repositories::generator_base::GeneratorBase;
use async_trait::async_trait;
use config::generator::config::GeneratorBaseConfig;
use domain::agent::protocol::message::GeneratorMode;

pub struct TestingGenerator {
    return_val: String,
}

impl TestingGenerator {
    pub fn new(return_val: String) -> Self {
        Self { return_val }
    }
}

#[async_trait]
impl GeneratorBase for TestingGenerator {
    fn get_name(&self) -> &str {
        "test"
    }

    async fn generate_commands(
        &self,
        _config: &GeneratorBaseConfig,
        arguments: &Vec<String>,
        _file_path: &String,
    ) -> Result<(Vec<String>, GeneratorMode, Option<String>), String> {
        let first = arguments.first().ok_or(String::from(
            "No arguments, setup() fn is probably wrong (TestingGenerator)",
        ))?;

        if first != "TestArgOk" {
            return Err(format!(
                "Wanted 'TestArgOk' as argument, got '{first}' (TestingGenerator)"
            ));
        }

        return Ok((
            vec![self.return_val.clone()],
            GeneratorMode::Unsandboxed,
            None,
        ));
    }
}
