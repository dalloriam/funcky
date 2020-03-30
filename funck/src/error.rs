use std::io;
use std::path::PathBuf;

use snafu::Snafu;
pub use snafu::{ensure, ResultExt};

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Funcktion [{}] not found", name))]
    NotFound { name: String },

    #[snafu(display("Build error: {}", source.to_string()))]
    BuildError { source: io::Error },

    #[snafu(display("Build failed (nonzero exit code)"))]
    BuildFailedStatus,

    #[snafu(display("Directory does not exist: {}", source.to_string()))]
    DirNotExist { source: io::Error },

    #[snafu(display("Error loading shared object from {}", path.display()))]
    LoadingError { path: PathBuf, source: io::Error },

    #[snafu(display("Couldn't acquire lock: poisoned."))]
    ConcurrencyError,

    #[snafu(display("Error occurred during call"))]
    CallError,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
