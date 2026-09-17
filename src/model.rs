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
    Side,
    Extra,
}

impl MouseButton {
    pub fn to_keycode(self) -> KeyCode {
        match self {
            MouseButton::Left => KeyCode::BTN_LEFT,
            MouseButton::Right => KeyCode::BTN_RIGHT,
            MouseButton::Middle => KeyCode::BTN_MIDDLE,
            MouseButton::Side => KeyCode::BTN_SIDE,
            MouseButton::Extra => KeyCode::BTN_EXTRA,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            MouseButton::Left => "Left Click",
            MouseButton::Right => "Right Click",
            MouseButton::Middle => "Middle Click",
            MouseButton::Side => "Side Button 1 (Back)",
            MouseButton::Extra => "Side Button 2 (Forward)",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    KeyDown(KeyCode),
    KeyUp(KeyCode),
    KeyPress { key: KeyCode, hold_ms: u64 },
    MouseDown(MouseButton),
    MouseUp(MouseButton),
    MouseClick { button: MouseButton, hold_ms: u64 },
    MouseMove { dx: i32, dy: i32 },
    MouseWheel { delta: i32 },
    Delay(u64),
    TypeText(String),
}

impl Action {
    pub fn summary(&self) -> String {
        match self {
            Action::KeyDown(k) => format!("Key Down: {}", format_key_name(*k)),
            Action::KeyUp(k) => format!("Key Up: {}", format_key_name(*k)),
            Action::KeyPress { key, hold_ms } => {
                format!("Key Press: {} ({} ms)", format_key_name(*key), hold_ms)
            }
            Action::MouseDown(b) => format!("Mouse Down: {}", b.name()),
            Action::MouseUp(b) => format!("Mouse Up: {}", b.name()),
            Action::MouseClick { button, hold_ms } => {
                format!("Mouse Click: {} ({} ms)", button.name(), hold_ms)
            }
            Action::MouseMove { dx, dy } => format!("Mouse Move: dx={}, dy={}", dx, dy),
            Action::MouseWheel { delta } => {
                if *delta > 0 {
                    format!("Mouse Wheel: Up ({})", delta)
                } else {
                    format!("Mouse Wheel: Down ({})", delta.abs())
                }
            }
            Action::Delay(ms) => format!("Delay: {} ms", ms),
            Action::TypeText(text) => format!("Type Text: \"{}\"", text),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerMode {
    Once,
    ToggleLoop,
    HoldLoop,
}

impl TriggerMode {
    pub fn name(self) -> &'static str {
        match self {
            TriggerMode::Once => "Run Once",
            TriggerMode::ToggleLoop => "Toggle Loop (Start/Stop)",
            TriggerMode::HoldLoop => "Hold Loop (While Pressed)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub trigger_key: Option<KeyCode>,
    pub trigger_mode: TriggerMode,
    /// 0 = infinite loop (in ToggleLoop/HoldLoop), N = repeat N times
    pub repeat_count: u32,
    pub actions: Vec<Action>,
}

impl Macro {
    pub fn new(name: impl Into<String>) -> Self {
        let id = format!(
            "m_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        );
        Self {
            id,
            name: name.into(),
            enabled: true,
            trigger_key: None,
            trigger_mode: TriggerMode::Once,
            repeat_count: 1,
            actions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub global_enabled: bool,
    pub killswitch_key: KeyCode,
    pub macros: Vec<Macro>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            global_enabled: true,
            killswitch_key: KeyCode::KEY_PAUSE,
            macros: vec![
                Macro {
                    id: "sample_autoclicker".into(),
                    name: "Rapid Auto Clicker (Left Click)".into(),
                    enabled: true,
                    trigger_key: Some(KeyCode::KEY_F8),
                    trigger_mode: TriggerMode::ToggleLoop,
                    repeat_count: 0,
                    actions: vec![
                        Action::MouseClick {
                            button: MouseButton::Left,
                            hold_ms: 15,
                        },
                        Action::Delay(40),
                    ],
                },
                Macro {
                    id: "sample_combo".into(),
                    name: "Sample Combo (Q + E)".into(),
                    enabled: true,
                    trigger_key: Some(KeyCode::KEY_F7),
                    trigger_mode: TriggerMode::Once,
                    repeat_count: 1,
                    actions: vec![
                        Action::KeyPress {
                            key: KeyCode::KEY_Q,
                            hold_ms: 30,
                        },
                        Action::Delay(50),
                        Action::KeyPress {
                            key: KeyCode::KEY_E,
                            hold_ms: 30,
                        },
                    ],
                },
                Macro {
                    id: "sample_text".into(),
                    name: "Text Macro (Hello World)".into(),
                    enabled: true,
                    trigger_key: Some(KeyCode::KEY_F6),
                    trigger_mode: TriggerMode::Once,
                    repeat_count: 1,
                    actions: vec![
                        Action::TypeText("Hello from LinMacro!\n".into()),
                    ],
                },
            ],
        }
    }
}

impl AppConfig {
    fn config_path() -> PathBuf {
        let base = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg)
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config")
        } else {
            PathBuf::from(".")
        };
        base.join("linmacro").join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&data) {
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
        "LEFTMETA" => "Super / Windows (Left)".into(),
        "RIGHTMETA" => "Super / Windows (Right)".into(),
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
        other => {
            if let Some(btn) = other.strip_prefix("BTN_") {
                match btn {
                    "LEFT" => "Mouse Left Click".into(),
                    "RIGHT" => "Mouse Right Click".into(),
                    "MIDDLE" => "Mouse Middle Click".into(),
                    "SIDE" => "Mouse Side Button 1".into(),
                    "EXTRA" => "Mouse Side Button 2".into(),
                    _ => format!("Mouse {}", btn),
                }
            } else {
                other.to_string()
            }
        }
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

        egui::Key::Minus => Some(KeyCode::KEY_MINUS),
        egui::Key::Equals => Some(KeyCode::KEY_EQUAL),
        egui::Key::Semicolon => Some(KeyCode::KEY_SEMICOLON),
        egui::Key::Comma => Some(KeyCode::KEY_COMMA),
        egui::Key::Period => Some(KeyCode::KEY_DOT),
        egui::Key::Slash => Some(KeyCode::KEY_SLASH),
        egui::Key::Backtick => Some(KeyCode::KEY_GRAVE),
        _ => None,
    }
}
