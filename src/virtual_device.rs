use crate::model::MouseButton;
use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, EventType, InputEvent, KeyCode, RelativeAxisCode};
use std::io;
use std::thread;
use std::time::Duration;

pub struct VirtualInput {
    device: VirtualDevice,
}

impl VirtualInput {
    pub fn new() -> io::Result<Self> {
        let mut keys = AttributeSet::<KeyCode>::new();
        // Register all keyboard keys and mouse buttons (codes 0 to 560)
        for code in 0..=560 {
            keys.insert(KeyCode(code));
        }

        let mut rels = AttributeSet::<RelativeAxisCode>::new();
        rels.insert(RelativeAxisCode::REL_X);
        rels.insert(RelativeAxisCode::REL_Y);
        rels.insert(RelativeAxisCode::REL_WHEEL);
        rels.insert(RelativeAxisCode::REL_HWHEEL);

        let device = VirtualDevice::builder()?
            .name("LinMacro Virtual Controller")
            .with_keys(&keys)?
            .with_relative_axes(&rels)?
            .build()?;

        // Give the kernel / display server 50ms to attach the new device node
        thread::sleep(Duration::from_millis(50));

        Ok(Self { device })
    }

    pub fn send_key(&mut self, key: KeyCode, is_down: bool) -> io::Result<()> {
        let val = if is_down { 1 } else { 0 };
        self.device.emit(&[
            InputEvent::new(EventType::KEY.0, key.0, val),
            InputEvent::new(EventType::SYNCHRONIZATION.0, 0, 0),
        ])
    }

    pub fn key_press(&mut self, key: KeyCode, hold_ms: u64) -> io::Result<()> {
        self.send_key(key, true)?;
        if hold_ms > 0 {
            thread::sleep(Duration::from_millis(hold_ms));
        }
        self.send_key(key, false)
    }

    pub fn send_mouse_button(&mut self, button: MouseButton, is_down: bool) -> io::Result<()> {
        let key = button.to_keycode();
        self.send_key(key, is_down)
    }

    pub fn mouse_click(&mut self, button: MouseButton, hold_ms: u64) -> io::Result<()> {
        self.send_mouse_button(button, true)?;
        if hold_ms > 0 {
            thread::sleep(Duration::from_millis(hold_ms));
        }
        self.send_mouse_button(button, false)
    }

    pub fn mouse_move(&mut self, dx: i32, dy: i32) -> io::Result<()> {
        let mut events = Vec::with_capacity(3);
        if dx != 0 {
            events.push(InputEvent::new(
                EventType::RELATIVE.0,
                RelativeAxisCode::REL_X.0,
                dx,
            ));
        }
        if dy != 0 {
            events.push(InputEvent::new(
                EventType::RELATIVE.0,
                RelativeAxisCode::REL_Y.0,
                dy,
            ));
        }
        events.push(InputEvent::new(EventType::SYNCHRONIZATION.0, 0, 0));
        self.device.emit(&events)
    }

    pub fn mouse_wheel(&mut self, delta: i32) -> io::Result<()> {
        self.device.emit(&[
            InputEvent::new(
                EventType::RELATIVE.0,
                RelativeAxisCode::REL_WHEEL.0,
                delta,
            ),
            InputEvent::new(EventType::SYNCHRONIZATION.0, 0, 0),
        ])
    }

    pub fn type_text(&mut self, text: &str, char_delay_ms: u64) -> io::Result<()> {
        for ch in text.chars() {
            if let Some((key, needs_shift)) = char_to_keycode(ch) {
                if needs_shift {
                    self.send_key(KeyCode::KEY_LEFTSHIFT, true)?;
                }
                self.key_press(key, 10)?;
                if needs_shift {
                    self.send_key(KeyCode::KEY_LEFTSHIFT, false)?;
                }
                if char_delay_ms > 0 {
                    thread::sleep(Duration::from_millis(char_delay_ms));
                }
            }
        }
        Ok(())
    }
}

fn char_to_keycode(ch: char) -> Option<(KeyCode, bool)> {
    match ch {
        'a' => Some((KeyCode::KEY_A, false)),
        'A' => Some((KeyCode::KEY_A, true)),
        'b' => Some((KeyCode::KEY_B, false)),
        'B' => Some((KeyCode::KEY_B, true)),
        'c' => Some((KeyCode::KEY_C, false)),
        'C' => Some((KeyCode::KEY_C, true)),
        'd' => Some((KeyCode::KEY_D, false)),
        'D' => Some((KeyCode::KEY_D, true)),
        'e' => Some((KeyCode::KEY_E, false)),
        'E' => Some((KeyCode::KEY_E, true)),
        'f' => Some((KeyCode::KEY_F, false)),
        'F' => Some((KeyCode::KEY_F, true)),
        'g' => Some((KeyCode::KEY_G, false)),
        'G' => Some((KeyCode::KEY_G, true)),
        'h' => Some((KeyCode::KEY_H, false)),
        'H' => Some((KeyCode::KEY_H, true)),
        'i' => Some((KeyCode::KEY_I, false)),
        'I' => Some((KeyCode::KEY_I, true)),
        'j' => Some((KeyCode::KEY_J, false)),
        'J' => Some((KeyCode::KEY_J, true)),
        'k' => Some((KeyCode::KEY_K, false)),
        'K' => Some((KeyCode::KEY_K, true)),
        'l' => Some((KeyCode::KEY_L, false)),
        'L' => Some((KeyCode::KEY_L, true)),
        'm' => Some((KeyCode::KEY_M, false)),
        'M' => Some((KeyCode::KEY_M, true)),
        'n' => Some((KeyCode::KEY_N, false)),
        'N' => Some((KeyCode::KEY_N, true)),
        'o' => Some((KeyCode::KEY_O, false)),
        'O' => Some((KeyCode::KEY_O, true)),
        'p' => Some((KeyCode::KEY_P, false)),
        'P' => Some((KeyCode::KEY_P, true)),
        'q' => Some((KeyCode::KEY_Q, false)),
        'Q' => Some((KeyCode::KEY_Q, true)),
        'r' => Some((KeyCode::KEY_R, false)),
        'R' => Some((KeyCode::KEY_R, true)),
        's' => Some((KeyCode::KEY_S, false)),
        'S' => Some((KeyCode::KEY_S, true)),
        't' => Some((KeyCode::KEY_T, false)),
        'T' => Some((KeyCode::KEY_T, true)),
        'u' => Some((KeyCode::KEY_U, false)),
        'U' => Some((KeyCode::KEY_U, true)),
        'v' => Some((KeyCode::KEY_V, false)),
        'V' => Some((KeyCode::KEY_V, true)),
        'w' => Some((KeyCode::KEY_W, false)),
        'W' => Some((KeyCode::KEY_W, true)),
        'x' => Some((KeyCode::KEY_X, false)),
        'X' => Some((KeyCode::KEY_X, true)),
        'y' => Some((KeyCode::KEY_Y, false)),
        'Y' => Some((KeyCode::KEY_Y, true)),
        'z' => Some((KeyCode::KEY_Z, false)),
        'Z' => Some((KeyCode::KEY_Z, true)),
        '1' => Some((KeyCode::KEY_1, false)),
        '!' => Some((KeyCode::KEY_1, true)),
        '2' => Some((KeyCode::KEY_2, false)),
        '@' => Some((KeyCode::KEY_2, true)),
        '3' => Some((KeyCode::KEY_3, false)),
        '#' => Some((KeyCode::KEY_3, true)),
        '4' => Some((KeyCode::KEY_4, false)),
        '$' => Some((KeyCode::KEY_4, true)),
        '5' => Some((KeyCode::KEY_5, false)),
        '%' => Some((KeyCode::KEY_5, true)),
        '6' => Some((KeyCode::KEY_6, false)),
        '^' => Some((KeyCode::KEY_6, true)),
        '7' => Some((KeyCode::KEY_7, false)),
        '&' => Some((KeyCode::KEY_7, true)),
        '8' => Some((KeyCode::KEY_8, false)),
        '*' => Some((KeyCode::KEY_8, true)),
        '9' => Some((KeyCode::KEY_9, false)),
        '(' => Some((KeyCode::KEY_9, true)),
        '0' => Some((KeyCode::KEY_0, false)),
        ')' => Some((KeyCode::KEY_0, true)),
        ' ' => Some((KeyCode::KEY_SPACE, false)),
        '\n' => Some((KeyCode::KEY_ENTER, false)),
        '\t' => Some((KeyCode::KEY_TAB, false)),
        '-' => Some((KeyCode::KEY_MINUS, false)),
        '_' => Some((KeyCode::KEY_MINUS, true)),
        '=' => Some((KeyCode::KEY_EQUAL, false)),
        '+' => Some((KeyCode::KEY_EQUAL, true)),
        '.' => Some((KeyCode::KEY_DOT, false)),
        ',' => Some((KeyCode::KEY_COMMA, false)),
        '/' => Some((KeyCode::KEY_SLASH, false)),
        '?' => Some((KeyCode::KEY_SLASH, true)),
        ';' => Some((KeyCode::KEY_SEMICOLON, false)),
        ':' => Some((KeyCode::KEY_SEMICOLON, true)),
        '\'' => Some((KeyCode::KEY_APOSTROPHE, false)),
        '"' => Some((KeyCode::KEY_APOSTROPHE, true)),
        _ => None,
    }
}
