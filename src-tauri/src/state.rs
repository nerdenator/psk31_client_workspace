//! Application state

use std::sync::Mutex;
use crate::domain::{ModemConfig, ModemStatus};

/// Shared application state managed by Tauri
pub struct AppState {
    pub config: Mutex<ModemConfig>,
    pub status: Mutex<ModemStatus>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(ModemConfig::default()),
            status: Mutex::new(ModemStatus::default()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
