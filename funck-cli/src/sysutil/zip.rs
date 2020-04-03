use std::path::{Path, PathBuf};

use std::io;
use std::process::{Command, Stdio};

pub fn zip_directory(zip_path: &Path, files_in_dir: &[PathBuf]) -> io::Result<()> {
    let mut command = Command::new("zip")
        .arg("-r")
        .arg(zip_path)
        .args(files_in_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let status = command.wait()?;

    if status.success() {
        Ok(())
    } else {
        let code = status.code().unwrap_or(1);
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("non-zero status code: {}", code),
        ))
    }
}
