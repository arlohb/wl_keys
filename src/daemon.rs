use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use std::{
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};
use tokio::sync::mpsc;
use tonic::{
    transport::{Channel, Server},
    Code, Request, Response, Status,
};

use crate::{config, keyboard::Keyboard};

use self::proto::{
    daemon_client::DaemonClient,
    daemon_server::{Daemon, DaemonServer},
};

/// The grpc API
pub mod proto {
    tonic::include_proto!("wl_keys");
}

/// The implementation of the Daemon grpc trait
pub struct MyDaemon {
    keyboard: Arc<RwLock<Keyboard>>,
    quit_tx: mpsc::Sender<()>,
}

impl MyDaemon {
    /// Create a new `MyDaemon`, passing in the ref to the keyboard and the quit sender.
    pub fn new(keyboard: Arc<RwLock<Keyboard>>, quit_tx: mpsc::Sender<()>) -> Result<Self> {
        Ok(Self { keyboard, quit_tx })
    }

    fn kb_read(&self) -> Result<RwLockReadGuard<Keyboard>, Status> {
        self.keyboard
            .read()
            .map_err(|_| Status::new(Code::Internal, "RwLock poisoned"))
    }

    fn kb_write(&self) -> Result<RwLockWriteGuard<Keyboard>, Status> {
        self.keyboard
            .write()
            .map_err(|_| Status::new(Code::Internal, "RwLock poisoned"))
    }
}

#[tonic::async_trait]
impl Daemon for MyDaemon {
    async fn send_key(&self, req: Request<proto::Key>) -> Result<Response<()>, Status> {
        let key = req.get_ref().key;

        self.kb_read()?
            .key(key, true)
            .map_err(|_| Status::new(Code::Internal, "Wayland request failed"))?;

        tokio::time::sleep(Duration::from_millis(10)).await;

        self.kb_read()?
            .key(key, false)
            .map_err(|_| Status::new(Code::Internal, "Wayland request failed"))?;

        Ok(Response::new(()))
    }

    async fn auto_enable(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.kb_write()?.auto_enable();
        Ok(Response::new(()))
    }

    async fn auto_disable(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.kb_write()?.auto_disable();
        Ok(Response::new(()))
    }

    async fn auto_toggle(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.kb_write()?.auto_toggle();
        Ok(Response::new(()))
    }

    async fn auto_query(&self, _: Request<()>) -> Result<Response<proto::AutoStatus>, Status> {
        Ok(Response::new(proto::AutoStatus {
            enabled: self.kb_read()?.auto_query(),
        }))
    }

    async fn stop(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.quit_tx
            .send(())
            .await
            .map_err(|_| Status::new(Code::Internal, "Quit signal closed"))?;
        Ok(Response::new(()))
    }
}

// No other way in a static
#[allow(clippy::unwrap_used)]
static KEYBOARD: Lazy<Arc<RwLock<Keyboard>>> =
    Lazy::new(|| Arc::new(RwLock::new(Keyboard::new().unwrap())));

/// Run the grpc daemon
pub async fn daemon() -> Result<()> {
    let (quit_tx, mut quit_rx) = mpsc::channel::<()>(1);
    let quit_signal = async {
        quit_rx.recv().await;
    };

    // This result needs to be explicitly typed
    let _x: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_millis(15)).await;

            KEYBOARD
                .write()
                // Have to do this because the PoisonError is not Sync
                .ok()
                .context("RwLock poisoned")?
                .roundtrip()?;
        }
    });

    Server::builder()
        .add_service(DaemonServer::new(MyDaemon::new(KEYBOARD.clone(), quit_tx)?))
        .serve_with_shutdown(config::ADDRESS.parse()?, quit_signal)
        .await?;

    Ok(())
}

/// Get a grpc client
pub async fn client() -> Result<DaemonClient<Channel>> {
    Ok(DaemonClient::connect(format!("https://{}", config::ADDRESS)).await?)
}
