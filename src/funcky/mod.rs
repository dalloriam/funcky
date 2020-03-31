mod builder;
mod dirmove;
mod loader;
mod manager;

// === Private Exports ===
use builder::FunckBuilder;
use dirmove::DirHook;
use loader::FunckLoader;

// === Public Exports ===
pub use manager::{BuilderError, Config, Error, FunckManager, LoaderError};
