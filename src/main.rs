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
// I only do this on purpose
#![allow(clippy::module_name_repetitions)]
// I think this lint is falsely triggering
#![allow(clippy::significant_drop_tightening)]

/// Values like the socket file location.
pub mod config;
/// The daemon.
pub mod daemon;
/// The actual virtual keyboard that connects to wayland.
pub mod keyboard;
/// Converts the key string to the xkb code
pub mod keycode;
/// Manages the eww UI
pub mod ui;

use anyhow::Result;
use daemon::client;
use keycode::str_to_key;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let args = args.iter().map(String::as_str).collect::<Vec<_>>();

    match &args[..] {
        ["daemon"] => daemon::daemon().await?,
        ["daemon", "stop"] => {
            let _ = client().await?.stop(()).await?;
        }
        ["auto", "enable"] => {
            let _ = client().await?.auto_enable(()).await?;
        }
        ["auto", "disable"] => {
            let _ = client().await?.auto_disable(()).await?;
        }
        ["auto", "toggle"] => {
            let _ = client().await?.auto_toggle(()).await?;
        }
        ["auto", "query"] => {
            let enabled = client().await?.auto_query(()).await?.get_ref().enabled;
            println!("{enabled}");
        }
        ["ui", "open"] => ui::open()?,
        ["ui", "close"] => ui::close()?,
        ["ui", "toggle"] => ui::toggle()?,
        ["key", key_str] => {
            let _ = client()
                .await?
                .send_key(daemon::proto::Key {
                    key: str_to_key(key_str)?,
                })
                .await?;
        }
        _ => anyhow::bail!("Command not recognised"),
    };

    Ok(())
}
