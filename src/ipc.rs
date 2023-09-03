use serde_derive::{Deserialize, Serialize};
use std::io::{Read, Write};

use crate::{keyboard::Keyboard, keycode::str_to_key};

#[derive(Serialize, Deserialize)]
enum Msg {
    Key(u32),
}

const IPC_DIR: &str = "/tmp/wl_keys";
const IPC_PATH: &str = "/tmp/wl_keys/socket.sock";

pub fn daemon() -> anyhow::Result<()> {
    let keyboard = Keyboard::new()?;

    std::fs::create_dir_all(IPC_DIR)?;
    let _ = std::fs::remove_file(IPC_PATH);
    let socket = std::os::unix::net::UnixListener::bind(IPC_PATH)?;

    for stream in socket.incoming() {
        let mut stream = stream?;

        let mut data = String::new();
        stream.read_to_string(&mut data)?;

        let msg = serde_json::from_str::<Msg>(&data)?;

        match msg {
            Msg::Key(key) => {
                keyboard.key(key, true)?;
                std::thread::sleep(std::time::Duration::from_millis(10));
                keyboard.key(key, false)?;
            }
        };
    }

    Ok(())
}

pub fn client(key_str: String) -> anyhow::Result<()> {
    let mut socket = std::os::unix::net::UnixStream::connect(IPC_PATH)?;

    let key = str_to_key(&key_str);
    let msg = Msg::Key(key);

    let data = serde_json::to_vec(&msg)?;
    socket.write_all(&data)?;

    Ok(())
}
