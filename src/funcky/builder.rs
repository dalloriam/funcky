use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use snafu::{ensure, ResultExt, Snafu};

use super::DirHook;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to spawn build command: {}", source))]
    BuildSpawnError { source: io::Error },

    #[snafu(display("Error waiting for build command: {}", source))]
    BuildJoinError { source: io::Error },

    #[snafu(display("Build command exited with non-zero status code: {}", code))]
    ExitCodeNonZero { code: i32 },

    #[snafu(display("The final shared object file path ({}) is invalid: {}", path.display(), source))]
    InvalidOutputPath { source: io::Error, path: PathBuf },

    #[snafu(display("Failed to move to directory [{}]: {}", path.display(), source))]
    SwitchDirError { source: io::Error, path: PathBuf },
}
type Result<T, E = Error> = std::result::Result<T, E>;

/// FunckBuilder simply performs a server-side cargo build on a funck directory and returns
/// its .so file.
pub struct FunckBuilder {}

impl FunckBuilder {
    pub fn new() -> FunckBuilder {
        // TODO: Track all funcks built by the builder for easy / periodic rebuilds.
        FunckBuilder {}
    }

    pub fn build<P: AsRef<Path>>(&self, dir: P) -> Result<PathBuf> {
        let _hk = DirHook::new(dir.as_ref()).context(SwitchDirError {
            path: PathBuf::from(dir.as_ref()),
        })?;

        let project_name = dir
            .as_ref()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let mut cmd = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .spawn()
            .context(BuildSpawnError)?;

        let res = cmd.wait().context(BuildJoinError)?;

        ensure!(
            res.success(),
            ExitCodeNonZero {
                code: res.code().unwrap_or(-1)
            }
        );

        let rel_out_path = PathBuf::from(&format!("./target/release/lib{}.so", project_name));
        let so_file_path = std::fs::canonicalize(&rel_out_path)
            .context(InvalidOutputPath { path: rel_out_path })?;

        Ok(so_file_path)
    }
}
