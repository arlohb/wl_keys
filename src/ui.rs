use std::process::{Command, Stdio};

pub fn open() -> anyhow::Result<()> {
    Command::new("eww")
        .args(["--config", "eww/", "open", "keyboard"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}

pub fn close() -> anyhow::Result<()> {
    Command::new("eww")
        .args(["--config", "eww/", "close", "keyboard"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}
