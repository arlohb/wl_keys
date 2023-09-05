use anyhow::Result;
use std::io::Read;

use crate::{config, keyboard::Keyboard, msg::Msg};

/// Kill the daemon if it's currently running.
pub fn kill_old() -> Result<()> {
    // Try and nicely stop a running daemon
    let _ = Msg::Stop.send();

    // Create the socket directory
    std::fs::create_dir_all(config::IPC_DIR)?;

    // Delete the socket file if it exists
    let _ = std::fs::remove_file(config::IPC_PATH);

    Ok(())
}

/// Act on a msg.
pub fn act_on_msg(keyboard: &mut Keyboard, msg: Msg) -> Result<()> {
    match msg {
        Msg::Key(key) => {
            keyboard.key(key, true)?;
            std::thread::sleep(std::time::Duration::from_millis(10));
            keyboard.key(key, false)?;
        }
        Msg::AutoEnable => keyboard.auto_enable(),
        Msg::AutoDisable => keyboard.auto_disable(),
        Msg::AutoToggle => keyboard.auto_toggle(),
        Msg::Stop => {
            return Ok(());
        }
    }

    Ok(())
}

/// Starts the daemon and listens for msgs
pub fn daemon() -> Result<()> {
    let mut keyboard = Keyboard::new()?;

    kill_old()?;

    let socket = std::os::unix::net::UnixListener::bind(config::IPC_PATH)?;
    socket.set_nonblocking(true)?;

    loop {
        if let Ok((mut stream, _)) = socket.accept() {
            let mut data = String::new();
            stream.read_to_string(&mut data)?;
            let msg = serde_json::from_str::<Msg>(&data)?;

            act_on_msg(&mut keyboard, msg)?;
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
        keyboard.roundtrip()?;
    }
}
