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

use crate::{
    config,
    keyboard::Keyboard,
    proto::{
        self,
        daemon_client::DaemonClient,
        daemon_server::{Daemon, DaemonServer},
    },
};

/// This allows me to use:
/// `.internal("Wayland request failed")`
/// on a `Result` instead of
/// `.map_err(|_| Status::new(Code::Internal, "Wayland request failed"))`
trait InternalError<T> {
    /// Map the error to a `Status` with a `Code::Internal` and given msg.
    fn internal(self, msg: impl Into<String>) -> Result<T, Status>;
}

impl<T, E> InternalError<T> for Result<T, E> {
    fn internal(self, msg: impl Into<String>) -> Result<T, Status> {
        self.map_err(|_| Status::new(Code::Internal, msg))
    }
}

trait ToResponse {
    fn to_res(self) -> Response<Self>
    where
        Self: Sized,
    {
        Response::new(self)
    }
}

impl<T> ToResponse for T {}

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
        self.keyboard.read().internal("RwLock poisoned")
    }

    fn kb_write(&self) -> Result<RwLockWriteGuard<Keyboard>, Status> {
        self.keyboard.write().internal("RwLock poisoned")
    }
}

#[tonic::async_trait]
impl Daemon for MyDaemon {
    async fn send_key(&self, req: Request<proto::Key>) -> Result<Response<()>, Status> {
        let key = req.get_ref().key;

        self.kb_read()?
            .key(key, true)
            .internal("Wayland request failed")?;

        tokio::time::sleep(Duration::from_millis(10)).await;

        self.kb_read()?
            .key(key, false)
            .internal("Wayland request failed")?;

        self.kb_write()?
            .mod_release_all()
            .internal("Wayland request failed")?;

        Ok(().to_res())
    }

    async fn auto_enable(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.kb_write()?.auto_enable();
        Ok(().to_res())
    }

    async fn auto_disable(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.kb_write()?.auto_disable();
        Ok(().to_res())
    }

    async fn auto_toggle(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.kb_write()?.auto_toggle();
        Ok(().to_res())
    }

    async fn auto_query(&self, _: Request<()>) -> Result<Response<proto::AutoStatus>, Status> {
        Ok(proto::AutoStatus {
            enabled: self.kb_read()?.auto_query(),
        }
        .to_res())
    }

    async fn mod_press(&self, req: Request<proto::ModMsg>) -> Result<Response<()>, Status> {
        let modifier = req.get_ref().modifier();
        self.kb_write()?
            .mod_press(modifier)
            .internal("Wayland request failed")?;
        Ok(().to_res())
    }

    async fn mod_release(&self, req: Request<proto::ModMsg>) -> Result<Response<()>, Status> {
        let modifier = req.get_ref().modifier();
        self.kb_write()?
            .mod_release(modifier)
            .internal("Wayland request failed")?;
        Ok(().to_res())
    }

    async fn mod_toggle(&self, req: Request<proto::ModMsg>) -> Result<Response<()>, Status> {
        let modifier = req.get_ref().modifier();
        self.kb_write()?
            .mod_toggle(modifier)
            .internal("Wayland request failed")?;
        Ok(().to_res())
    }

    async fn mod_query(
        &self,
        req: Request<proto::ModMsg>,
    ) -> Result<Response<proto::ModStatus>, Status> {
        let modifier = req.get_ref().modifier();
        let pressed = self.kb_read()?.mod_query(modifier);
        Ok(proto::ModStatus { pressed }.to_res())
    }

    async fn stop(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.quit_tx.send(()).await.internal("Quit signal closed")?;
        Ok(().to_res())
    }

    async fn get_protocols(&self, _: Request<()>) -> Result<Response<proto::Protocols>, Status> {
        Ok(proto::Protocols {
            protocols: self.kb_read()?.protocols(),
        }
        .to_res())
    }
}

// No other way in a static
#[allow(clippy::unwrap_used)]
static KEYBOARD: Lazy<Arc<RwLock<Keyboard>>> =
    Lazy::new(|| Arc::new(RwLock::new(Keyboard::new().unwrap())));

/// Run the grpc daemon
pub async fn daemon() -> Result<()> {
    // Stop the daemon if its already running
    if let Ok(mut client) = client().await {
        let _ = client.stop(()).await;
    }

    let (quit_tx, mut quit_rx) = mpsc::channel::<()>(1);
    let quit_signal = async {
        quit_rx.recv().await;
    };

    tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_millis(15)).await;

            KEYBOARD
                .write()
                // Have to do this because the PoisonError is not Sync
                .ok()
                .context("RwLock poisoned")?
                .roundtrip()?;
        }

        // This avoids having to explicitely type the return value
        #[allow(unreachable_code)]
        Result::<()>::Ok(())
    });

    Server::builder()
        .add_service(DaemonServer::new(MyDaemon::new(KEYBOARD.clone(), quit_tx)?))
        .serve_with_shutdown(config::ADDRESS.parse()?, quit_signal)
        .await?;

    Ok(())
}

/// Get a grpc client
pub async fn client() -> Result<DaemonClient<Channel>> {
    let addr = format!("https://{}", config::ADDRESS);
    DaemonClient::connect(addr).await.map_err(Into::into)
}
