use crate::model::{Action, AppConfig, KeyCode, Macro, MouseButton, TriggerMode};
use crate::virtual_device::VirtualInput;
use evdev::EventType;
use parking_lot::Mutex;
use std::collections::HashSet;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum EngineCommand {
    UpdateConfig(AppConfig),
    StartRecording,
    StopRecording,
    ListenForTriggerKey,
    CancelKeyListen,
    RunMacro(Macro),
    StopMacro(String),
    StopAll,
}

#[derive(Debug, Clone)]
pub enum EngineEvent {
    RecordingAction(Action),
    RecordingFinished(Vec<Action>),
    KeyDetected(KeyCode),
    MacroStarted(String),
    MacroStopped(String),
    Status {
        accessible_devices: usize,
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
    pub fn new(initial_config: AppConfig) -> Self {
        let (cmd_tx, cmd_rx) = channel();
        let (event_tx, event_rx) = channel();
        let is_running = Arc::new(AtomicBool::new(true));

        let runner = EngineRunner::new(
            initial_config,
            cmd_rx,
            event_tx,
            Arc::clone(&is_running),
        );

        thread::Builder::new()
            .name("linmacro_engine".into())
            .spawn(move || {
                runner.run();
            })
            .expect("Engine thread failed to spawn");

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
    config: AppConfig,
    cmd_rx: Receiver<EngineCommand>,
    event_tx: Sender<EngineEvent>,
    is_running: Arc<AtomicBool>,
    virtual_input: Option<Arc<Mutex<VirtualInput>>>,
    recording: bool,
    recording_actions: Vec<Action>,
    last_record_time: Option<Instant>,
    listening_for_trigger: bool,
    active_macro_stops: Arc<Mutex<HashSet<String>>>,
    active_keys_held: HashSet<KeyCode>,
}

impl EngineRunner {
    fn new(
        config: AppConfig,
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
                    "Virtual controller could not be initialized: {}. Check /dev/uinput permissions.",
                    e
                ));
            }
        }

        let runner = Self {
            config,
            cmd_rx,
            event_tx,
            is_running,
            virtual_input,
            recording: false,
            recording_actions: Vec::new(),
            last_record_time: None,
            listening_for_trigger: false,
            active_macro_stops: Arc::new(Mutex::new(HashSet::new())),
            active_keys_held: HashSet::new(),
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
            accessible_devices: count,
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
            .name("linmacro_device_scanner".into())
            .spawn(move || {
                let mut watched_paths = HashSet::new();

                while is_running.load(Ordering::Relaxed) {
                    for (path, mut device) in evdev::enumerate() {
                        let name = device.name().unwrap_or("").to_string();
                        // Ignore our own virtual device
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

                        // Set nonblocking to prevent thread stalls
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
                                        Err(_) => {
                                            break;
                                        }
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
            EngineCommand::StartRecording => {
                self.recording = true;
                self.recording_actions.clear();
                self.last_record_time = Some(Instant::now());
            }
            EngineCommand::StopRecording => {
                self.recording = false;
                self.last_record_time = None;
                let actions = std::mem::take(&mut self.recording_actions);
                let _ = self.event_tx.send(EngineEvent::RecordingFinished(actions));
            }
            EngineCommand::ListenForTriggerKey => {
                self.listening_for_trigger = true;
            }
            EngineCommand::CancelKeyListen => {
                self.listening_for_trigger = false;
            }
            EngineCommand::RunMacro(m) => {
                self.execute_macro(m);
            }
            EngineCommand::StopMacro(id) => {
                let mut stops = self.active_macro_stops.lock();
                stops.insert(id.clone());
                let _ = self.event_tx.send(EngineEvent::MacroStopped(id));
            }
            EngineCommand::StopAll => {
                {
                    let mut stops = self.active_macro_stops.lock();
                    for m in &self.config.macros {
                        stops.insert(m.id.clone());
                        let _ = self.event_tx.send(EngineEvent::MacroStopped(m.id.clone()));
                    }
                }
                self.release_all_held_keys();
            }
        }
    }

    fn handle_key_event(&mut self, key: KeyCode, value: i32) {
        if value == 1 && key == self.config.killswitch_key {
            self.handle_command(EngineCommand::StopAll);
            return;
        }

        if self.listening_for_trigger && value == 1 {
            self.listening_for_trigger = false;
            let _ = self.event_tx.send(EngineEvent::KeyDetected(key));
            return;
        }

        if self.recording {
            if value == 0 || value == 1 {
                let now = Instant::now();
                if let Some(last) = self.last_record_time {
                    let elapsed = now.duration_since(last).as_millis() as u64;
                    if elapsed >= 10 {
                        let delay_action = Action::Delay(elapsed);
                        self.recording_actions.push(delay_action.clone());
                        let _ = self.event_tx.send(EngineEvent::RecordingAction(delay_action));
                    }
                }
                self.last_record_time = Some(now);

                let action = if value == 1 {
                    Action::KeyDown(key)
                } else {
                    Action::KeyUp(key)
                };
                self.recording_actions.push(action.clone());
                let _ = self.event_tx.send(EngineEvent::RecordingAction(action));
            }
            return;
        }

        if !self.config.global_enabled {
            return;
        }

        for m in self.config.macros.clone() {
            if !m.enabled {
                continue;
            }
            if let Some(trigger) = m.trigger_key {
                if trigger == key {
                    match m.trigger_mode {
                        TriggerMode::Once => {
                            if value == 1 {
                                self.execute_macro(m);
                            }
                        }
                        TriggerMode::ToggleLoop => {
                            if value == 1 {
                                let mut stops = self.active_macro_stops.lock();
                                if stops.contains(&m.id) {
                                    stops.insert(m.id.clone());
                                    let _ = self.event_tx.send(EngineEvent::MacroStopped(m.id.clone()));
                                } else {
                                    drop(stops);
                                    self.execute_macro(m);
                                }
                            }
                        }
                        TriggerMode::HoldLoop => {
                            if value == 1 {
                                self.execute_macro(m);
                            } else if value == 0 {
                                let mut stops = self.active_macro_stops.lock();
                                stops.insert(m.id.clone());
                                let _ = self.event_tx.send(EngineEvent::MacroStopped(m.id.clone()));
                            }
                        }
                    }
                }
            }
        }
    }

    fn execute_macro(&mut self, m: Macro) {
        let macro_id = m.id.clone();
        let active_stops = Arc::clone(&self.active_macro_stops);
        let event_tx = self.event_tx.clone();

        let vi_arc = match &self.virtual_input {
            Some(vi) => Arc::clone(vi),
            None => {
                let _ = event_tx.send(EngineEvent::Status {
                    accessible_devices: 0,
                    virtual_device_ok: false,
                    error: Some("Virtual controller not initialized.".into()),
                });
                return;
            }
        };

        {
            let mut stops = active_stops.lock();
            stops.remove(&macro_id);
        }

        let _ = event_tx.send(EngineEvent::MacroStarted(macro_id.clone()));

        thread::Builder::new()
            .name(format!("macro_{}", macro_id))
            .spawn(move || {
                let is_loop = m.trigger_mode != TriggerMode::Once;
                let mut iterations = 0;

                'outer: loop {
                    {
                        let stops = active_stops.lock();
                        if stops.contains(&macro_id) {
                            break 'outer;
                        }
                    }

                    for action in &m.actions {
                        {
                            let stops = active_stops.lock();
                            if stops.contains(&macro_id) {
                                break 'outer;
                            }
                        }

                        match action {
                            Action::KeyDown(k) => {
                                let mut vi = vi_arc.lock();
                                let _ = vi.send_key(*k, true);
                            }
                            Action::KeyUp(k) => {
                                let mut vi = vi_arc.lock();
                                let _ = vi.send_key(*k, false);
                            }
                            Action::KeyPress { key, hold_ms } => {
                                {
                                    let mut vi = vi_arc.lock();
                                    let _ = vi.send_key(*key, true);
                                }
                                if *hold_ms > 0 {
                                    thread::sleep(Duration::from_millis(*hold_ms));
                                }
                                {
                                    let mut vi = vi_arc.lock();
                                    let _ = vi.send_key(*key, false);
                                }
                            }
                            Action::MouseDown(b) => {
                                let mut vi = vi_arc.lock();
                                let _ = vi.send_mouse_button(*b, true);
                            }
                            Action::MouseUp(b) => {
                                let mut vi = vi_arc.lock();
                                let _ = vi.send_mouse_button(*b, false);
                            }
                            Action::MouseClick { button, hold_ms } => {
                                {
                                    let mut vi = vi_arc.lock();
                                    let _ = vi.send_mouse_button(*button, true);
                                }
                                if *hold_ms > 0 {
                                    thread::sleep(Duration::from_millis(*hold_ms));
                                }
                                {
                                    let mut vi = vi_arc.lock();
                                    let _ = vi.send_mouse_button(*button, false);
                                }
                            }
                            Action::MouseMove { dx, dy } => {
                                let mut vi = vi_arc.lock();
                                let _ = vi.mouse_move(*dx, *dy);
                            }
                            Action::MouseWheel { delta } => {
                                let mut vi = vi_arc.lock();
                                let _ = vi.mouse_wheel(*delta);
                            }
                            Action::Delay(ms) => {
                                let slices = ms / 5;
                                let rem = ms % 5;
                                for _ in 0..slices {
                                    {
                                        let stops = active_stops.lock();
                                        if stops.contains(&macro_id) {
                                            break 'outer;
                                        }
                                    }
                                    thread::sleep(Duration::from_millis(5));
                                }
                                if rem > 0 {
                                    thread::sleep(Duration::from_millis(rem));
                                }
                            }
                            Action::TypeText(text) => {
                                let mut vi = vi_arc.lock();
                                let _ = vi.type_text(text, 10);
                            }
                        }
                    }

                    iterations += 1;
                    if !is_loop {
                        break;
                    }
                    if m.repeat_count > 0 && iterations >= m.repeat_count {
                        break;
                    }
                }

                {
                    let mut stops = active_stops.lock();
                    stops.remove(&macro_id);
                }
                let _ = event_tx.send(EngineEvent::MacroStopped(macro_id));
            })
            .ok();
    }

    fn release_all_held_keys(&mut self) {
        if let Some(vi_arc) = &self.virtual_input {
            let mut vi = vi_arc.lock();
            for key in self.active_keys_held.drain() {
                let _ = vi.send_key(key, false);
            }
            for btn in [
                MouseButton::Left,
                MouseButton::Right,
                MouseButton::Middle,
                MouseButton::Side,
                MouseButton::Extra,
            ] {
                let _ = vi.send_mouse_button(btn, false);
            }
        }
    }
}
