mod cli;
mod sysutil;

use clap::Clap;

use rood::cli::OutputManager;

use cli::CLI;

#[tokio::main]
async fn main() {
    if let Err(e) = CLI::parse().run().await {
        OutputManager::new(true).error(&e.to_string())
    }
}
