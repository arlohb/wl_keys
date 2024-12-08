use anyhow::{anyhow, Context, Result};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

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

fn is_config_here(path: impl AsRef<Path>) -> Result<bool> {
    Ok(
        std::fs::read_dir(path)?
            .any(|entry| entry.map(|e| e.file_name() == "eww").unwrap_or(false)),
    )
}

fn find_config_path(mut path: PathBuf) -> Result<PathBuf> {
    if is_config_here(&path)? {
        return Ok(path);
    }

    if !path.pop() {
        return Err(anyhow!("Can't find eww config"));
    }

    find_config_path(path)
}

fn config_path() -> Result<String> {
    let mut exe = std::env::current_exe()?;
    exe.pop();
    let path_buf = find_config_path(exe)?;
    let path = path_buf
        .to_str()
        .ok_or_else(|| anyhow!("Exe path was invalid unicode"))?;
    Ok(format!("{path}/eww"))
}

/// Open the UI
pub fn open() -> Result<()> {
    spawn_silent(format!("eww --config {} open keyboard", config_path()?))
}

/// Close the UI
pub fn close() -> Result<()> {
    spawn_silent(format!("eww --config {} close keyboard", config_path()?))
}

/// Toggle the UI
pub fn toggle() -> Result<()> {
    spawn_silent(format!(
        "eww --config {} open --toggle keyboard",
        config_path()?
    ))
}
