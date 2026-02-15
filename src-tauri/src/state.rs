//! Application state

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use crate::domain::{ModemConfig, ModemStatus};
use crate::ports::RadioControl;

/// Shared application state managed by Tauri
pub struct AppState {
    pub config: Mutex<ModemConfig>,
    pub status: Mutex<ModemStatus>,
    pub radio: Mutex<Option<Box<dyn RadioControl>>>,
    /// Shared flag to signal the audio thread to stop
    pub audio_running: Arc<AtomicBool>,
    /// Handle to the audio processing thread (for clean shutdown)
    pub audio_thread: Mutex<Option<JoinHandle<()>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(ModemConfig::default()),
            status: Mutex::new(ModemStatus::default()),
            radio: Mutex::new(None),
            audio_running: Arc::new(AtomicBool::new(false)),
            audio_thread: Mutex::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
