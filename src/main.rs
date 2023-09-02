mod keyboard;
mod keymap;

fn main() -> anyhow::Result<()> {
    let keyboard = keyboard::Keyboard::new()?;

    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        keyboard.key(input_event_codes::KEY_W!(), true)?;

        std::thread::sleep(std::time::Duration::from_millis(10));
        keyboard.key(input_event_codes::KEY_W!(), false)?;
    }

    Ok(())
}
