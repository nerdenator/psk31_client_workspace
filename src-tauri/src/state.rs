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
    /// Shared flag to signal the TX thread to abort
    pub tx_abort: Arc<AtomicBool>,
    /// Handle to the TX thread (for clean shutdown)
    pub tx_thread: Mutex<Option<JoinHandle<()>>>,
    /// Shared flag to enable/disable the RX decoder in the audio thread
    pub rx_running: Arc<AtomicBool>,
    /// Carrier frequency for RX decoder (updated by click-to-tune)
    pub rx_carrier_freq: Arc<Mutex<f64>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(ModemConfig::default()),
            status: Mutex::new(ModemStatus::default()),
            radio: Mutex::new(None),
            audio_running: Arc::new(AtomicBool::new(false)),
            audio_thread: Mutex::new(None),
            tx_abort: Arc::new(AtomicBool::new(false)),
            tx_thread: Mutex::new(None),
            rx_running: Arc::new(AtomicBool::new(false)),
            rx_carrier_freq: Arc::new(Mutex::new(1000.0)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
