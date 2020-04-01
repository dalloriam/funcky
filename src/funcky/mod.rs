mod compiler;
mod dirmove;
mod dropdir;
mod loader;
mod manager;

// === Private Exports ===
use dirmove::DirHook;
use loader::FunckLoader;

// === Public Exports ===
pub use dropdir::DropDir;
pub use manager::{Config, Error, FunckManager, LoaderError};
