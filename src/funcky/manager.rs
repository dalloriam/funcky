use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use snafu::{ResultExt, Snafu};

pub use super::builder::Error as BuilderError;
pub use super::loader::Error as LoaderError;
use super::{FunckBuilder, FunckLoader};

#[derive(Debug, Snafu)]
pub enum Error {
    BuildFailure {
        source: BuilderError,
    },
    #[snafu(display("Error calling function: {}", source))]
    CallError {
        source: LoaderError,
    },
    #[snafu(display("Failed to move shared object: {}", source))]
    CantMoveSharedObject {
        source: io::Error,
    },
    InitializationError {
        source: io::Error,
    },
    MissingFileName,
    MissingSharedObject,
    LoaderLockFailure,
    LoadError {
        source: LoaderError,
    },
}
type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Config {
    pub shared_object_directory: PathBuf,
    pub tmp_dir: PathBuf,
}

pub struct FunckManager {
    builder: FunckBuilder,
    pub cfg: Config,
    loader: RwLock<FunckLoader>,
}

impl FunckManager {
    pub fn new(cfg: Config) -> Result<FunckManager> {
        FunckManager::ensure_dirs_exist(&cfg)?;
        let mut manager = FunckManager {
            builder: FunckBuilder::new(),
            cfg,
            loader: RwLock::new(FunckLoader::new()),
        };

        // Perform initial loading of .so files.
        manager.refresh_shared_objects()?;
        Ok(manager)
    }

    fn ensure_dirs_exist(cfg: &Config) -> Result<()> {
        log::debug!("initializing directories...");
        // Ensure shared object directory exists.
        if !cfg.shared_object_directory.exists() {
            fs::create_dir_all(&cfg.shared_object_directory).context(InitializationError)?;
        }

        if !cfg.tmp_dir.exists() {
            fs::create_dir_all(&cfg.tmp_dir).context(InitializationError)?;
        }
        Ok(())
    }

    fn refresh_shared_objects(&mut self) -> Result<()> {
        log::info!("refreshing loaded shared objects...");
        let mut fn_loader = FunckLoader::new();
        for f in fs::read_dir(&self.cfg.shared_object_directory)
            .context(InitializationError)?
            .filter_map(|e| e.ok())
        {
            if let Some(ext) = f.path().extension() {
                if ext != "so" {
                    continue;
                }
            } else {
                continue;
            }

            log::info!("found shared library: {}", f.path().display());

            // Load .so file.
            fn_loader.load_funcktion(f.path()).context(LoadError)?;
        }

        std::mem::swap(&mut self.loader, &mut RwLock::new(fn_loader));
        log::info!("refresh complete");
        Ok(())
    }

    pub fn add<P: AsRef<Path>>(&self, src_dir: P) -> Result<()> {
        // Build the function.
        let output_so_file = self.builder.build(src_dir).context(BuildFailure)?;

        // Move the shared object file to the managed .so directory.
        let output_file_name = output_so_file.file_name().ok_or(Error::MissingFileName)?; // Fail if no file name because filename is used to find the .so file.

        let so_file_path = self.cfg.shared_object_directory.join(output_file_name);
        fs::rename(&output_so_file, &so_file_path).context(CantMoveSharedObject)?;

        // Load the function from the shared object file.
        let mut loader_w_guard = self.loader.write().map_err(|_e| Error::LoaderLockFailure)?;
        loader_w_guard
            .load_funcktion(so_file_path)
            .context(LoadError)?;
        Ok(())
    }

    pub fn call(&self, function_name: &str) -> Result<()> {
        let loader_r_guard = self.loader.read().map_err(|_e| Error::LoaderLockFailure)?;
        loader_r_guard.call(function_name).context(CallError)
    }
}
