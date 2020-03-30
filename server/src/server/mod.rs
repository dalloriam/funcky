mod builder;
mod dirmove;
mod filters;
mod handlers;
mod loader;
mod manager;
mod zip;

use dirmove::DirHook;

use builder::FunckBuilder;
use loader::FunckLoader;
pub use manager::{Config, FunckManager};

use std::sync::Arc;

use funck::error::*;

use tokio::sync::oneshot;
use tokio::task::{spawn, JoinHandle};

struct SrvProcess {
    pub join_handle: JoinHandle<()>,
    pub tx_stop: oneshot::Sender<()>,
}

pub struct Server {
    manager: Arc<FunckManager>,
    handle: Option<SrvProcess>,
}

impl Server {
    pub fn new(manager: FunckManager) -> Server {
        Server {
            manager: Arc::new(manager),
            handle: None,
        }
    }

    pub fn start(&mut self) {
        assert!(self.handle.is_none()); // TODO: Proper handling.

        let (tx_stop, rx) = oneshot::channel();
        let (_addr, srv) = warp::serve(filters::all(self.manager.clone()))
            .bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), async {
                rx.await.ok();
            });

        //log::info!("starting http layer");
        let join_handle: JoinHandle<()> = spawn(srv);
        self.handle = Some(SrvProcess {
            join_handle,
            tx_stop,
        });
        //log::info!("http layer ready");
    }

    pub async fn stop(&mut self) -> Result<()> {
        //log::info!("Shutdown signal received.");
        let handle = self.handle.take().ok_or_else(|| Error::ConcurrencyError)?;
        handle
            .tx_stop
            .send(())
            .map_err(|_e| Error::ConcurrencyError)?;

        //log::info!("waiting for server to quit gracefully");
        handle
            .join_handle
            .await
            .map_err(|e| Error::ConcurrencyError)?;

        Ok(())
    }
}
