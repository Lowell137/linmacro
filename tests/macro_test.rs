use eframe::egui;
use linmacro::model::{
    egui_key_to_keycode, Action, AppConfig, KeyCode, Macro, MouseButton, TriggerMode,
};
use linmacro::virtual_device::VirtualInput;

#[test]
fn test_key_mapping() {
    assert_eq!(egui_key_to_keycode(egui::Key::A), Some(KeyCode::KEY_A));
    assert_eq!(egui_key_to_keycode(egui::Key::F8), Some(KeyCode::KEY_F8));
    assert_eq!(egui_key_to_keycode(egui::Key::Space), Some(KeyCode::KEY_SPACE));
    assert_eq!(egui_key_to_keycode(egui::Key::Enter), Some(KeyCode::KEY_ENTER));
    assert_eq!(egui_key_to_keycode(egui::Key::Escape), Some(KeyCode::KEY_ESC));
}

#[test]
fn test_macro_serialization() {
    let mut m = Macro::new("Test Macro");
    m.trigger_key = Some(KeyCode::KEY_F8);
    m.trigger_mode = TriggerMode::ToggleLoop;
    m.actions.push(Action::MouseClick {
        button: MouseButton::Left,
        hold_ms: 20,
    });
    m.actions.push(Action::Delay(50));

    let json = serde_json::to_string_pretty(&m).expect("Serialization failed");
    let deserialized: Macro = serde_json::from_str(&json).expect("Deserialization failed");

    assert_eq!(deserialized.name, "Test Macro");
    assert_eq!(deserialized.trigger_key, Some(KeyCode::KEY_F8));
    assert_eq!(deserialized.trigger_mode, TriggerMode::ToggleLoop);
    assert_eq!(deserialized.actions.len(), 2);
}

#[test]
fn test_config_defaults() {
    let config = AppConfig::default();
    assert!(config.global_enabled);
    assert_eq!(config.killswitch_key, KeyCode::KEY_PAUSE);
    assert!(!config.macros.is_empty());
}

#[test]
fn test_virtual_input_singleton() {
    // Check that VirtualInput can be initialized without error
    let mut vi = VirtualInput::new().expect("Failed to initialize VirtualInput");
    vi.send_key(KeyCode::KEY_A, false).expect("Failed to send key");
}
