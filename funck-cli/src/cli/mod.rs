mod deploy;

use anyhow::Result;

use clap::Clap;

use rood::cli::OutputManager;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
            Action::Deploy(cmd) => cmd.run(output_manager).await?,
        }

        Ok(())
    }
}
