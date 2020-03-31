use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use snafu::{ensure, ResultExt, Snafu};

use super::compiler;
use super::DirHook;

#[derive(Debug, Snafu)]
pub enum Error {
    CompileError { source: compiler::Error },
}
type Result<T, E = Error> = std::result::Result<T, E>;

/// FunckBuilder simply performs a server-side cargo build on a funck directory and returns
/// its .so file.
pub struct FunckBuilder {}

impl FunckBuilder {}
