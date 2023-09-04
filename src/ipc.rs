use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use std::io::{Read, Write};

use crate::{keyboard::Keyboard, keycode::str_to_key};

#[derive(Serialize, Deserialize, Clone, Copy)]
enum Msg {
    Key(u32),
    Stop,
}

const IPC_DIR: &str = "/tmp/wl_keys";
const IPC_PATH: &str = "/tmp/wl_keys/socket.sock";

/// Starts the daemon and listens for msgs
pub fn daemon() -> Result<()> {
    let mut keyboard = Keyboard::new()?;

    let _ = send_stop();
    std::fs::create_dir_all(IPC_DIR)?;
    let _ = std::fs::remove_file(IPC_PATH);
    let socket = std::os::unix::net::UnixListener::bind(IPC_PATH)?;
    socket.set_nonblocking(true)?;

    loop {
        if let Ok((mut stream, _)) = socket.accept() {
            let mut data = String::new();
            stream.read_to_string(&mut data)?;

            let msg = serde_json::from_str::<Msg>(&data)?;

            match msg {
                Msg::Key(key) => {
                    keyboard.key(key, true)?;
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    keyboard.key(key, false)?;
                }
                Msg::Stop => {
                    return Ok(());
                }
            };
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
        keyboard.roundtrip()?;
    }
}

fn send_msg(msg: Msg) -> Result<()> {
    let mut socket = std::os::unix::net::UnixStream::connect(IPC_PATH)?;

    let data = serde_json::to_vec(&msg)?;
    socket.write_all(&data)?;

    Ok(())
}

/// Send a key press of a key string to the daemon
pub fn send_key(key_str: &str) -> Result<()> {
    let key = str_to_key(key_str)?;
    send_msg(Msg::Key(key))?;

    Ok(())
}

/// Send the stop msg to the daemon
pub fn send_stop() -> Result<()> {
    send_msg(Msg::Stop)?;

    Ok(())
}
