use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct DropDir {
    path: PathBuf,
}

impl DropDir {
    pub fn new<T: AsRef<Path>>(path: T) -> io::Result<DropDir> {
        fs::create_dir_all(path.as_ref())?;
        Ok(DropDir {
            path: PathBuf::from(path.as_ref()),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for DropDir {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_dir_all(&self.path) {
            log::error!("error dropping dir [{}]: {}", self.path.display(), e);
        }
    }
}
