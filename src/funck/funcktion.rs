use std::any::Any;

/// The Funcktion trait must be implemented by any cloud functions to be loaded in the system.
pub trait Funcktion: Any + Send + Sync {
    fn name(&self) -> &'static str;
    fn call(&self) {}
}
