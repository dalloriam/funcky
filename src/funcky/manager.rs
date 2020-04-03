use std::collections::HashMap;
use std::fs;
use std::io;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;

use funck::{Request, Response};

use snafu::{ResultExt, Snafu};

use super::compiler;
pub use super::loader::Error as LoaderError;
use super::{FunckLoader, FuncktionEntry, Status, StatusTracker};

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
    status_tracker: Arc<StatusTracker>,
}

impl FunckManager {
    pub fn new(cfg: Config) -> Result<FunckManager> {
        FunckManager::ensure_dirs_exist(&cfg)?;

        let stat_tracker = Arc::new(StatusTracker::new());
        let compile_worker =
            compiler::Worker::new(&cfg.shared_object_directory, stat_tracker.clone());

        let mut manager = FunckManager {
            cfg,
            compile_worker,
            loader: Arc::new(RwLock::new(FunckLoader::new())),
            result_thread_handle: None,
            status_tracker: stat_tracker,
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
        let tracker = self.status_tracker.clone();
        let result_thread_handle = thread::spawn(move || {
            FunckManager::shared_object_install_loop(loader, tracker, so_directory, result_rx)
        });
        self.result_thread_handle = Some(result_thread_handle);
    }

    fn shared_object_install_loop(
        loader: Arc<RwLock<FunckLoader>>,
        status: Arc<StatusTracker>,
        so_dir: PathBuf,
        so_rx: mpsc::Receiver<compiler::Response>,
    ) {
        loop {
            match so_rx.recv() {
                Ok(res) => {
                    // Move the shared object file to the managed .so directory.
                    let output_file_name = res
                        .so_path
                        .file_name()
                        .ok_or(Error::MissingFileName)
                        .unwrap(); // TODO: HANDLE Fail if no file name because filename is used to find the .so file.

                    let so_file_path = so_dir.join(output_file_name);
                    fs::rename(&res.so_path, &so_file_path)
                        .context(CantMoveSharedObject)
                        .unwrap(); // TODO: Handle

                    let mut loader_guard = loader.write().unwrap(); // TODO: Handle.
                    loader_guard.load_funcktion(&res.so_path).unwrap();
                    // TODO: Handle.

                    status.update_status(&res.job_name, Status::Ready);
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
            let name = fn_loader.load_funcktion(f.path()).context(LoadError)?;
            self.status_tracker.new_with_status(&name, Status::Ready);
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
        self.status_tracker.add(&src_dir.name);
        self.compile_worker
            .new_job(compiler::Request::new(src_dir))
            .unwrap(); // TODO: Handle.
        Ok(())
    }

    pub fn call(&self, function_name: &str, request: Request) -> Result<Response> {
        let loader_r_guard = self.loader.read().map_err(|_e| Error::LoaderLockFailure)?;
        loader_r_guard
            .call(function_name, request)
            .context(CallError)
    }

    pub fn stat(&self) -> HashMap<String, FuncktionEntry> {
        self.status_tracker.all()
    }
}
