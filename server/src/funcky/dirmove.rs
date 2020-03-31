use std::env;
use std::io;
use std::path::{Path, PathBuf};

pub struct DirHook {
    return_path: PathBuf,
}

impl DirHook {
    pub fn new<P: AsRef<Path>>(new_path: P) -> io::Result<DirHook> {
        let cur_dir = env::current_dir()?;
        env::set_current_dir(new_path.as_ref())?;
        Ok(DirHook {
            return_path: cur_dir,
        })
    }
}

impl Drop for DirHook {
    fn drop(&mut self) {
        if env::set_current_dir(&self.return_path).is_err() {
            eprintln!("Error jumping back to directory.");
        }
    }
}
