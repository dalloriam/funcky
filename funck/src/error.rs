use std::fmt;

#[derive(Debug, Default, Clone)]
pub struct CallError;

impl fmt::Display for CallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FFI Call error")
    }
}

impl std::error::Error for CallError {}

pub type CallResult<T> = Result<T, CallError>;
