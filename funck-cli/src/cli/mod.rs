mod deploy;

use clap::Clap;

use rood::cli::OutputManager;

use snafu::{ResultExt, Snafu};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Snafu)]
pub enum Error {
    DeployFailed { source: deploy::Error },
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clap)]
pub enum Action {
    #[clap(name = "deploy")]
    Deploy(deploy::DeployCommand),
}

#[derive(Clap)]
#[clap(version = VERSION, author = "William Dussault")]
pub struct CLI {
    #[clap(short = "v", long = "verbose", global = true)]
    verbose: bool,

    #[clap(subcommand)]
    action: Action,
}

impl CLI {
    pub async fn run(&self) -> Result<()> {
        let output_manager = OutputManager::new(self.verbose);

        match &self.action {
            Action::Deploy(cmd) => cmd.run(output_manager).await.context(DeployFailed)?,
        }

        Ok(())
    }
}
