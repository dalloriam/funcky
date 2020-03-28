use std::env;
use std::path::{Path, PathBuf};

use funck::error::*;

pub struct DirHook {
    return_path: PathBuf,
}

impl DirHook {
    pub fn new<P: AsRef<Path>>(new_path: P) -> Result<DirHook> {
        let cur_dir = env::current_dir().unwrap(); // TODO: Handle.
        env::set_current_dir(new_path.as_ref()).context(DirNotExist)?;
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
