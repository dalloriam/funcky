mod cli;

use rood::cli::OutputManager;

#[tokio::main]
async fn main() {
    if let Err(e) = CLI::parse().run().await {
        OutputManager::new(true).error(&e.to_string())
    }
}
