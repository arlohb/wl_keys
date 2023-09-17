use std::{
    collections::HashMap,
    os::fd::{AsFd, OwnedFd},
};

use anyhow::{bail, Context, Result};
use wayland_client::{
    delegate_noop,
    protocol::{
        wl_display::WlDisplay,
        wl_keyboard::{self, WlKeyboard},
        wl_registry::{self, WlRegistry},
        wl_seat::WlSeat,
    },
    Connection, Dispatch, EventQueue, Proxy, QueueHandle,
};
use wayland_protocols_misc::{
    zwp_input_method_v2::client::{
        zwp_input_method_manager_v2::ZwpInputMethodManagerV2,
        zwp_input_method_v2::{self, ZwpInputMethodV2},
    },
    zwp_virtual_keyboard_v1::client::{
        zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
        zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
    },
};

/// This is taken from the real `WlKeyboard`,
/// and passed as the keymap for my virtual keyboard.
struct Keymap {
    fd: OwnedFd,
    size: u32,
}

#[derive(Default)]
struct State {
    globals: HashMap<String, (u32, u32)>,
    keymap: Option<Keymap>,
    // Whether it will automatically open and close
    auto: bool,
}

delegate_noop!(State: ignore WlSeat);
delegate_noop!(State: ignore ZwpVirtualKeyboardManagerV1);
delegate_noop!(State: ignore ZwpVirtualKeyboardV1);
delegate_noop!(State: ignore ZwpInputMethodManagerV2);

impl Dispatch<ZwpInputMethodV2, ()> for State {
    fn event(
        state: &mut Self,
        _zwp_input_method: &ZwpInputMethodV2,
        event: zwp_input_method_v2::Event,
        _user_state: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        use zwp_input_method_v2::Event;

        if !state.auto {
            return;
        }

        let _ = match event {
            Event::Activate => crate::ui::open(),
            Event::Deactivate => crate::ui::close(),
            _ => Ok(()),
        };
    }
}

impl Dispatch<WlKeyboard, ()> for State {
    fn event(
        state: &mut Self,
        _keyboard: &WlKeyboard,
        event: wl_keyboard::Event,
        _user_state: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Keymap { format, fd, size }
                if format == wayland_client::WEnum::Value(wl_keyboard::KeymapFormat::XkbV1) =>
            {
                state.keymap = Some(Keymap { fd, size });
            }
            _ => (),
        };
    }
}

impl Dispatch<WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        _registry: &WlRegistry,
        event: wl_registry::Event,
        _user_state: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
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
        qh: &QueueHandle<Self>,
    ) -> Result<T>
    where
        Self: Dispatch<T, ()>,
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

/// The virtual keyboard
pub struct Keyboard {
    state: State,

    _conn: Connection,
    _display: WlDisplay,
    event_queue: EventQueue<State>,
    _qh: QueueHandle<State>,
    _registry: WlRegistry,

    _seat: WlSeat,
    _keyboard_manager: ZwpVirtualKeyboardManagerV1,
    keyboard: ZwpVirtualKeyboardV1,
}

impl Keyboard {
    /// Creates the virtual keyboard
    pub fn new() -> Result<Self> {
        let mut state = State::default();

        let conn = Connection::connect_to_env()?;
        let display = conn.display();
        let mut event_queue = conn.new_event_queue();
        let qh = event_queue.handle();

        // Get the globals from the registry
        let registry = display.get_registry(&qh, ());
        event_queue.roundtrip(&mut state)?;

        let seat = state.bind_global::<WlSeat>(&registry, &qh)?;
        // Take the keyboard
        seat.get_keyboard(&qh, ());
        event_queue.roundtrip(&mut state)?;

        // zwp_input_method_v2 is used for clients to become their own input method,
        // that manages text instead of just keypresses like the virtual keyboard.
        // I'm not using it to input the text, only for when to show and hide the keyboard.
        // This article was a great explainer for this
        // https://dorotac.eu/posts/input_method/
        let input_method_manager = state.bind_global::<ZwpInputMethodManagerV2>(&registry, &qh)?;
        input_method_manager.get_input_method(&seat, &qh, ());

        // Create the virtual keyboard
        let keyboard_manager = state.bind_global::<ZwpVirtualKeyboardManagerV1>(&registry, &qh)?;
        let keyboard = keyboard_manager.create_virtual_keyboard(&seat, &qh, ());

        // Set the keymap for the virtual keyboard
        if let Some(Keymap { fd, size }) = state.keymap.take() {
            keyboard.keymap(1, fd.as_fd(), size);
        } else {
            bail!("Keymap not found");
        }

        event_queue.roundtrip(&mut state)?;

        Ok(Self {
            state,

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

    fn time() -> u32 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_millis(0))
            .as_millis() as u32
    }

    /// Set the state of a key
    pub fn key(&self, key: u32, pressed: bool) -> Result<()> {
        self.keyboard.key(Self::time(), key, pressed.into());
        self.event_queue.flush()?;
        Ok(())
    }

    /// Blocks until all events are sent and processed
    pub fn roundtrip(&mut self) -> Result<()> {
        self.event_queue.roundtrip(&mut self.state)?;
        Ok(())
    }

    /// Enable input detection
    pub fn auto_enable(&mut self) {
        self.state.auto = true;
    }

    /// Disable input detection
    pub fn auto_disable(&mut self) {
        self.state.auto = false;
    }

    /// Toggle input detection
    pub fn auto_toggle(&mut self) {
        self.state.auto = !self.state.auto;
    }

    /// Get the auto status
    #[must_use]
    pub const fn auto_query(&self) -> bool {
        self.state.auto
    }
}
