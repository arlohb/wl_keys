mod keyboard;
mod keycode;
mod keymap;

use anyhow::Context;
use keycode::str_to_key;

fn main() -> anyhow::Result<()> {
    let key_str = std::env::args().nth(1).context("Need a key argument")?;
    let key = str_to_key(&key_str);

    let keyboard = keyboard::Keyboard::new()?;

    keyboard.key(key, true)?;

    std::thread::sleep(std::time::Duration::from_millis(10));
    keyboard.key(key, false)?;

    Ok(())
}
