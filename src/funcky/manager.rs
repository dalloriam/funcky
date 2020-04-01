use std::fs;
use std::io;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;

use snafu::{ResultExt, Snafu};

use super::compiler;
pub use super::loader::Error as LoaderError;
use super::FunckLoader;

#[derive(Debug, Snafu)]
pub enum Error {
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
    pub cfg: Config,
    compile_worker: compiler::Worker,
    loader: Arc<RwLock<FunckLoader>>,
    result_thread_handle: Option<thread::JoinHandle<()>>,
}

impl FunckManager {
    pub fn new(cfg: Config) -> Result<FunckManager> {
        FunckManager::ensure_dirs_exist(&cfg)?;

        let compile_worker = compiler::Worker::new(&cfg.shared_object_directory);

        let mut manager = FunckManager {
            cfg,
            compile_worker,
            loader: Arc::new(RwLock::new(FunckLoader::new())),
            result_thread_handle: None,
        };

        // Perform initial loading of .so files.
        manager.refresh_shared_objects()?;
        Ok(manager)
    }

    pub fn start(&mut self) {
        assert!(self.result_thread_handle.is_none()); // TODO: Proper handle.
        let result_rx = self.compile_worker.start();
        let loader = self.loader.clone();
        let so_directory = self.cfg.shared_object_directory.clone();
        let result_thread_handle = thread::spawn(move || {
            FunckManager::shared_object_install_loop(loader, so_directory, result_rx)
        });
        self.result_thread_handle = Some(result_thread_handle);
    }

    fn shared_object_install_loop(
        loader: Arc<RwLock<FunckLoader>>,
        so_dir: PathBuf,
        so_rx: mpsc::Receiver<PathBuf>,
    ) {
        loop {
            match so_rx.recv() {
                Ok(output_so_file) => {
                    // Move the shared object file to the managed .so directory.
                    let output_file_name = output_so_file
                        .file_name()
                        .ok_or(Error::MissingFileName)
                        .unwrap(); // TODO: HANDLE Fail if no file name because filename is used to find the .so file.

                    let so_file_path = so_dir.join(output_file_name);
                    fs::rename(&output_so_file, &so_file_path)
                        .context(CantMoveSharedObject)
                        .unwrap(); // TODO: Handle

                    let mut loader_guard = loader.write().unwrap(); // TODO: Handle.
                    loader_guard.load_funcktion(so_file_path).unwrap(); // TODO: Handle.
                }
                Err(_e) => {
                    log::info!("shared object installer disconnected");
                    break;
                }
            }
        }
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

        {
            let mut loader_lock = self.loader.write().unwrap(); // TODO: Handle.
            std::mem::swap(loader_lock.deref_mut(), &mut fn_loader);
        }

        log::info!("refresh complete");
        Ok(())
    }

    pub fn add(&self, src_dir: super::DropDir) -> Result<()> {
        // Build the function.
        self.compile_worker
            .new_job(compiler::Request::new(src_dir))
            .unwrap(); // TODO: Handle.
        Ok(())
    }

    pub fn call(&self, function_name: &str) -> Result<()> {
        let loader_r_guard = self.loader.read().map_err(|_e| Error::LoaderLockFailure)?;
        loader_r_guard.call(function_name).context(CallError)
    }
}
