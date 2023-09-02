use std::{
    collections::HashMap,
    io::{Seek, SeekFrom, Write},
    os::fd::IntoRawFd,
};

use anyhow::{bail, Context};
use wayland_client::{
    delegate_noop,
    protocol::{
        wl_display::WlDisplay,
        wl_registry::{self, WlRegistry},
        wl_seat::WlSeat,
    },
    Connection, Dispatch, EventQueue, Proxy, QueueHandle,
};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
    zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
};

mod keymap;

#[derive(Default)]
struct State {
    globals: HashMap<String, (u32, u32)>,
}

delegate_noop!(State: ignore WlSeat);
delegate_noop!(State: ignore ZwpVirtualKeyboardManagerV1);
delegate_noop!(State: ignore ZwpVirtualKeyboardV1);

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        _registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _user_state: &(),
        _conn: &Connection,
        _qh: &QueueHandle<State>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            state.globals.insert(interface, (name, version));
        }
    }
}

impl State {
    pub fn bind_global<T: Proxy + 'static>(
        &self,
        registry: &wl_registry::WlRegistry,
        qh: &QueueHandle<State>,
    ) -> anyhow::Result<T>
    where
        State: Dispatch<T, ()>,
    {
        let interface = T::interface();
        let &(id, version) = self
            .globals
            .get(interface.name)
            .context(format!("{interface} not found"))?;

        if interface.version < version {
            bail!(
                "{} v{version} exceeds the max supported version (v{})",
                interface.name,
                interface.version
            );
        }

        Ok(registry.bind::<T, _, _>(id, version, qh, ()))
    }
}

struct App {
    _state: State,

    _conn: Connection,
    _display: WlDisplay,
    event_queue: EventQueue<State>,
    _qh: QueueHandle<State>,
    _registry: WlRegistry,

    _seat: WlSeat,
    _keyboard_manager: ZwpVirtualKeyboardManagerV1,
    keyboard: ZwpVirtualKeyboardV1,
}

impl App {
    pub fn new() -> anyhow::Result<App> {
        let conn = Connection::connect_to_env()?;
        let display = conn.display();
        let mut event_queue = conn.new_event_queue();
        let qh = event_queue.handle();
        let registry = display.get_registry(&qh, ());

        let mut state = State::default();

        event_queue.roundtrip(&mut state)?;

        let seat = state.bind_global::<WlSeat>(&registry, &qh)?;
        let keyboard_manager = state.bind_global::<ZwpVirtualKeyboardManagerV1>(&registry, &qh)?;
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
        event_queue.flush()?;

        Ok(App {
            _state: state,

            _conn: conn,
            _display: display,
            event_queue,
            _qh: qh,
            _registry: registry,

            _seat: seat,
            _keyboard_manager: keyboard_manager,
            keyboard,
        })
    }

    fn time(&self) -> u32 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u32
    }

    fn key(&self, key: u32, pressed: bool) -> anyhow::Result<()> {
        self.keyboard
            .key(self.time(), key, if pressed { 1 } else { 0 });
        self.event_queue.flush()?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let app = App::new()?;

    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        app.key(input_event_codes::KEY_W!(), true)?;

        std::thread::sleep(std::time::Duration::from_millis(10));
        app.key(input_event_codes::KEY_W!(), false)?;
    }

    Ok(())
}
