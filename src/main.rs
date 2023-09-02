mod keyboard;
mod keycode;
mod keymap;
mod ui;

use keycode::str_to_key;

fn key(key_str: &str) -> anyhow::Result<()> {
    let key = str_to_key(key_str);

    let keyboard = keyboard::Keyboard::new()?;

    keyboard.key(key, true)?;

    std::thread::sleep(std::time::Duration::from_millis(10));
    keyboard.key(key, false)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let args = args
        .iter()
        .map(|string| string.as_str())
        .collect::<Vec<_>>();

    match &args[..] {
        ["ui", "open"] => ui::open()?,
        ["ui", "close"] => ui::close()?,
        ["key", key_str] => key(key_str)?,
        _ => anyhow::bail!("Command not recognised"),
    };

    Ok(())
}
