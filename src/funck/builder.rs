use std::process::Command;

use rood::CausedResult;

/// FunckBuilder simply performs a server-side cargo build on a funck directory and returns
/// its .so file.
pub struct FunckBuilder {}

impl FunckBuilder {
    pub fn new() -> FuncktionBuilder {
        // TODO: Track all funcks built by the builder for easy / periodic rebuilds.
        FuncktionBuilder {}
    }

    pub fn build(&self, path: &str, debug: bool) -> CausedResult<()> {
        Ok(())
    }
}
