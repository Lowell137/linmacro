use crate::engine::{EngineCommand, EngineEvent, EngineHandle};
use crate::model::{
    egui_key_to_keycode, format_key_name, ClickMode, ClickerConfig,
    MouseButton,
};
use eframe::egui::{self, Color32, RichText, Vec2};

pub struct LinMacroApp {
    config: ClickerConfig,
    engine: EngineHandle,
    is_clicking: bool,
    is_assigning_hotkey: bool,
    total_clicks: u64,
    status_error: Option<String>,
}

impl LinMacroApp {
    pub fn new() -> Self {
        let config = ClickerConfig::load();
        let engine = EngineHandle::new(config.clone());

        Self {
            config,
            engine,
            is_clicking: false,
            is_assigning_hotkey: false,
            total_clicks: 0,
            status_error: None,
        }
    }

    fn poll_events(&mut self) {
        while let Ok(event) = self.engine.event_rx.try_recv() {
            match event {
                EngineEvent::ClickingStarted => {
                    self.is_clicking = true;
                }
                EngineEvent::ClickingStopped => {
                    self.is_clicking = false;
                }
                EngineEvent::TotalClicks(n) => {
                    self.total_clicks = n;
                }
                EngineEvent::KeyDetected(key) => {
                    self.is_assigning_hotkey = false;
                    self.config.hotkey = Some(key);
                    self.save_and_sync();
                }
                EngineEvent::Status { error, .. } => {
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
}

impl eframe::App for LinMacroApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_events();

        // Direct window key capture when assigning hotkey
        if self.is_assigning_hotkey {
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
                    self.is_assigning_hotkey = false;
                    let _ = self.engine.cmd_tx.send(EngineCommand::CancelKeyListen);
                    self.config.hotkey = Some(kc);
                    self.save_and_sync();
                }
            }
        }

        if self.is_clicking || self.is_assigning_hotkey {
            ui.ctx().request_repaint();
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(4.0);

            // Title & Status
            ui.horizontal(|ui| {
                ui.heading(RichText::new("⚡ LinMacro").strong().color(Color32::from_rgb(100, 200, 255)));
                ui.label(RichText::new("Auto Clicker").weak());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.is_clicking {
                        ui.label(RichText::new("🟢 ACTIVE").strong().color(Color32::from_rgb(80, 240, 120)));
                    } else {
                        ui.label(RichText::new("⚪ IDLE").weak());
                    }
                });
            });

            ui.separator();
            ui.add_space(6.0);

            // 1. Hotkey Section
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Activation Key:").strong());

                    if self.is_assigning_hotkey {
                        ui.label(RichText::new("⌨️ Press ANY key...").color(Color32::from_rgb(255, 200, 80)).strong());
                        if ui.button("Cancel").clicked() {
                            self.is_assigning_hotkey = false;
                            let _ = self.engine.cmd_tx.send(EngineCommand::CancelKeyListen);
                        }
                    } else {
                        let key_label = match self.config.hotkey {
                            Some(k) => format!("Hotkey: [{}]", format_key_name(k)),
                            None => "Click to Set Hotkey".to_string(),
                        };

                        if ui.button(RichText::new(key_label).strong()).clicked() {
                            self.is_assigning_hotkey = true;
                            let _ = self.engine.cmd_tx.send(EngineCommand::ListenForTriggerKey);
                        }

                        if self.config.hotkey.is_some() {
                            if ui.small_button("✕ Clear").on_hover_text("Remove Hotkey").clicked() {
                                self.config.hotkey = None;
                                self.save_and_sync();
                            }
                        }
                    }
                });
            });

            ui.add_space(8.0);

            // 2. CPS Speed Control
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Click Speed (CPS):").strong());
                    ui.label(RichText::new(format!("{} CPS", self.config.cps)).strong().color(Color32::from_rgb(100, 200, 255)));
                    let delay_ms = 1000 / self.config.cps.max(1);
                    ui.label(RichText::new(format!("({} ms delay)", delay_ms)).weak().small());
                });

                ui.add_space(4.0);

                let mut cps = self.config.cps;
                if ui.add(egui::Slider::new(&mut cps, 1..=100).text("Clicks / Second")).changed() {
                    self.config.cps = cps;
                    self.save_and_sync();
                }

                ui.add_space(4.0);

                // Quick Presets
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Presets:").weak().small());
                    for preset in [5, 10, 20, 50, 100] {
                        let is_current = self.config.cps == preset;
                        let text = RichText::new(format!("{} CPS", preset));
                        let text = if is_current { text.strong().color(Color32::from_rgb(100, 200, 255)) } else { text };
                        if ui.selectable_label(is_current, text).clicked() {
                            self.config.cps = preset;
                            self.save_and_sync();
                        }
                    }
                });
            });

            ui.add_space(8.0);

            // 3. Mouse Button & Mode Settings
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Button:").strong());
                    for btn in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
                        if ui.radio_value(&mut self.config.button, btn, btn.name()).clicked() {
                            self.save_and_sync();
                        }
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Mode:").strong());
                    if ui.radio_value(&mut self.config.mode, ClickMode::Toggle, "Toggle (Press to Start / Stop)").clicked() {
                        self.save_and_sync();
                    }
                    if ui.radio_value(&mut self.config.mode, ClickMode::Hold, "Hold (While key is held)").clicked() {
                        self.save_and_sync();
                    }
                });
            });

            ui.add_space(14.0);

            // 4. BIG PROMINENT START / STOP BUTTON
            let btn_size = Vec2::new(ui.available_width(), 50.0);
            if self.is_clicking {
                let text = format!("⏹️ STOP AUTO CLICKER (Clicks: {})", self.total_clicks);
                let stop_btn = egui::Button::new(RichText::new(text).heading().strong().color(Color32::WHITE))
                    .fill(Color32::from_rgb(220, 60, 60));
                if ui.add_sized(btn_size, stop_btn).clicked() {
                    let _ = self.engine.cmd_tx.send(EngineCommand::StopClicking);
                }
            } else {
                let hotkey_hint = match self.config.hotkey {
                    Some(k) => format!(" (or press {})", format_key_name(k)),
                    None => "".to_string(),
                };
                let text = format!("▶ START AUTO CLICKER{}", hotkey_hint);
                let start_btn = egui::Button::new(RichText::new(text).heading().strong().color(Color32::WHITE))
                    .fill(Color32::from_rgb(50, 150, 240));
                if ui.add_sized(btn_size, start_btn).clicked() {
                    let _ = self.engine.cmd_tx.send(EngineCommand::StartClicking);
                }
            }

            ui.add_space(10.0);

            // Footer info
            ui.separator();
            ui.horizontal(|ui| {
                let kill_name = format_key_name(self.config.killswitch);
                ui.label(RichText::new(format!("Emergency Killswitch: [{}]", kill_name)).weak().small());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(err) = &self.status_error {
                        ui.label(RichText::new(err).color(Color32::from_rgb(255, 100, 100)).small());
                    } else {
                        ui.label(RichText::new("Wayland & X11 Ready").weak().small());
                    }
                });
            });
        });
    }
}
