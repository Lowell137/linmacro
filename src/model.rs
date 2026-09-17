use eframe::egui;
pub use evdev::KeyCode;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    pub fn to_keycode(self) -> KeyCode {
        match self {
            MouseButton::Left => KeyCode::BTN_LEFT,
            MouseButton::Right => KeyCode::BTN_RIGHT,
            MouseButton::Middle => KeyCode::BTN_MIDDLE,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            MouseButton::Left => "Left Click",
            MouseButton::Right => "Right Click",
            MouseButton::Middle => "Middle Click",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClickType {
    Single,
    Double,
}

impl ClickType {
    pub fn name(self) -> &'static str {
        match self {
            ClickType::Single => "Single",
            ClickType::Double => "Double",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClickMode {
    Toggle, // Press key once to start, press again to stop
    Hold,   // Click while key is held down
}

impl ClickMode {
    pub fn name(self) -> &'static str {
        match self {
            ClickMode::Toggle => "Toggle (Press to Start/Stop)",
            ClickMode::Hold => "Hold (Click while holding key)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickerConfig {
    pub cps: u32,
    pub button: MouseButton,
    pub click_type: ClickType,
    pub mode: ClickMode,
    pub hotkey: Option<KeyCode>,
    pub killswitch: KeyCode,
}

impl Default for ClickerConfig {
    fn default() -> Self {
        Self {
            cps: 20,
            button: MouseButton::Left,
            click_type: ClickType::Single,
            mode: ClickMode::Toggle,
            hotkey: Some(KeyCode::KEY_F8),
            killswitch: KeyCode::KEY_PAUSE,
        }
    }
}

impl ClickerConfig {
    fn config_path() -> PathBuf {
        let base = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg)
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config")
        } else {
            PathBuf::from(".")
        };
        base.join("linmacro").join("clicker.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<ClickerConfig>(&data) {
                return cfg;
            }
        }
        let default_cfg = Self::default();
        let _ = default_cfg.save();
        default_cfg
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub fn format_key_name(key: KeyCode) -> String {
    let raw = format!("{:?}", key);
    let s = raw.strip_prefix("KEY_").unwrap_or(&raw);
    match s {
        "ESC" => "Escape".into(),
        "BACKSPACE" => "Backspace".into(),
        "ENTER" => "Enter".into(),
        "SPACE" => "Space".into(),
        "TAB" => "Tab".into(),
        "LEFTSHIFT" => "Left Shift".into(),
        "RIGHTSHIFT" => "Right Shift".into(),
        "LEFTCTRL" => "Left Ctrl".into(),
        "RIGHTCTRL" => "Right Ctrl".into(),
        "LEFTALT" => "Left Alt".into(),
        "RIGHTALT" => "Right Alt".into(),
        "LEFTMETA" => "Super / Win (Left)".into(),
        "RIGHTMETA" => "Super / Win (Right)".into(),
        "CAPSLOCK" => "Caps Lock".into(),
        "PAUSE" => "Pause/Break".into(),
        "UP" => "Up Arrow".into(),
        "DOWN" => "Down Arrow".into(),
        "LEFT" => "Left Arrow".into(),
        "RIGHT" => "Right Arrow".into(),
        "DELETE" => "Delete".into(),
        "INSERT" => "Insert".into(),
        "HOME" => "Home".into(),
        "END" => "End".into(),
        "PAGEUP" => "Page Up".into(),
        "PAGEDOWN" => "Page Down".into(),
        other => other.to_string(),
    }
}

pub fn egui_key_to_keycode(key: egui::Key) -> Option<KeyCode> {
    match key {
        egui::Key::A => Some(KeyCode::KEY_A),
        egui::Key::B => Some(KeyCode::KEY_B),
        egui::Key::C => Some(KeyCode::KEY_C),
        egui::Key::D => Some(KeyCode::KEY_D),
        egui::Key::E => Some(KeyCode::KEY_E),
        egui::Key::F => Some(KeyCode::KEY_F),
        egui::Key::G => Some(KeyCode::KEY_G),
        egui::Key::H => Some(KeyCode::KEY_H),
        egui::Key::I => Some(KeyCode::KEY_I),
        egui::Key::J => Some(KeyCode::KEY_J),
        egui::Key::K => Some(KeyCode::KEY_K),
        egui::Key::L => Some(KeyCode::KEY_L),
        egui::Key::M => Some(KeyCode::KEY_M),
        egui::Key::N => Some(KeyCode::KEY_N),
        egui::Key::O => Some(KeyCode::KEY_O),
        egui::Key::P => Some(KeyCode::KEY_P),
        egui::Key::Q => Some(KeyCode::KEY_Q),
        egui::Key::R => Some(KeyCode::KEY_R),
        egui::Key::S => Some(KeyCode::KEY_S),
        egui::Key::T => Some(KeyCode::KEY_T),
        egui::Key::U => Some(KeyCode::KEY_U),
        egui::Key::V => Some(KeyCode::KEY_V),
        egui::Key::W => Some(KeyCode::KEY_W),
        egui::Key::X => Some(KeyCode::KEY_X),
        egui::Key::Y => Some(KeyCode::KEY_Y),
        egui::Key::Z => Some(KeyCode::KEY_Z),

        egui::Key::Num0 => Some(KeyCode::KEY_0),
        egui::Key::Num1 => Some(KeyCode::KEY_1),
        egui::Key::Num2 => Some(KeyCode::KEY_2),
        egui::Key::Num3 => Some(KeyCode::KEY_3),
        egui::Key::Num4 => Some(KeyCode::KEY_4),
        egui::Key::Num5 => Some(KeyCode::KEY_5),
        egui::Key::Num6 => Some(KeyCode::KEY_6),
        egui::Key::Num7 => Some(KeyCode::KEY_7),
        egui::Key::Num8 => Some(KeyCode::KEY_8),
        egui::Key::Num9 => Some(KeyCode::KEY_9),

        egui::Key::F1 => Some(KeyCode::KEY_F1),
        egui::Key::F2 => Some(KeyCode::KEY_F2),
        egui::Key::F3 => Some(KeyCode::KEY_F3),
        egui::Key::F4 => Some(KeyCode::KEY_F4),
        egui::Key::F5 => Some(KeyCode::KEY_F5),
        egui::Key::F6 => Some(KeyCode::KEY_F6),
        egui::Key::F7 => Some(KeyCode::KEY_F7),
        egui::Key::F8 => Some(KeyCode::KEY_F8),
        egui::Key::F9 => Some(KeyCode::KEY_F9),
        egui::Key::F10 => Some(KeyCode::KEY_F10),
        egui::Key::F11 => Some(KeyCode::KEY_F11),
        egui::Key::F12 => Some(KeyCode::KEY_F12),

        egui::Key::Escape => Some(KeyCode::KEY_ESC),
        egui::Key::Space => Some(KeyCode::KEY_SPACE),
        egui::Key::Enter => Some(KeyCode::KEY_ENTER),
        egui::Key::Tab => Some(KeyCode::KEY_TAB),
        egui::Key::Backspace => Some(KeyCode::KEY_BACKSPACE),
        egui::Key::Insert => Some(KeyCode::KEY_INSERT),
        egui::Key::Delete => Some(KeyCode::KEY_DELETE),
        egui::Key::Home => Some(KeyCode::KEY_HOME),
        egui::Key::End => Some(KeyCode::KEY_END),
        egui::Key::PageUp => Some(KeyCode::KEY_PAGEUP),
        egui::Key::PageDown => Some(KeyCode::KEY_PAGEDOWN),
        egui::Key::ArrowUp => Some(KeyCode::KEY_UP),
        egui::Key::ArrowDown => Some(KeyCode::KEY_DOWN),
        egui::Key::ArrowLeft => Some(KeyCode::KEY_LEFT),
        egui::Key::ArrowRight => Some(KeyCode::KEY_RIGHT),
        _ => None,
    }
}
