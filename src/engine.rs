use crate::model::{ClickMode, ClickType, ClickerConfig, KeyCode};
use crate::virtual_device::VirtualInput;
use evdev::EventType;
use parking_lot::Mutex;
use std::collections::HashSet;
use std::io;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum EngineCommand {
    UpdateConfig(ClickerConfig),
    StartClicking,
    StopClicking,
    ToggleClicking,
    ListenForTriggerKey,
    CancelKeyListen,
}

#[derive(Debug, Clone)]
pub enum EngineEvent {
    ClickingStarted,
    ClickingStopped,
    TotalClicks(u64),
    KeyDetected(KeyCode),
    Status {
        devices_count: usize,
        virtual_device_ok: bool,
        error: Option<String>,
    },
}

pub struct EngineHandle {
    pub cmd_tx: Sender<EngineCommand>,
    pub event_rx: Receiver<EngineEvent>,
    is_running: Arc<AtomicBool>,
}

impl EngineHandle {
    pub fn new(config: ClickerConfig) -> Self {
        let (cmd_tx, cmd_rx) = channel();
        let (event_tx, event_rx) = channel();
        let is_running = Arc::new(AtomicBool::new(true));

        let runner = EngineRunner::new(config, cmd_rx, event_tx, Arc::clone(&is_running));

        thread::Builder::new()
            .name("linmacro_engine".into())
            .spawn(move || runner.run())
            .expect("Failed to spawn engine thread");

        Self {
            cmd_tx,
            event_rx,
            is_running,
        }
    }
}

impl Drop for EngineHandle {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}

struct EngineRunner {
    config: ClickerConfig,
    cmd_rx: Receiver<EngineCommand>,
    event_tx: Sender<EngineEvent>,
    is_running: Arc<AtomicBool>,
    virtual_input: Option<Arc<Mutex<VirtualInput>>>,
    is_clicking: Arc<AtomicBool>,
    total_clicks: Arc<AtomicU64>,
    listening_for_trigger: bool,
}

impl EngineRunner {
    fn new(
        config: ClickerConfig,
        cmd_rx: Receiver<EngineCommand>,
        event_tx: Sender<EngineEvent>,
        is_running: Arc<AtomicBool>,
    ) -> Self {
        let mut virtual_input = None;
        let mut err = None;

        match VirtualInput::new() {
            Ok(vi) => {
                virtual_input = Some(Arc::new(Mutex::new(vi)));
            }
            Err(e) => {
                err = Some(format!(
                    "Virtual mouse initialization failed: {}. Check /dev/uinput permissions.",
                    e
                ));
            }
        }

        let is_clicking = Arc::new(AtomicBool::new(false));
        let total_clicks = Arc::new(AtomicU64::new(0));

        let runner = Self {
            config,
            cmd_rx,
            event_tx,
            is_running,
            virtual_input,
            is_clicking,
            total_clicks,
            listening_for_trigger: false,
        };

        runner.emit_status(err);
        runner
    }

    fn emit_status(&self, err: Option<String>) {
        let count = evdev::enumerate()
            .filter(|(_, dev)| {
                dev.name()
                    .map(|n| !n.contains("LinMacro") && !n.contains("Virtual"))
                    .unwrap_or(true)
            })
            .count();

        let _ = self.event_tx.send(EngineEvent::Status {
            devices_count: count,
            virtual_device_ok: self.virtual_input.is_some(),
            error: err,
        });
    }

    fn run(mut self) {
        let (raw_tx, raw_rx) = channel::<(KeyCode, i32)>();
        self.spawn_device_watchers(raw_tx);

        while self.is_running.load(Ordering::Relaxed) {
            while let Ok(cmd) = self.cmd_rx.try_recv() {
                self.handle_command(cmd);
            }

            match raw_rx.recv_timeout(Duration::from_millis(5)) {
                Ok((key, value)) => {
                    self.handle_key_event(key, value);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    }

    fn spawn_device_watchers(&self, raw_tx: Sender<(KeyCode, i32)>) {
        let is_running = Arc::clone(&self.is_running);

        thread::Builder::new()
            .name("linmacro_scanner".into())
            .spawn(move || {
                let mut watched_paths = HashSet::new();

                while is_running.load(Ordering::Relaxed) {
                    for (path, mut device) in evdev::enumerate() {
                        let name = device.name().unwrap_or("").to_string();
                        if name.contains("LinMacro") || name.contains("Virtual") {
                            continue;
                        }

                        if !device.supported_events().contains(EventType::KEY) {
                            continue;
                        }

                        let path_str = path.to_string_lossy().to_string();
                        if watched_paths.contains(&path_str) {
                            continue;
                        }

                        let _ = device.set_nonblocking(true);
                        watched_paths.insert(path_str.clone());

                        let tx = raw_tx.clone();
                        let running_clone = Arc::clone(&is_running);

                        thread::Builder::new()
                            .name(format!("evdev_{}", path_str))
                            .spawn(move || {
                                while running_clone.load(Ordering::Relaxed) {
                                    match device.fetch_events() {
                                        Ok(events) => {
                                            for ev in events {
                                                if ev.event_type() == EventType::KEY {
                                                    let key = KeyCode(ev.code());
                                                    let val = ev.value();
                                                    let _ = tx.send((key, val));
                                                }
                                            }
                                        }
                                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                            thread::sleep(Duration::from_millis(5));
                                        }
                                        Err(_) => break,
                                    }
                                }
                            })
                            .ok();
                    }
                    thread::sleep(Duration::from_millis(1000));
                }
            })
            .ok();
    }

    fn handle_command(&mut self, cmd: EngineCommand) {
        match cmd {
            EngineCommand::UpdateConfig(cfg) => {
                self.config = cfg;
            }
            EngineCommand::StartClicking => {
                self.start_clicking();
            }
            EngineCommand::StopClicking => {
                self.stop_clicking();
            }
            EngineCommand::ToggleClicking => {
                if self.is_clicking.load(Ordering::Relaxed) {
                    self.stop_clicking();
                } else {
                    self.start_clicking();
                }
            }
            EngineCommand::ListenForTriggerKey => {
                self.listening_for_trigger = true;
            }
            EngineCommand::CancelKeyListen => {
                self.listening_for_trigger = false;
            }
        }
    }

    fn handle_key_event(&mut self, key: KeyCode, value: i32) {
        // Emergency killswitch: Pause/Break
        if value == 1 && key == self.config.killswitch {
            self.stop_clicking();
            return;
        }

        // Assigning key mode
        if self.listening_for_trigger && value == 1 {
            self.listening_for_trigger = false;
            let _ = self.event_tx.send(EngineEvent::KeyDetected(key));
            return;
        }

        // Check if key is configured hotkey
        if let Some(hotkey) = self.config.hotkey {
            if key == hotkey {
                match self.config.mode {
                    ClickMode::Toggle => {
                        if value == 1 {
                            self.handle_command(EngineCommand::ToggleClicking);
                        }
                    }
                    ClickMode::Hold => {
                        if value == 1 {
                            self.start_clicking();
                        } else if value == 0 {
                            self.stop_clicking();
                        }
                    }
                }
            }
        }
    }

    fn start_clicking(&mut self) {
        if self.is_clicking.swap(true, Ordering::Relaxed) {
            return; // already clicking
        }

        let _ = self.event_tx.send(EngineEvent::ClickingStarted);

        let vi_arc = match &self.virtual_input {
            Some(vi) => Arc::clone(vi),
            None => {
                self.is_clicking.store(false, Ordering::Relaxed);
                return;
            }
        };

        let is_clicking = Arc::clone(&self.is_clicking);
        let total_clicks = Arc::clone(&self.total_clicks);
        let event_tx = self.event_tx.clone();
        let cfg = self.config.clone();

        thread::Builder::new()
            .name("linmacro_click_loop".into())
            .spawn(move || {
                let cps = cfg.cps.clamp(1, 200) as u64;
                let interval = Duration::from_micros(1_000_000 / cps);
                let hold = Duration::from_millis(4);

                while is_clicking.load(Ordering::Relaxed) {
                    let start = Instant::now();

                    // Perform Click
                    {
                        let mut vi = vi_arc.lock();
                        let _ = vi.send_mouse_button(cfg.button, true);
                    }
                    thread::sleep(hold);
                    {
                        let mut vi = vi_arc.lock();
                        let _ = vi.send_mouse_button(cfg.button, false);
                    }

                    if cfg.click_type == ClickType::Double {
                        thread::sleep(Duration::from_millis(15));
                        {
                            let mut vi = vi_arc.lock();
                            let _ = vi.send_mouse_button(cfg.button, true);
                        }
                        thread::sleep(hold);
                        {
                            let mut vi = vi_arc.lock();
                            let _ = vi.send_mouse_button(cfg.button, false);
                        }
                    }

                    let count = total_clicks.fetch_add(1, Ordering::Relaxed) + 1;
                    if count % 5 == 0 {
                        let _ = event_tx.send(EngineEvent::TotalClicks(count));
                    }

                    // Precise sleep till next click
                    let elapsed = start.elapsed();
                    if elapsed < interval {
                        let remaining = interval - elapsed;
                        // Sleep in slices for instant cancellation
                        let slices = remaining.as_millis() / 2;
                        for _ in 0..slices {
                            if !is_clicking.load(Ordering::Relaxed) {
                                break;
                            }
                            thread::sleep(Duration::from_millis(2));
                        }
                        let rem_micros = remaining.as_micros() % 2000;
                        if rem_micros > 0 && is_clicking.load(Ordering::Relaxed) {
                            thread::sleep(Duration::from_micros(rem_micros as u64));
                        }
                    }
                }

                let final_count = total_clicks.load(Ordering::Relaxed);
                let _ = event_tx.send(EngineEvent::TotalClicks(final_count));
                let _ = event_tx.send(EngineEvent::ClickingStopped);
            })
            .ok();
    }

    fn stop_clicking(&mut self) {
        if self.is_clicking.swap(false, Ordering::Relaxed) {
            let _ = self.event_tx.send(EngineEvent::ClickingStopped);
            if let Some(vi) = &self.virtual_input {
                let mut vi = vi.lock();
                let _ = vi.send_mouse_button(self.config.button, false);
            }
        }
    }
}
