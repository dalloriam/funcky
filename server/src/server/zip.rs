use std::path::Path;
use std::process::{Command, Stdio};

use funck::error::*;

pub fn unzip(zip_file: &Path, tgt_dir: &Path) -> Result<()> {
    // TODO: Validate that unzip is installed.
    // This throws a bad error.
    let mut child = Command::new("unzip")
        .arg(zip_file.to_str().unwrap())
        .arg("-d")
        .arg(tgt_dir.to_str().unwrap())
        //.stdout(Stdio::piped())
        //.stderr(Stdio::piped())
        .spawn()
        .context(BuildError)?;

    let status = child.wait().context(BuildError)?;
    if status.success() {
        Ok(())
    } else {
        let code = status.code().unwrap_or(1);
        Err(Error::BuildFailedStatus)
    }
}
