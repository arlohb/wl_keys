use anyhow::{Context, Result};
use std::process::{Command, Stdio};

fn spawn_silent(cmd: impl Into<String>) -> Result<()> {
    let cmd: String = cmd.into();
    let mut parts = cmd.split(' ');

    Command::new(parts.next().context("Empty string given")?)
        .args(parts)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}

/// Open the UI
pub fn open() -> Result<()> {
    spawn_silent("eww --config eww/ open keyboard")
}

/// Close the UI
pub fn close() -> Result<()> {
    spawn_silent("eww --config eww/ close keyboard")
}

/// Toggle the UI
pub fn toggle() -> Result<()> {
    spawn_silent("eww --config eww/ open --toggle keyboard")
}
