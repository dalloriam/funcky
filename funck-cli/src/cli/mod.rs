use clap::Clap;

use snafu::Snafu;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Snafu)]
enum Error {}

type Result<T> = std::result::Result<T, Error>;

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
        Ok(())
    }
}
