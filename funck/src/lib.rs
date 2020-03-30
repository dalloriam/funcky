/*
Funck is a small library containing a trait for funcktions as well as a macro for exporting
them in a dylib
*/

mod error;
mod export;
mod funcktion;

pub use error::{CallError, CallResult};
pub use funcktion::Funcktion;
