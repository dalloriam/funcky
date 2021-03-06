use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::iter::FromIterator;
use std::path::PathBuf;

use clap::Clap;

use reqwest::multipart;
use rood::cli::OutputManager;

use snafu::{ResultExt, Snafu};

use tempfile::TempDir;

use crate::sysutil::zip;

const DEPLOYABLE_FILES: [&str; 3] = ["Cargo.toml", "Cargo.lock", "src"];

#[derive(Debug, Snafu)]
pub enum Error {
    FailedToGetCurrentDirectory { source: io::Error },
    FailedToListFiles { source: io::Error },
    FailedToCompress { source: io::Error },
    FailedToReadBundle { source: io::Error },
    FailedToUploadBundle { source: reqwest::Error },
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clap)]
pub struct DeployCommand {
    /// The path of the function to deploy.
    #[clap(default_value = ".")]
    path: PathBuf,

    /// The host of the funck server.
    #[clap(default_value = "localhost")]
    host: String,

    /// The port of the funck server.
    #[clap(default_value = "3030")]
    port: u16,
}

impl DeployCommand {
    fn get_deployable_files(&self) -> io::Result<Vec<PathBuf>> {
        let fileset: HashSet<String> =
            HashSet::from_iter(DEPLOYABLE_FILES.iter().map(|s| String::from(*s)));

        Ok(fs::read_dir(&self.path)?
            .filter_map(|f| f.ok())
            .filter(|f| fileset.contains(f.file_name().to_string_lossy().as_ref()))
            .map(|f| f.path().clone())
            .collect())
    }

    async fn zip_and_upload(&self, output: OutputManager, name: &OsStr) -> Result<()> {
        output.step("Create source bundle");
        let files = self.get_deployable_files().context(FailedToListFiles)?;

        let temp_dir = TempDir::new().context(FailedToCompress)?;
        let zip_path = temp_dir
            .path()
            .join(&format!("{}.zip", name.to_string_lossy()));
        zip::zip_directory(&zip_path, files.as_ref()).context(FailedToCompress)?;
        output
            .push()
            .progress(&format!("Source bundle => {}", zip_path.display()));

        let fmted_url = format!("http://{}:{}/_funck_add", self.host, self.port);
        output.step(&format!("Upload bundle to {}", fmted_url));
        let client = reqwest::Client::new();

        let form = multipart::Form::new().part(
            "src",
            multipart::Part::bytes(fs::read(&zip_path).context(FailedToReadBundle)?)
                .file_name(zip_path.to_string_lossy().to_string()),
        );

        let resp = client
            .post(&fmted_url)
            .multipart(form)
            .send()
            .await
            .context(FailedToUploadBundle)?;

        // TODO: Handle response.

        Ok(())
    }

    pub async fn run(&self, output: OutputManager) -> Result<()> {
        let tgt_dir = std::fs::canonicalize(&self.path).context(FailedToGetCurrentDirectory)?;
        let name = tgt_dir.file_name().unwrap_or("new_funcktion".as_ref());
        output.step(&format!("Deploy [{}]", name.to_string_lossy()));

        self.zip_and_upload(output.push(), name).await?;

        output.success("OK");
        Ok(())
    }
}
