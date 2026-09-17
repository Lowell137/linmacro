use eframe::egui;
use linmacro::model::{
    egui_key_to_keycode, ClickMode, ClickerConfig, KeyCode, MouseButton,
};
use linmacro::virtual_device::VirtualInput;

#[test]
fn test_key_mapping() {
    assert_eq!(egui_key_to_keycode(egui::Key::A), Some(KeyCode::KEY_A));
    assert_eq!(egui_key_to_keycode(egui::Key::F8), Some(KeyCode::KEY_F8));
    assert_eq!(egui_key_to_keycode(egui::Key::Space), Some(KeyCode::KEY_SPACE));
}

#[test]
fn test_config_save_load() {
    let mut cfg = ClickerConfig::default();
    cfg.cps = 35;
    cfg.button = MouseButton::Right;
    cfg.mode = ClickMode::Hold;
    cfg.hotkey = Some(KeyCode::KEY_F9);

    let json = serde_json::to_string_pretty(&cfg).expect("Serialization failed");
    let deserialized: ClickerConfig = serde_json::from_str(&json).expect("Deserialization failed");

    assert_eq!(deserialized.cps, 35);
    assert_eq!(deserialized.button, MouseButton::Right);
    assert_eq!(deserialized.mode, ClickMode::Hold);
    assert_eq!(deserialized.hotkey, Some(KeyCode::KEY_F9));
}

#[test]
fn test_virtual_devices_init() {
    let mut vi = VirtualInput::new().expect("VirtualInput init failed");
    vi.mouse_click(MouseButton::Left, 5).expect("Click failed");
}
