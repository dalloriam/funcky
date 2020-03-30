use std::any::Any;

use crate::CallError;

/// The Funcktion trait must be implemented by any cloud functions to be loaded in the system.
pub trait Funcktion: Any + Send + Sync {
    fn name(&self) -> &'static str;
    fn _call_internal(&self) -> Result<(), CallError>;
}
