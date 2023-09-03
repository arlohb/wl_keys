use anyhow::Result;
use std::process::{Command, Stdio};

/// Open the UI
pub fn open() -> Result<()> {
    Command::new("eww")
        .args(["--config", "eww/", "open", "keyboard"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}

/// Close the UI
pub fn close() -> Result<()> {
    Command::new("eww")
        .args(["--config", "eww/", "close", "keyboard"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}

/// Toggle the UI
pub fn toggle() -> Result<()> {
    Command::new("eww")
        .args(["--config", "eww/", "open", "--toggle", "keyboard"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}
