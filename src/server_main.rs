use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use env_logger::Env;

mod server;
use server::Server;

mod funcky;
use funcky::{Config, FunckManager};

const SO_DIR: &str = "./shared_object";

fn block_til_ctrlc() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C Handler");

    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(500));
    }
}

fn init_logger() {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
}

#[tokio::main]
async fn main() {
    init_logger();
    let config = Config {
        shared_object_directory: PathBuf::from(SO_DIR),
        tmp_dir: PathBuf::from("build_tmp"),
    };
    let manager = FunckManager::new(config);

    log::info!("server starting up...");
    let mut server = Server::new(manager);
    if let Err(e) = server.start() {
        log::error!("{}", e);
        return;
    }

    log::info!("HTTP server started");
    block_til_ctrlc();

    log::info!("exit signal received, waiting for server to terminate...");
    if let Err(e) = server.stop().await {
        log::error!("{}", e);
    }
    log::info!("goodbye");
}
