use std::sync::Arc;

use anyhow::{Context, Result};

use executor::FunckManager;

use tokio::sync::oneshot;
use tokio::task::{spawn, JoinHandle};

use super::filters;

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

    pub fn start(&mut self) -> Result<()> {
        assert!(self.handle.is_none()); // TODO: Handle

        let (tx_stop, rx) = oneshot::channel();
        let (_addr, srv) = warp::serve(filters::all(self.manager.clone()))
            .bind_with_graceful_shutdown(([0, 0, 0, 0], 3030), async {
                rx.await.ok();
            });

        log::info!("starting http layer");
        let join_handle: JoinHandle<()> = spawn(srv);
        self.handle = Some(SrvProcess {
            join_handle,
            tx_stop,
        });
        log::info!("http layer ready");

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        log::info!("Shutdown signal received.");
        let handle = self.handle.take().unwrap(); // TODO: Handle.

        handle.tx_stop.send(()).unwrap(); // TODO: Handle empty error.

        log::info!("waiting for server to quit gracefully");
        handle.join_handle.await.context("Shutdown error")?;

        Ok(())
    }
}
