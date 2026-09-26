use std::sync::Arc;

use crate::generators::repositories::generator_base::GeneratorBase;

pub trait GeneratorRegistry {
    fn generator_for_name(&self, name: String) -> Option<Arc<dyn GeneratorBase>>;
}
