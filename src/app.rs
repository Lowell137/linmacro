use crate::engine::{EngineCommand, EngineEvent, EngineHandle};
use crate::model::{
    egui_key_to_keycode, format_key_name, Action, AppConfig, KeyCode, Macro, MouseButton,
    TriggerMode,
};
use eframe::egui::{self, Color32, RichText};
use std::collections::HashSet;

pub struct LinMacroApp {
    config: AppConfig,
    engine: EngineHandle,
    selected_macro_id: Option<String>,
    running_macros: HashSet<String>,
    is_recording: bool,
    is_assigning_trigger: bool,
    recorded_actions_count: usize,
    status_error: Option<String>,
    device_count: usize,
    virtual_device_ok: bool,

    // Action creator state
    selected_action_kind: ActionKind,
    action_key: KeyCode,
    action_hold_ms: u64,
    action_mouse_btn: MouseButton,
    action_delay_ms: u64,
    action_text: String,
    action_mouse_dx: i32,
    action_mouse_dy: i32,
    action_wheel_delta: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActionKind {
    KeyPress,
    KeyDown,
    KeyUp,
    MouseClick,
    MouseDown,
    MouseUp,
    MouseMove,
    MouseWheel,
    Delay,
    TypeText,
}

impl LinMacroApp {
    pub fn new() -> Self {
        let config = AppConfig::load();
        let engine = EngineHandle::new(config.clone());
        let first_id = config.macros.first().map(|m| m.id.clone());

        Self {
            config,
            engine,
            selected_macro_id: first_id,
            running_macros: HashSet::new(),
            is_recording: false,
            is_assigning_trigger: false,
            recorded_actions_count: 0,
            status_error: None,
            device_count: 0,
            virtual_device_ok: true,

            selected_action_kind: ActionKind::KeyPress,
            action_key: KeyCode::KEY_A,
            action_hold_ms: 30,
            action_mouse_btn: MouseButton::Left,
            action_delay_ms: 50,
            action_text: String::from("Hello World"),
            action_mouse_dx: 50,
            action_mouse_dy: 0,
            action_wheel_delta: 1,
        }
    }

    fn poll_engine_events(&mut self) {
        while let Ok(event) = self.engine.event_rx.try_recv() {
            match event {
                EngineEvent::RecordingAction(_) => {
                    self.recorded_actions_count += 1;
                }
                EngineEvent::RecordingFinished(actions) => {
                    self.is_recording = false;
                    if !actions.is_empty() {
                        let mut new_macro = Macro::new(format!(
                            "Recording #{}",
                            self.config.macros.len() + 1
                        ));
                        new_macro.actions = actions;
                        let id = new_macro.id.clone();
                        self.config.macros.push(new_macro);
                        self.selected_macro_id = Some(id);
                        self.save_and_sync();
                    }
                }
                EngineEvent::KeyDetected(key) => {
                    self.is_assigning_trigger = false;
                    if let Some(m) = self.get_selected_macro_mut() {
                        m.trigger_key = Some(key);
                        self.save_and_sync();
                    }
                }
                EngineEvent::MacroStarted(id) => {
                    self.running_macros.insert(id);
                }
                EngineEvent::MacroStopped(id) => {
                    self.running_macros.remove(&id);
                }
                EngineEvent::Status {
                    accessible_devices,
                    virtual_device_ok,
                    error,
                } => {
                    self.device_count = accessible_devices;
                    self.virtual_device_ok = virtual_device_ok;
                    self.status_error = error;
                }
            }
        }
    }

    fn save_and_sync(&mut self) {
        let _ = self.config.save();
        let _ = self
            .engine
            .cmd_tx
            .send(EngineCommand::UpdateConfig(self.config.clone()));
    }

    fn get_selected_macro_mut(&mut self) -> Option<&mut Macro> {
        let id = self.selected_macro_id.as_ref()?;
        self.config.macros.iter_mut().find(|m| &m.id == id)
    }

    fn get_selected_macro(&self) -> Option<&Macro> {
        let id = self.selected_macro_id.as_ref()?;
        self.config.macros.iter().find(|m| &m.id == id)
    }
}

impl eframe::App for LinMacroApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_engine_events();

        // Check window keyboard input when assigning trigger
        if self.is_assigning_trigger {
            let pressed_key = ui.input(|i| {
                for ev in &i.raw.events {
                    if let egui::Event::Key {
                        key, pressed: true, ..
                    } = ev
                    {
                        return Some(*key);
                    }
                }
                None
            });

            if let Some(ekey) = pressed_key {
                if let Some(kc) = egui_key_to_keycode(ekey) {
                    self.is_assigning_trigger = false;
                    let _ = self.engine.cmd_tx.send(EngineCommand::CancelKeyListen);
                    if let Some(m) = self.get_selected_macro_mut() {
                        m.trigger_key = Some(kc);
                        self.save_and_sync();
                    }
                }
            }
        }

        if self.is_recording || !self.running_macros.is_empty() || self.is_assigning_trigger {
            ui.ctx().request_repaint();
        }

        // Top Header Panel
        egui::Panel::top("header").show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("⚡ LinMacro").strong().color(Color32::from_rgb(100, 200, 255)));
                ui.label(RichText::new("v0.1.0").weak().small());

                ui.separator();

                // Global Activation Toggle
                if self.config.global_enabled {
                    if ui.button(RichText::new("🟢 System Active").strong().color(Color32::from_rgb(70, 220, 100))).clicked() {
                        self.config.global_enabled = false;
                        self.save_and_sync();
                    }
                } else {
                    if ui.button(RichText::new("🔴 System Paused").strong().color(Color32::from_rgb(240, 80, 80))).clicked() {
                        self.config.global_enabled = true;
                        self.save_and_sync();
                    }
                }

                ui.separator();

                // Recording Button
                if self.is_recording {
                    if ui
                        .button(
                            RichText::new(format!("⏹️ Finish Recording ({} steps)", self.recorded_actions_count))
                                .strong()
                                .color(Color32::from_rgb(255, 100, 100)),
                        )
                        .clicked()
                    {
                        let _ = self.engine.cmd_tx.send(EngineCommand::StopRecording);
                    }
                } else {
                    if ui
                        .button(RichText::new("🔴 Live Record").strong().color(Color32::from_rgb(255, 120, 120)))
                        .clicked()
                    {
                        self.is_recording = true;
                        self.recorded_actions_count = 0;
                        let _ = self.engine.cmd_tx.send(EngineCommand::StartRecording);
                    }
                }

                ui.separator();

                // Stop All button
                if ui.button(RichText::new("⏹️ Stop All").color(Color32::LIGHT_GRAY)).clicked() {
                    let _ = self.engine.cmd_tx.send(EngineCommand::StopAll);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let kill_name = format_key_name(self.config.killswitch_key);
                    ui.label(RichText::new(format!("Emergency Stop: [{}]", kill_name)).weak().small());

                    if self.virtual_device_ok {
                        ui.label(RichText::new(format!("🟢 {} Input Devices", self.device_count)).small().color(Color32::from_rgb(120, 200, 120)));
                    } else {
                        ui.label(RichText::new("⚠️ Virtual Controller Error").small().color(Color32::from_rgb(255, 100, 100)));
                    }
                });
            });
            ui.add_space(6.0);
        });

        // Bottom status panel
        egui::Panel::bottom("footer").show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if let Some(err) = &self.status_error {
                    ui.label(RichText::new(format!("⚠️ {}", err)).color(Color32::from_rgb(255, 120, 120)).small());
                } else {
                    ui.label(RichText::new("💡 Tips: Macros run in background across Wayland and X11.").weak().small());
                }
            });
            ui.add_space(4.0);
        });

        // Left Panel: Macro List
        egui::Panel::left("macro_list")
            .resizable(true)
            .default_size(260.0)
            .show(ui, |ui| {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Macros").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("+ New").clicked() {
                            let new_m = Macro::new(format!("Macro #{}", self.config.macros.len() + 1));
                            let id = new_m.id.clone();
                            self.config.macros.push(new_m);
                            self.selected_macro_id = Some(id);
                            self.save_and_sync();
                        }
                    });
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut delete_id = None;

                    for m in &mut self.config.macros {
                        let is_selected = self.selected_macro_id.as_deref() == Some(&m.id);
                        let is_running = self.running_macros.contains(&m.id);

                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                let mut enabled = m.enabled;
                                if ui.checkbox(&mut enabled, "").changed() {
                                    m.enabled = enabled;
                                }

                                let name_text = if is_selected {
                                    RichText::new(&m.name).strong().color(Color32::from_rgb(100, 200, 255))
                                } else {
                                    RichText::new(&m.name)
                                };

                                if ui.selectable_label(is_selected, name_text).clicked() {
                                    self.selected_macro_id = Some(m.id.clone());
                                }
                            });

                            ui.horizontal(|ui| {
                                let key_text = match m.trigger_key {
                                    Some(k) => format!("[{}]", format_key_name(k)),
                                    None => "[No Key]".to_string(),
                                };
                                ui.label(RichText::new(key_text).small().weak());

                                if is_running {
                                    ui.label(RichText::new("⚡ Running").small().color(Color32::from_rgb(255, 200, 80)));
                                }

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("🗑").on_hover_text("Delete Macro").clicked() {
                                        delete_id = Some(m.id.clone());
                                    }
                                });
                            });
                        });
                        ui.add_space(2.0);
                    }

                    if let Some(del_id) = delete_id {
                        self.config.macros.retain(|m| m.id != del_id);
                        if self.selected_macro_id.as_deref() == Some(&del_id) {
                            self.selected_macro_id = self.config.macros.first().map(|m| m.id.clone());
                        }
                        self.save_and_sync();
                    }
                });
            });

        // Central Panel: Selected Macro Editor
        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(m) = self.get_selected_macro() {
                let m_clone = m.clone();
                let macro_id = m_clone.id.clone();
                let is_running = self.running_macros.contains(&macro_id);

                ui.horizontal(|ui| {
                    ui.label("Name:");
                    let mut name = m_clone.name.clone();
                    if ui.text_edit_singleline(&mut name).changed() {
                        if let Some(target) = self.get_selected_macro_mut() {
                            target.name = name;
                            self.save_and_sync();
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_running {
                            if ui.button(RichText::new("⏹️ Stop").color(Color32::from_rgb(255, 100, 100))).clicked() {
                                let _ = self.engine.cmd_tx.send(EngineCommand::StopMacro(macro_id.clone()));
                            }
                        } else {
                            if ui.button(RichText::new("▶ Test / Run").color(Color32::from_rgb(100, 220, 120))).clicked() {
                                let _ = self.engine.cmd_tx.send(EngineCommand::RunMacro(m_clone.clone()));
                            }
                        }
                    });
                });

                ui.add_space(8.0);

                // Trigger Configuration Box
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Hotkey Trigger:").strong());

                        if self.is_assigning_trigger {
                            ui.label(RichText::new("⌨️ Press any key...").color(Color32::from_rgb(255, 200, 80)));
                            if ui.small_button("Cancel").clicked() {
                                self.is_assigning_trigger = false;
                                let _ = self.engine.cmd_tx.send(EngineCommand::CancelKeyListen);
                            }
                        } else {
                            let btn_label = match m_clone.trigger_key {
                                Some(k) => format!("Key: {}", format_key_name(k)),
                                None => "Assign Key".to_string(),
                            };
                            if ui.button(btn_label).clicked() {
                                self.is_assigning_trigger = true;
                                let _ = self.engine.cmd_tx.send(EngineCommand::ListenForTriggerKey);
                            }

                            if m_clone.trigger_key.is_some() {
                                if ui.small_button("Clear").clicked() {
                                    if let Some(target) = self.get_selected_macro_mut() {
                                        target.trigger_key = None;
                                        self.save_and_sync();
                                    }
                                }
                            }
                        }

                        ui.separator();

                        ui.label(RichText::new("Trigger Mode:").strong());
                        let mut mode = m_clone.trigger_mode;
                        let mode_name = mode.name();
                        egui::ComboBox::from_id_salt("trigger_mode_combo")
                            .selected_text(mode_name)
                            .show_ui(ui, |ui| {
                                for m_opt in [TriggerMode::Once, TriggerMode::ToggleLoop, TriggerMode::HoldLoop] {
                                    if ui.selectable_value(&mut mode, m_opt, m_opt.name()).clicked() {
                                        if let Some(target) = self.get_selected_macro_mut() {
                                            target.trigger_mode = mode;
                                            self.save_and_sync();
                                        }
                                    }
                                }
                            });

                        if mode != TriggerMode::Once {
                            ui.separator();
                            ui.label("Loop Count:");
                            let mut reps = m_clone.repeat_count;
                            if ui.add(egui::DragValue::new(&mut reps).range(0..=10000).speed(1)).changed() {
                                if let Some(target) = self.get_selected_macro_mut() {
                                    target.repeat_count = reps;
                                    self.save_and_sync();
                                }
                            }
                            if reps == 0 {
                                ui.label(RichText::new("(Infinite)").weak().small());
                            } else {
                                ui.label(RichText::new("(Times)").weak().small());
                            }
                        }
                    });
                });

                ui.add_space(8.0);

                // Add Action Tool Bar
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Add Action:").strong());

                    egui::ComboBox::from_id_salt("action_type_selector")
                        .selected_text(match self.selected_action_kind {
                            ActionKind::KeyPress => "Key Press (Click)",
                            ActionKind::KeyDown => "Key Down",
                            ActionKind::KeyUp => "Key Up",
                            ActionKind::MouseClick => "Mouse Click",
                            ActionKind::MouseDown => "Mouse Down",
                            ActionKind::MouseUp => "Mouse Up",
                            ActionKind::MouseMove => "Mouse Move",
                            ActionKind::MouseWheel => "Mouse Wheel",
                            ActionKind::Delay => "Delay (Sleep)",
                            ActionKind::TypeText => "Type Text",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_action_kind, ActionKind::KeyPress, "Key Press (Click)");
                            ui.selectable_value(&mut self.selected_action_kind, ActionKind::KeyDown, "Key Down");
                            ui.selectable_value(&mut self.selected_action_kind, ActionKind::KeyUp, "Key Up");
                            ui.selectable_value(&mut self.selected_action_kind, ActionKind::MouseClick, "Mouse Click");
                            ui.selectable_value(&mut self.selected_action_kind, ActionKind::MouseDown, "Mouse Down");
                            ui.selectable_value(&mut self.selected_action_kind, ActionKind::MouseUp, "Mouse Up");
                            ui.selectable_value(&mut self.selected_action_kind, ActionKind::MouseMove, "Mouse Move");
                            ui.selectable_value(&mut self.selected_action_kind, ActionKind::MouseWheel, "Mouse Wheel");
                            ui.selectable_value(&mut self.selected_action_kind, ActionKind::Delay, "Delay (Sleep)");
                            ui.selectable_value(&mut self.selected_action_kind, ActionKind::TypeText, "Type Text");
                        });

                    match self.selected_action_kind {
                        ActionKind::KeyPress => {
                            ui.label("Key Code:");
                            let mut code = self.action_key.0;
                            if ui.add(egui::DragValue::new(&mut code).range(1..=500)).changed() {
                                self.action_key = KeyCode(code);
                            }
                            ui.label(format!("({})", format_key_name(self.action_key)));

                            ui.label("Hold Duration (ms):");
                            ui.add(egui::DragValue::new(&mut self.action_hold_ms).range(5..=5000));
                        }
                        ActionKind::KeyDown | ActionKind::KeyUp => {
                            ui.label("Key Code:");
                            let mut code = self.action_key.0;
                            if ui.add(egui::DragValue::new(&mut code).range(1..=500)).changed() {
                                self.action_key = KeyCode(code);
                            }
                            ui.label(format!("({})", format_key_name(self.action_key)));
                        }
                        ActionKind::MouseClick => {
                            egui::ComboBox::from_id_salt("mouse_btn_sel")
                                .selected_text(self.action_mouse_btn.name())
                                .show_ui(ui, |ui| {
                                    for b in [MouseButton::Left, MouseButton::Right, MouseButton::Middle, MouseButton::Side, MouseButton::Extra] {
                                        ui.selectable_value(&mut self.action_mouse_btn, b, b.name());
                                    }
                                });
                            ui.label("Hold (ms):");
                            ui.add(egui::DragValue::new(&mut self.action_hold_ms).range(5..=5000));
                        }
                        ActionKind::MouseDown | ActionKind::MouseUp => {
                            egui::ComboBox::from_id_salt("mouse_btn_sel_simple")
                                .selected_text(self.action_mouse_btn.name())
                                .show_ui(ui, |ui| {
                                    for b in [MouseButton::Left, MouseButton::Right, MouseButton::Middle, MouseButton::Side, MouseButton::Extra] {
                                        ui.selectable_value(&mut self.action_mouse_btn, b, b.name());
                                    }
                                });
                        }
                        ActionKind::MouseMove => {
                            ui.label("dx:");
                            ui.add(egui::DragValue::new(&mut self.action_mouse_dx).range(-5000..=5000));
                            ui.label("dy:");
                            ui.add(egui::DragValue::new(&mut self.action_mouse_dy).range(-5000..=5000));
                        }
                        ActionKind::MouseWheel => {
                            ui.label("Amount (+ up, - down):");
                            ui.add(egui::DragValue::new(&mut self.action_wheel_delta).range(-50..=50));
                        }
                        ActionKind::Delay => {
                            ui.label("Duration (ms):");
                            ui.add(egui::DragValue::new(&mut self.action_delay_ms).range(1..=60000));
                        }
                        ActionKind::TypeText => {
                            ui.label("Text:");
                            ui.text_edit_singleline(&mut self.action_text);
                        }
                    }

                    if ui.button(RichText::new("+ Add").strong().color(Color32::from_rgb(100, 200, 255))).clicked() {
                        let action = match self.selected_action_kind {
                            ActionKind::KeyPress => Action::KeyPress {
                                key: self.action_key,
                                hold_ms: self.action_hold_ms,
                            },
                            ActionKind::KeyDown => Action::KeyDown(self.action_key),
                            ActionKind::KeyUp => Action::KeyUp(self.action_key),
                            ActionKind::MouseClick => Action::MouseClick {
                                button: self.action_mouse_btn,
                                hold_ms: self.action_hold_ms,
                            },
                            ActionKind::MouseDown => Action::MouseDown(self.action_mouse_btn),
                            ActionKind::MouseUp => Action::MouseUp(self.action_mouse_btn),
                            ActionKind::MouseMove => Action::MouseMove {
                                dx: self.action_mouse_dx,
                                dy: self.action_mouse_dy,
                            },
                            ActionKind::MouseWheel => Action::MouseWheel {
                                delta: self.action_wheel_delta,
                            },
                            ActionKind::Delay => Action::Delay(self.action_delay_ms),
                            ActionKind::TypeText => Action::TypeText(self.action_text.clone()),
                        };

                        if let Some(target) = self.get_selected_macro_mut() {
                            target.actions.push(action);
                            self.save_and_sync();
                        }
                    }
                });

                ui.separator();

                // Actions List
                ui.label(RichText::new(format!("Action Sequence ({} steps):", m_clone.actions.len())).strong());

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut swap_indices = None;
                    let mut remove_index = None;

                    for (idx, action) in m_clone.actions.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("#{}:", idx + 1)).weak().monospace());

                            let icon = match action {
                                Action::KeyDown(_) | Action::KeyUp(_) | Action::KeyPress { .. } => "⌨️",
                                Action::MouseDown(_) | Action::MouseUp(_) | Action::MouseClick { .. } | Action::MouseMove { .. } | Action::MouseWheel { .. } => "🖱️",
                                Action::Delay(_) => "⏳",
                                Action::TypeText(_) => "🔤",
                            };
                            ui.label(icon);

                            ui.label(RichText::new(action.summary()).monospace());

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("❌").on_hover_text("Delete Action").clicked() {
                                    remove_index = Some(idx);
                                }
                                if idx + 1 < m_clone.actions.len() {
                                    if ui.small_button("⬇").on_hover_text("Move Down").clicked() {
                                        swap_indices = Some((idx, idx + 1));
                                    }
                                }
                                if idx > 0 {
                                    if ui.small_button("⬆").on_hover_text("Move Up").clicked() {
                                        swap_indices = Some((idx, idx - 1));
                                    }
                                }
                            });
                        });
                        ui.separator();
                    }

                    if let Some(rm_idx) = remove_index {
                        if let Some(target) = self.get_selected_macro_mut() {
                            if rm_idx < target.actions.len() {
                                target.actions.remove(rm_idx);
                                self.save_and_sync();
                            }
                        }
                    }

                    if let Some((a, b)) = swap_indices {
                        if let Some(target) = self.get_selected_macro_mut() {
                            if a < target.actions.len() && b < target.actions.len() {
                                target.actions.swap(a, b);
                                self.save_and_sync();
                            }
                        }
                    }
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("Select a macro from the sidebar or click + New to create one.").weak());
                });
            }
        });
    }
}
