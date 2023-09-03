use anyhow::Result;
use std::process::{Command, Stdio};

pub fn open() -> Result<()> {
    Command::new("eww")
        .args(["--config", "eww/", "open", "keyboard"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}

pub fn close() -> Result<()> {
    Command::new("eww")
        .args(["--config", "eww/", "close", "keyboard"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}

pub fn toggle() -> Result<()> {
    Command::new("eww")
        .args(["--config", "eww/", "open", "--toggle", "keyboard"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}
