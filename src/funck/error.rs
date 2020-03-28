use std::io;
use std::path::PathBuf;

pub use snafu::ResultExt;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Funcktion [{}] not found", name))]
    NotFound { name: String },

    #[snafu(display("Error loading shared object from {}", path.display()))]
    LoadingError { path: PathBuf, source: io::Error },

    #[snafu(display("Error occurred during call"))]
    CallError,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
