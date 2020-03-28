use std::any::Any;

pub trait Funcktion: Any + Send + Sync {
    fn name(&self) -> &'static str;
    fn call(&self) {}
}
