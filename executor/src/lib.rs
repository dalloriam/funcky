mod compiler;
mod dirmove;
mod dropdir;
mod loader;
mod manager;
mod status;

// === Private Exports ===
use dirmove::DirHook;
use loader::FunckLoader;
use status::{FuncktionEntry, Status, StatusTracker};

// === Public Exports ===
pub use dropdir::DropDir;
pub use manager::{Config, Error, FunckManager, LoaderError};

#[cfg(test)]
mod tests;
