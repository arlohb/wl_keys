use std::{
    io::{Seek, SeekFrom, Write},
    os::fd::IntoRawFd,
};

use wayland_client::{
    delegate_noop,
    protocol::{wl_registry, wl_seat::WlSeat},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
    zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
};

mod keymap;

struct App;

delegate_noop!(App: ignore WlSeat);
delegate_noop!(App: ignore ZwpVirtualKeyboardManagerV1);
delegate_noop!(App: ignore ZwpVirtualKeyboardV1);

impl Dispatch<wl_registry::WlRegistry, ()> for App {
    fn event(
        _state: &mut Self,
        _registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _user_state: &(),
        _conn: &Connection,
        _qh: &QueueHandle<App>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            println!("[{}] {} (v{})", name, interface, version);
        }
    }
}

fn millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}

fn main() -> anyhow::Result<()> {
    let conn = Connection::connect_to_env()?;
    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let registry = display.get_registry(&qh, ());

    event_queue.roundtrip(&mut App)?;

    let seat = registry.bind::<WlSeat, _, _>(14, 8, &qh, ());
    let keyboard_manager = registry.bind::<ZwpVirtualKeyboardManagerV1, _, _>(27, 1, &qh, ());
    let keyboard = keyboard_manager.create_virtual_keyboard(&seat, &qh, ());

    let src = keymap::KEYMAP;
    let size = keymap::KEYMAP.len();
    let mut file = tempfile::tempfile()?;
    file.seek(SeekFrom::Start(size as u64))?;
    file.write_all(&[0])?;
    file.seek(SeekFrom::Start(0))?;
    let mut data = unsafe { memmap2::MmapOptions::new().map_mut(&file)? };
    data[..src.len()].copy_from_slice(src.as_bytes());
    let fd = file.into_raw_fd();

    keyboard.keymap(1, fd, size as u32);

    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        keyboard.key(millis() as u32, input_event_codes::KEY_W!(), 1);
        std::thread::sleep(std::time::Duration::from_millis(1000));
        keyboard.key(millis() as u32, input_event_codes::KEY_W!(), 0);
        event_queue.roundtrip(&mut App)?;
    }

    Ok(())
}
