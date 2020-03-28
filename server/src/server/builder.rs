use std::path::{Path, PathBuf};
use std::process::Command;

use funck::error::*;

use crate::server::DirHook;

/// FunckBuilder simply performs a server-side cargo build on a funck directory and returns
/// its .so file.
pub struct FunckBuilder {}

impl FunckBuilder {
    pub fn new() -> FunckBuilder {
        // TODO: Track all funcks built by the builder for easy / periodic rebuilds.
        FunckBuilder {}
    }

    pub fn build<P: AsRef<Path>>(&self, dir: P) -> Result<PathBuf> {
        let _hk = DirHook::new(dir.as_ref());

        let project_name = dir
            .as_ref()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let so_file_path = std::fs::canonicalize(PathBuf::from(&format!(
            "./target/release/lib{}.so",
            project_name
        )))
        .context(BuildError)?;

        let mut cmd = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .spawn()
            .context(BuildError)?;

        let res = cmd.wait().context(BuildError)?;
        ensure!(res.success(), BuildFailedStatus);

        Ok(so_file_path)
    }
}
