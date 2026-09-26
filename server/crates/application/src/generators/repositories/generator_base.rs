use config::generator::config::Arguments;

pub trait GeneratorBase {
    fn get_name(&self) -> &str;
    fn generate_commands(&self, config: &Arguments) -> Result<Vec<String>, String>;
}
