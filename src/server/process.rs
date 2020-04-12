use std::sync::Arc;

use funcky::FunckManager;

use snafu::{ensure, ResultExt, Snafu};

use tokio::sync::oneshot;
use tokio::task::{spawn, JoinError, JoinHandle};

use super::filters;

#[derive(Debug, Snafu)]
pub enum ServerError {
    DoubleStartError,
    ShutdownRequestError,
    ShutdownError { source: JoinError },
    StopWithoutStartError,
}

pub type Result<T, E = ServerError> = std::result::Result<T, E>;

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
        ensure!(self.handle.is_none(), DoubleStartError);

        let (tx_stop, rx) = oneshot::channel();
        let (_addr, srv) = warp::serve(filters::all(self.manager.clone()))
            .bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), async {
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
        let handle = self
            .handle
            .take()
            .ok_or_else(|| ServerError::StopWithoutStartError)?;
        ensure!(handle.tx_stop.send(()).is_ok(), ShutdownRequestError);

        log::info!("waiting for server to quit gracefully");
        handle.join_handle.await.context(ShutdownError)?;

        Ok(())
    }
}
