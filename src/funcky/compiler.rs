use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{mpsc, Mutex};
use std::thread;

use snafu::{ensure, ResultExt, Snafu};

use super::{DirHook, DropDir};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to move to directory [{}]: {}", path.display(), source))]
    SwitchDirError { source: io::Error, path: PathBuf },

    #[snafu(display("Failed to spawn build command: {}", source))]
    BuildSpawnError { source: io::Error },

    #[snafu(display("Error waiting for build command: {}", source))]
    BuildJoinError { source: io::Error },

    #[snafu(display("Build command exited with non-zero status code: {}", code))]
    ExitCodeNonZero { code: i32 },

    #[snafu(display("The final shared object file path ({}) is invalid: {}", path.display(), source))]
    InvalidOutputPath { source: io::Error, path: PathBuf },

    #[snafu(display("Couldn't send the job to the compilation worker: {}", source))]
    JobDispatchError { source: mpsc::SendError<Request> },

    #[snafu(display("Compile worker is not started"))]
    WorkerNotStarted,
}

pub struct Request {
    pub source_directory: DropDir,
}

impl Request {
    pub fn new(source_dir: DropDir) -> Request {
        Request {
            source_directory: source_dir,
        }
    }

    /// Execute a compilation job.
    pub fn execute(&self) -> Result<PathBuf, Error> {
        log::info!(
            "started compile job for {}",
            self.source_directory.path().display()
        );

        let _hk = DirHook::new(self.source_directory.path()).context(SwitchDirError {
            path: self.source_directory.path(),
        })?;

        let project_name = self
            .source_directory
            .path()
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

        log::info!("compiled {} successfully", so_file_path.display());
        Ok(so_file_path)
    }
}

struct WorkHandle {
    pub handle: thread::JoinHandle<()>,
    pub job_tx: Mutex<mpsc::Sender<Request>>,
}

pub struct Worker {
    handle: Option<WorkHandle>,
    shared_object_destination: PathBuf,
}

impl Worker {
    pub fn new<P: AsRef<Path>>(shared_object_path: P) -> Worker {
        let shared_object_destination = PathBuf::from(shared_object_path.as_ref());
        Worker {
            handle: None,
            shared_object_destination,
        }
    }

    pub fn start(&mut self) -> mpsc::Receiver<PathBuf> {
        assert!(self.handle.is_none()); // TODO: Handle properly
        let (job_tx, job_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();
        let dst_path = self.shared_object_destination.clone();
        let handle = thread::spawn(move || Worker::compile_loop(job_rx, result_tx, dst_path));
        let work_handle = WorkHandle {
            handle,
            job_tx: Mutex::new(job_tx),
        };
        self.handle = Some(work_handle);
        result_rx
    }

    fn compile_loop(
        incoming_jobs: mpsc::Receiver<Request>,
        result_tx: mpsc::Sender<PathBuf>,
        so_out_dir: PathBuf,
    ) {
        loop {
            match incoming_jobs.recv() {
                Ok(job) => match job.execute() {
                    Ok(so_file) => {
                        // Move the so_file from the temp dir to the dest dir.
                        let fname_maybe = so_file.file_name();
                        if fname_maybe.is_none() {
                            log::error!("shared object file has no file name");
                            continue;
                        }

                        let fname = fname_maybe.unwrap();
                        let dst_so_file = so_out_dir.join(fname);
                        if let Err(e) = fs::rename(so_file, &dst_so_file) {
                            log::error!("error moving shared object file: {}", e);
                            continue;
                        }

                        if let Err(e) = result_tx.send(dst_so_file) {
                            log::error!("error sending result: {}", e);
                        }
                    }
                    Err(e) => log::error!("compile error: {}", e),
                },
                Err(_e) => log::info!("compile job channel disconnected"),
            };
        }
    }

    pub fn new_job(&self, job: Request) -> Result<(), Error> {
        if let Some(worker) = &self.handle {
            let mtx_handle = worker.job_tx.lock().unwrap(); // TODO: Handle
            mtx_handle.send(job).context(JobDispatchError)
        } else {
            Err(Error::WorkerNotStarted)
        }
    }
}
