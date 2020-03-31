use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Message {
    message: String,
}

impl Message {
    pub fn new(message: &str) -> Message {
        Message {
            message: String::from(message),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ErrorMessage {
    error: String,
}

impl ErrorMessage {
    pub fn new<T: fmt::Display>(error: &T) -> ErrorMessage {
        ErrorMessage {
            error: error.to_string(),
        }
    }
}
