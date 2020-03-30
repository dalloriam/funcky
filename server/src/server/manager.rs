use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use funck::error::*;

use super::{FunckBuilder, FunckLoader};

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
    pub fn new(cfg: Config) -> FunckManager {
        // Ensure shared object directory exists.
        if !cfg.shared_object_directory.exists() {
            fs::create_dir_all(&cfg.shared_object_directory).unwrap(); // TODO: Handle
        }

        if !cfg.tmp_dir.exists() {
            fs::create_dir_all(&cfg.tmp_dir).unwrap();
        }
        FunckManager {
            builder: FunckBuilder::new(),
            cfg,
            loader: RwLock::new(FunckLoader::new()),
        }
    }

    pub fn add<P: AsRef<Path>>(&self, src_dir: P) -> Result<()> {
        // Build the function.
        let output_so_file = self.builder.build(src_dir)?;

        // Move the shared object file to the managed so file directory.
        let so_file_path = self
            .cfg
            .shared_object_directory
            .join(output_so_file.file_name().unwrap()); // TODO: Handle.
        fs::rename(&output_so_file, &so_file_path).unwrap();

        // Load the function from the shared object file.
        let mut loader_w_guard = self
            .loader
            .write()
            .map_err(|_e| Error::ConcurrencyError {})?;
        loader_w_guard.load_funcktion(so_file_path)?;
        Ok(())
    }

    pub fn call(&self, function_name: &str) -> Result<()> {
        let loader_r_guard = self
            .loader
            .read()
            .map_err(|_e| Error::ConcurrencyError {})?;

        loader_r_guard.call(function_name)
    }
}
