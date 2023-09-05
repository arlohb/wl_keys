use std::io::Write;

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

use crate::config;

/// The msg sent from the cli to the daemon.
#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Msg {
    /// Send a keycode to press
    Key(u32),
    /// Enable the input detection
    AutoEnable,
    /// Disable the input detection
    AutoDisable,
    /// Toggle the input detection
    AutoToggle,
    /// Stop the daemon
    Stop,
}

impl Msg {
    /// Send the msg to the daemon
    pub fn send(self) -> Result<()> {
        let mut socket = std::os::unix::net::UnixStream::connect(config::IPC_PATH)?;

        let data = serde_json::to_vec(&self)?;
        socket.write_all(&data)?;

        Ok(())
    }
}
