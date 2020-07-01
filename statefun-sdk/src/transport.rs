use crate::function_registry::FunctionRegistry;

pub mod hyper;

pub trait Transport {
    fn run(self, function_registry: FunctionRegistry) -> Result<(), failure::Error>;
}
