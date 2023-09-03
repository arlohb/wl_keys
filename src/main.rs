mod ipc;
mod keyboard;
mod keycode;
mod keymap;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let args = args
        .iter()
        .map(|string| string.as_str())
        .collect::<Vec<_>>();

    match &args[..] {
        ["daemon"] => ipc::daemon()?,
        ["daemon", "stop"] => ipc::send_stop()?,
        ["ui", "open"] => ui::open()?,
        ["ui", "close"] => ui::close()?,
        ["ui", "toggle"] => ui::toggle()?,
        ["key", key_str] => ipc::send_key(key_str.to_string())?,
        _ => anyhow::bail!("Command not recognised"),
    };

    Ok(())
}
