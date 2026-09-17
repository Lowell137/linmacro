use crate::model::MouseButton;
use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, EventType, InputEvent, KeyCode, RelativeAxisCode};
use std::io;
use std::thread;
use std::time::Duration;

pub struct VirtualInput {
    mouse: VirtualDevice,
    keyboard: VirtualDevice,
}

impl VirtualInput {
    pub fn new() -> io::Result<Self> {
        // 1. Dedicated Virtual Mouse (Pure pointer device for libinput / mutter)
        let mut mouse_keys = AttributeSet::<KeyCode>::new();
        mouse_keys.insert(KeyCode::BTN_LEFT);
        mouse_keys.insert(KeyCode::BTN_RIGHT);
        mouse_keys.insert(KeyCode::BTN_MIDDLE);
        mouse_keys.insert(KeyCode::BTN_SIDE);
        mouse_keys.insert(KeyCode::BTN_EXTRA);

        let mut rels = AttributeSet::<RelativeAxisCode>::new();
        rels.insert(RelativeAxisCode::REL_X);
        rels.insert(RelativeAxisCode::REL_Y);
        rels.insert(RelativeAxisCode::REL_Z);
        rels.insert(RelativeAxisCode::REL_WHEEL);
        rels.insert(RelativeAxisCode::REL_HWHEEL);

        let mouse = VirtualDevice::builder()?
            .name("LinMacro Virtual Mouse")
            .with_keys(&mouse_keys)?
            .with_relative_axes(&rels)?
            .build()?;

        // 2. Dedicated Virtual Keyboard
        let mut kbd_keys = AttributeSet::<KeyCode>::new();
        for code in 1..=255 {
            kbd_keys.insert(KeyCode(code));
        }

        let keyboard = VirtualDevice::builder()?
            .name("LinMacro Virtual Keyboard")
            .with_keys(&kbd_keys)?
            .build()?;

        // Give the kernel / display server time to attach
        thread::sleep(Duration::from_millis(100));

        Ok(Self { mouse, keyboard })
    }

    pub fn send_mouse_button(&mut self, button: MouseButton, is_down: bool) -> io::Result<()> {
        let key = button.to_keycode();
        let val = if is_down { 1 } else { 0 };
        self.mouse.emit(&[
            InputEvent::new(EventType::KEY.0, key.0, val),
            InputEvent::new(EventType::SYNCHRONIZATION.0, 0, 0),
        ])
    }

    pub fn mouse_click(&mut self, button: MouseButton, hold_ms: u64) -> io::Result<()> {
        self.send_mouse_button(button, true)?;
        if hold_ms > 0 {
            thread::sleep(Duration::from_millis(hold_ms));
        }
        self.send_mouse_button(button, false)
    }

    pub fn send_key(&mut self, key: KeyCode, is_down: bool) -> io::Result<()> {
        let val = if is_down { 1 } else { 0 };
        self.keyboard.emit(&[
            InputEvent::new(EventType::KEY.0, key.0, val),
            InputEvent::new(EventType::SYNCHRONIZATION.0, 0, 0),
        ])
    }
}
