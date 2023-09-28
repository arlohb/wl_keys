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

/// The program arguments with clap
pub mod args;
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

use anyhow::{Context, Result};
use args::{AutoCmd, Command, DaemonCmd, ModCmd, UiCmd};
use clap::Parser;
use daemon::{
    client,
    proto::{ModMsg, Modifier},
};
use keycode::str_to_key;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    let args = Command::parse();

    match args {
        Command::Daemon(cmd) => match cmd {
            DaemonCmd::Start => daemon::daemon().await?,
            DaemonCmd::Stop => {
                let _ = client().await?.stop(()).await?;
            }
            DaemonCmd::Protocols => {
                let protocols = client().await?.get_protocols(()).await?;
                for protocol in &protocols.get_ref().protocols {
                    println!("{protocol}");
                }
            }
        },
        Command::Auto(cmd) => match cmd {
            AutoCmd::Enable => {
                let _ = client().await?.auto_enable(()).await?;
            }
            AutoCmd::Disable => {
                let _ = client().await?.auto_disable(()).await?;
            }
            AutoCmd::Toggle => {
                let _ = client().await?.auto_toggle(()).await?;
            }
            AutoCmd::Query => {
                let enabled = client().await?.auto_query(()).await?.get_ref().enabled;
                println!("{enabled}");
            }
        },
        Command::Ui(cmd) => match cmd {
            UiCmd::Open => ui::open()?,
            UiCmd::Close => ui::close()?,
            UiCmd::Toggle => ui::toggle()?,
        },
        Command::Mod(cmd) => match cmd {
            ModCmd::Press { modifier: mod_str } => {
                let _ = client()
                    .await?
                    .mod_press(ModMsg {
                        modifier: Modifier::from_str_name(&mod_str)
                            .context("Invalid modifier")?
                            .into(),
                    })
                    .await?;
            }
            ModCmd::Release { modifier: mod_str } => {
                let _ = client()
                    .await?
                    .mod_release(ModMsg {
                        modifier: Modifier::from_str_name(&mod_str)
                            .context("Invalid modifier")?
                            .into(),
                    })
                    .await?;
            }
            ModCmd::Toggle { modifier: mod_str } => {
                let _ = client()
                    .await?
                    .mod_toggle(ModMsg {
                        modifier: Modifier::from_str_name(&mod_str)
                            .context("Invalid modifier")?
                            .into(),
                    })
                    .await?;
            }
            ModCmd::Query { modifier: mod_str } => {
                let pressed = client()
                    .await?
                    .mod_query(ModMsg {
                        modifier: Modifier::from_str_name(&mod_str)
                            .context("Invalid modifier")?
                            .into(),
                    })
                    .await?
                    .get_ref()
                    .pressed;
                println!("{pressed}");
            }
        },
        Command::Key { key: key_str } => {
            let _ = client()
                .await?
                .send_key(daemon::proto::Key {
                    key: str_to_key(&key_str)?,
                })
                .await?;
        }
    };

    Ok(())
}
