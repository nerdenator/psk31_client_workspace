//! Application state

use std::sync::Mutex;
use crate::domain::{ModemConfig, ModemStatus};
use crate::ports::RadioControl;

/// Shared application state managed by Tauri
pub struct AppState {
    pub config: Mutex<ModemConfig>,
    pub status: Mutex<ModemStatus>,
    pub radio: Mutex<Option<Box<dyn RadioControl>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(ModemConfig::default()),
            status: Mutex::new(ModemStatus::default()),
            radio: Mutex::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
