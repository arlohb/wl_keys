//! A simple onscreen keyboard for wayland.

// I don't think these are at all mandatory for using Rust,
// I just like programming with stricter rules to learn more about the language.
#![warn(missing_docs)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(future_incompatible)]
// Rust makes it obvious when this is happening
// without this lint.
#![allow(clippy::cast_precision_loss)]
// Same as above.
#![allow(clippy::cast_possible_truncation)]
// Same as above.
#![allow(clippy::cast_sign_loss)]
// Too many errors possible to list all of them
#![allow(clippy::missing_errors_doc)]

/// Handles everything about the daemon.
pub mod ipc;
/// The actual virtual keyboard that connects to wayland.
pub mod keyboard;
/// Converts the key string to the xkb code
pub mod keycode;
/// The raw xkb keymap
/// I do not understand this file
pub mod keymap;
/// Manages the eww UI
pub mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let args = args.iter().map(String::as_str).collect::<Vec<_>>();

    match &args[..] {
        ["daemon"] => ipc::daemon()?,
        ["daemon", "stop"] => ipc::send_stop()?,
        ["ui", "open"] => ui::open()?,
        ["ui", "close"] => ui::close()?,
        ["ui", "toggle"] => ui::toggle()?,
        ["key", key_str] => ipc::send_key(key_str)?,
        _ => anyhow::bail!("Command not recognised"),
    };

    Ok(())
}
