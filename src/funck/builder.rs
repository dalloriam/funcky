use std::process::Command;

/// FunckBuilder simply performs a server-side cargo build on a funck directory and returns
/// its .so file.
pub struct FunckBuilder {}

impl FunckBuilder {
    pub fn new() -> FunckBuilder {
        // TODO: Track all funcks built by the builder for easy / periodic rebuilds.
        FunckBuilder {}
    }
}
