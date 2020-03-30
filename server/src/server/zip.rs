use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn unzip(zip_file: &Path, tgt_dir: &Path) -> io::Result<()> {
    // TODO: Validate that unzip is installed.
    // This throws a bad error.
    let mut child = Command::new("unzip")
        .arg(zip_file.to_str().unwrap())
        .arg("-d")
        .arg(tgt_dir.to_str().unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        let code = status.code().unwrap_or(1);
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("exited with status {}", code),
        ))
    }
}
