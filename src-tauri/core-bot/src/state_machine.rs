use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use serde_json::json;

use crate::config::BotConfig;
use crate::capture::ScreenCapture;
use crate::controller::GamepadController;

#[derive(Debug, Clone, PartialEq)]
pub enum FSMState {
    Idle,
    Running,
    Stage1,
    Stage2,
    Stage3,
    Stage4,
    Paused,
    Stopping,
    Error,
    TestInputs,
}

impl FSMState {
    pub fn as_str(&self) -> &'static str {
        match self {
            FSMState::Idle => "idle",
            FSMState::Running => "running",
            FSMState::Stage1 => "stage_1",
            FSMState::Stage2 => "stage_2",
            FSMState::Stage3 => "stage_3",
            FSMState::Stage4 => "stage_4",
            FSMState::Paused => "paused",
            FSMState::Stopping => "stopping",
            FSMState::Error => "error",
            FSMState::TestInputs => "test_inputs",
        }
    }
}

pub struct BotLogger {
    app_handle: Option<AppHandle>,
}

impl BotLogger {
    pub fn new(app_handle: Option<AppHandle>) -> Self {
        Self { app_handle }
    }

    pub fn log(&self, level: &str, message: &str) {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("{} [{}] {}", now, level.to_uppercase(), message);
        if let Some(ref handle) = self.app_handle {
            let payload = json!({
                "type": "log",
                "level": level.to_string(),
                "message": message.to_string(),
                "ts": null, // UI automatically adds local timestamp if null
            });
            let _ = handle.emit("log", payload);
        }
    }

    pub fn info(&self, msg: &str) { self.log("info", msg); }
    pub fn warn(&self, msg: &str) { self.log("warn", msg); }
    pub fn error(&self, msg: &str) { self.log("error", msg); }
    pub fn debug(&self, msg: &str) { self.log("debug", msg); }
}

pub struct BotFSM {
    pub config: Arc<Mutex<BotConfig>>,
    pub state: Arc<Mutex<FSMState>>,
    pub sub_state: Arc<Mutex<Option<String>>>,
    pub session_sp: Arc<Mutex<i32>>,
    pub current_cycle: Arc<Mutex<i32>>,
    pub session_cr: Arc<Mutex<i32>>,
    pub session_wspins: Arc<Mutex<i32>>,
    pub session_driving_seconds: Arc<Mutex<f64>>,
    
    pub is_stop_requested: Arc<AtomicBool>,
    pub is_paused: Arc<AtomicBool>,
    pre_pause_state: Arc<Mutex<FSMState>>,
    
    pub capture: Arc<Mutex<ScreenCapture>>,
    pub pad: Arc<Mutex<GamepadController>>,
    pub logger: Arc<BotLogger>,
    
    app_handle: Option<AppHandle>,
    thread_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl BotFSM {
    pub fn new(app_handle: Option<AppHandle>) -> Self {
        let config = Arc::new(Mutex::new(BotConfig::load()));
        let state = Arc::new(Mutex::new(FSMState::Idle));
        let sub_state = Arc::new(Mutex::new(None));
        let session_sp = Arc::new(Mutex::new(0));
        let current_cycle = Arc::new(Mutex::new(0));
        let session_cr = Arc::new(Mutex::new(0));
        let session_wspins = Arc::new(Mutex::new(0));
        let session_driving_seconds = Arc::new(Mutex::new(0.0));
        let pre_pause_state = Arc::new(Mutex::new(FSMState::Idle));

        let is_stop_requested = Arc::new(AtomicBool::new(false));
        let is_paused = Arc::new(AtomicBool::new(false));

        let capture = Arc::new(Mutex::new(ScreenCapture::new()));
        let pad = Arc::new(Mutex::new(GamepadController::new()));
        let logger = Arc::new(BotLogger::new(app_handle.clone()));

        // Wire controller cancel check callback
        {
            let is_stop = Arc::clone(&is_stop_requested);
            let mut pad_lock = pad.lock().unwrap();
            pad_lock.set_cancel_fn(Some(Arc::new(move || {
                is_stop.load(Ordering::Relaxed)
            })));
        }

        Self {
            config,
            state,
            sub_state,
            session_sp,
            current_cycle,
            session_cr,
            session_wspins,
            session_driving_seconds,
            is_stop_requested,
            is_paused,
            pre_pause_state,
            capture,
            pad,
            logger,
            app_handle,
            thread_handle: Mutex::new(None),
        }
    }

    pub fn set_sub_state(&self, name: Option<String>) {
        *self.sub_state.lock().unwrap() = name;
        self.broadcast_status();
    }

    pub fn add_session_sp(&self, count: i32) {
        {
            let mut sp = self.session_sp.lock().unwrap();
            *sp = (*sp + count).clamp(-999, 999);
        }
        self.broadcast_status();
    }

    pub fn add_session_cr(&self, count: i32) {
        {
            let mut cr = self.session_cr.lock().unwrap();
            *cr += count;
        }
        self.broadcast_status();
    }

    pub fn add_session_wspins(&self, count: i32) {
        {
            let mut wspins = self.session_wspins.lock().unwrap();
            *wspins += count;
        }
        self.broadcast_status();
    }

    pub fn add_driving_time(&self, elapsed_secs: f64) {
        {
            let mut secs = self.session_driving_seconds.lock().unwrap();
            *secs += elapsed_secs;
            if *secs >= 345.0 {
                let periods = (*secs / 345.0).floor() as i32;
                *secs -= periods as f64 * 345.0;
                let mut cr = self.session_cr.lock().unwrap();
                *cr += periods * 160_000;
            }
        }
        self.broadcast_status();
    }

    pub fn broadcast_status(&self) {
        if let Some(ref handle) = self.app_handle {
            let status = json!({
                "type": "status",
                "state": self.state.lock().unwrap().as_str(),
                "sub_state": *self.sub_state.lock().unwrap(),
                "cycle": *self.current_cycle.lock().unwrap(),
                "session_sp": *self.session_sp.lock().unwrap(),
                "session_cr": *self.session_cr.lock().unwrap(),
                "session_wspins": *self.session_wspins.lock().unwrap(),
                "config": *self.config.lock().unwrap(),
            });
            let _ = handle.emit("status", status);
        }
    }

    pub fn wait_if_paused(&self) {
        if self.is_paused.load(Ordering::Relaxed) {
            {
                let mut pad_lock = self.pad.lock().unwrap();
                pad_lock.pause_inputs();
            }
            self.logger.info("Bot execution paused...");
            
            while self.is_paused.load(Ordering::Relaxed) && !self.is_stop_requested.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(50));
            }
            
            self.logger.info("Bot execution resumed...");
            {
                let mut pad_lock = self.pad.lock().unwrap();
                pad_lock.resume_inputs();
            }
        }
    }

    pub fn smart_sleep(&self, duration_s: f64) {
        let check_interval = 0.05f64;
        let mut elapsed = 0.0;
        while elapsed < duration_s {
            if self.is_stop_requested.load(Ordering::Relaxed) {
                break;
            }
            self.wait_if_paused();
            if self.is_stop_requested.load(Ordering::Relaxed) {
                break;
            }
            let sleep_time = check_interval.min(duration_s - elapsed);
            std::thread::sleep(Duration::from_secs_f64(sleep_time));
            elapsed += sleep_time;
        }
    }

    pub fn update_config(&self, new_config: BotConfig) {
        let mut cfg = self.config.lock().unwrap();
        *cfg = new_config;
        cfg.save();
        
        if let Some(ref handle) = self.app_handle {
            let _ = handle.emit("config_saved", json!({
                "type": "config_saved",
                "config": *cfg,
            }));
        }
    }

    pub fn set_state(&self, new_state: FSMState) {
        {
            let mut current = self.state.lock().unwrap();
            *current = new_state.clone();
        }
        
        // Reset sub_state when switching main stages (except when pausing/resuming)
        let is_paused_transition = new_state == FSMState::Paused;
        let is_resume_transition = {
            let pre_pause = self.pre_pause_state.lock().unwrap();
            new_state == *pre_pause
        };

        if !is_paused_transition && !is_resume_transition {
            *self.sub_state.lock().unwrap() = None;
        }

        self.broadcast_status();
    }

    // --- Control Commands ---

    pub fn start(&self) {
        let is_ok = {
            let mut current_state = self.state.lock().unwrap();
            if *current_state != FSMState::Idle {
                self.logger.warn(&format!("Cannot start: FSM is in state {:?}", *current_state));
                false
            } else {
                self.is_stop_requested.store(false, Ordering::Relaxed);
                self.is_paused.store(false, Ordering::Relaxed);
                 *self.current_cycle.lock().unwrap() = 0;
                 *self.session_sp.lock().unwrap() = 0;
                 *self.session_cr.lock().unwrap() = 0;
                 *self.session_wspins.lock().unwrap() = 0;
                 *self.session_driving_seconds.lock().unwrap() = 0.0;

                self.logger.info("Locating game window 'Forza Horizon 6'...");
                let found = {
                    let mut capture_lock = self.capture.lock().unwrap();
                    capture_lock.find_game_window()
                };

                if !found {
                    self.logger.error("Game window 'Forza Horizon 6' not found!");
                    *current_state = FSMState::Idle;
                    if let Some(ref handle) = self.app_handle {
                        let _ = handle.emit("error", json!({
                            "type": "error",
                            "stage": "startup",
                            "message": "Game window 'Forza Horizon 6' not found. Make sure the game is running."
                        }));
                    }
                    false
                } else {
                    self.logger.info("Activating game window 'Forza Horizon 6'...");
                    {
                        let capture_lock = self.capture.lock().unwrap();
                        if !capture_lock.focus_game_window() {
                            self.logger.warn("Failed to activate game window 'Forza Horizon 6'.");
                        }
                    }

                    self.logger.info("Connecting to virtual controller (ViGEmBus)...");
                    let mut pad_lock = self.pad.lock().unwrap();
                    if let Err(e) = pad_lock.try_connect() {
                        self.logger.error(&format!("Virtual controller connection failed: {}", e));
                        *current_state = FSMState::Idle;
                        if let Some(ref handle) = self.app_handle {
                            let _ = handle.emit("error", json!({
                                "type": "error",
                                "stage": "startup",
                                "message": format!("Virtual controller (ViGEmBus) connection failed. Please make sure the driver is installed. Details: {}", e)
                            }));
                        }
                        false
                    } else {
                        *current_state = FSMState::Running;
                        true
                    }
                }
            }
        };

        if !is_ok {
            return;
        }

        self.broadcast_status();

        // Spawn background thread for executing the farm loop
        let fsm_clone = self.clone_ptrs();
        let handle = std::thread::spawn(move || {
            fsm_clone.run_loop();
        });

        *self.thread_handle.lock().unwrap() = Some(handle);
        self.logger.info("Bot started");
    }

    pub fn stop(&self) {
        let state = {
            let s = self.state.lock().unwrap();
            s.clone()
        };
        if state == FSMState::Idle || state == FSMState::Stopping {
            return;
        }

        self.logger.info("Stopping bot...");
        self.is_stop_requested.store(true, Ordering::Relaxed);
        self.is_paused.store(false, Ordering::Relaxed); // Unblock if paused

        self.set_state(FSMState::Stopping);

        let mut handle_opt = self.thread_handle.lock().unwrap();
        if let Some(handle) = handle_opt.take() {
            let fsm_clone = self.clone_ptrs();
            let pad_clone = Arc::clone(&self.pad);
            let state_clone = Arc::clone(&self.state);
            let logger_clone = Arc::clone(&self.logger);

            std::thread::spawn(move || {
                let _ = handle.join();
                {
                    let mut pad_lock = pad_clone.lock().unwrap();
                    pad_lock.reset();
                }
                *state_clone.lock().unwrap() = FSMState::Idle;
                fsm_clone.broadcast_status();
                logger_clone.info("Bot stopped");
            });
        } else {
            {
                let mut pad_lock = self.pad.lock().unwrap();
                pad_lock.reset();
            }
            self.set_state(FSMState::Idle);
            self.logger.info("Bot stopped");
        }
    }

    pub fn pause(&self) {
        let current = {
            let s = self.state.lock().unwrap();
            s.clone()
        };
        if current != FSMState::Paused && current != FSMState::Idle && current != FSMState::Stopping && current != FSMState::Error {
            *self.pre_pause_state.lock().unwrap() = current;
            self.is_paused.store(true, Ordering::Relaxed);
            self.set_state(FSMState::Paused);
            self.logger.info("Bot paused");
        }
    }

    pub fn resume(&self) {
        let current = {
            let s = self.state.lock().unwrap();
            s.clone()
        };
        if current == FSMState::Paused {
            self.is_paused.store(false, Ordering::Relaxed);
            let restore = {
                let p = self.pre_pause_state.lock().unwrap();
                p.clone()
            };
            self.set_state(restore);
            self.logger.info("Bot resumed");
        }
    }

    pub fn clone_ptrs(&self) -> BotFSMContext {
        BotFSMContext {
            config: Arc::clone(&self.config),
            state: Arc::clone(&self.state),
            sub_state: Arc::clone(&self.sub_state),
            session_sp: Arc::clone(&self.session_sp),
            current_cycle: Arc::clone(&self.current_cycle),
            session_cr: Arc::clone(&self.session_cr),
            session_wspins: Arc::clone(&self.session_wspins),
            session_driving_seconds: Arc::clone(&self.session_driving_seconds),
            is_stop_requested: Arc::clone(&self.is_stop_requested),
            is_paused: Arc::clone(&self.is_paused),
            capture: Arc::clone(&self.capture),
            pad: Arc::clone(&self.pad),
            logger: Arc::clone(&self.logger),
            pre_pause_state: Arc::clone(&self.pre_pause_state),
            app_handle: self.app_handle.clone(),
        }
    }
}

// Thread-safe pointer context passed into background runner thread
#[derive(Clone)]
pub struct BotFSMContext {
    pub config: Arc<Mutex<BotConfig>>,
    pub state: Arc<Mutex<FSMState>>,
    pub sub_state: Arc<Mutex<Option<String>>>,
    pub session_sp: Arc<Mutex<i32>>,
    pub current_cycle: Arc<Mutex<i32>>,
    pub session_cr: Arc<Mutex<i32>>,
    pub session_wspins: Arc<Mutex<i32>>,
    pub session_driving_seconds: Arc<Mutex<f64>>,
    pub is_stop_requested: Arc<AtomicBool>,
    pub is_paused: Arc<AtomicBool>,
    pub capture: Arc<Mutex<ScreenCapture>>,
    pub pad: Arc<Mutex<GamepadController>>,
    pub logger: Arc<BotLogger>,
    #[allow(dead_code)]
    pre_pause_state: Arc<Mutex<FSMState>>,
    pub app_handle: Option<AppHandle>,
}

impl BotFSMContext {
    pub fn wait_if_paused(&self) {
        if self.is_paused.load(Ordering::Relaxed) {
            {
                let mut pad_lock = self.pad.lock().unwrap();
                pad_lock.pause_inputs();
            }
            self.logger.info("Bot execution paused...");
            while self.is_paused.load(Ordering::Relaxed) && !self.is_stop_requested.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(50));
            }
            self.logger.info("Bot execution resumed...");
            {
                let mut pad_lock = self.pad.lock().unwrap();
                pad_lock.resume_inputs();
            }
        }
    }

    pub fn smart_sleep(&self, duration_s: f64) {
        let check_interval = 0.05f64;
        let mut elapsed = 0.0;
        while elapsed < duration_s {
            if self.is_stop_requested.load(Ordering::Relaxed) {
                break;
            }
            self.wait_if_paused();
            if self.is_stop_requested.load(Ordering::Relaxed) {
                break;
            }
            let sleep_time = check_interval.min(duration_s - elapsed);
            std::thread::sleep(Duration::from_secs_f64(sleep_time));
            elapsed += sleep_time;
        }
    }

    pub fn set_sub_state(&self, name: Option<String>) {
        *self.sub_state.lock().unwrap() = name;
        self.broadcast_status();
    }

    pub fn add_session_sp(&self, count: i32) {
        {
            let mut sp = self.session_sp.lock().unwrap();
            *sp = (*sp + count).clamp(-999, 999);
        }
        self.broadcast_status();
    }

    pub fn add_session_cr(&self, count: i32) {
        {
            let mut cr = self.session_cr.lock().unwrap();
            *cr += count;
        }
        self.broadcast_status();
    }

    pub fn add_session_wspins(&self, count: i32) {
        {
            let mut wspins = self.session_wspins.lock().unwrap();
            *wspins += count;
        }
        self.broadcast_status();
    }

    pub fn add_driving_time(&self, elapsed_secs: f64) {
        {
            let mut secs = self.session_driving_seconds.lock().unwrap();
            *secs += elapsed_secs;
            if *secs >= 345.0 {
                let periods = (*secs / 345.0).floor() as i32;
                *secs -= periods as f64 * 345.0;
                let mut cr = self.session_cr.lock().unwrap();
                *cr += periods * 160_000;
            }
        }
        self.broadcast_status();
    }

    pub fn set_state(&self, new_state: FSMState) {
        *self.state.lock().unwrap() = new_state;
        *self.sub_state.lock().unwrap() = None;
        self.broadcast_status();
    }

    pub fn broadcast_status(&self) {
        if let Some(ref handle) = self.app_handle {
            let status = json!({
                "type": "status",
                "state": self.state.lock().unwrap().as_str(),
                "sub_state": *self.sub_state.lock().unwrap(),
                "cycle": *self.current_cycle.lock().unwrap(),
                "session_sp": *self.session_sp.lock().unwrap(),
                "session_cr": *self.session_cr.lock().unwrap(),
                "session_wspins": *self.session_wspins.lock().unwrap(),
                "config": *self.config.lock().unwrap(),
            });
            let _ = handle.emit("status", status);
        }
    }

    pub fn run_loop(&self) {
        crate::stages::run_loop(self);
    }
}
