//! Radio control commands — PTT, frequency, mode
//!
//! Each command locks the radio from AppState, checks it's connected,
//! calls the trait method, and maps errors to String for Tauri IPC.

use crate::domain::{Frequency, Psk31Result};
use crate::ports::RadioControl;
use crate::state::AppState;
use tauri::State;

/// Lock the radio mutex, check it's connected, and run `f` on it.
/// Think of this like a Python context manager — handles the
/// boilerplate of acquiring the lock and checking the connection.
fn with_radio<T>(
    state: &State<AppState>,
    f: impl FnOnce(&mut Box<dyn RadioControl>) -> Psk31Result<T>,
) -> Result<T, String> {
    let mut guard = state
        .radio
        .lock()
        .map_err(|_| "Radio state corrupted".to_string())?;
    let radio = guard.as_mut().ok_or("Radio not connected")?;
    f(radio).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ptt_on(state: State<AppState>) -> Result<(), String> {
    with_radio(&state, |r| r.ptt_on())
}

#[tauri::command]
pub fn ptt_off(state: State<AppState>) -> Result<(), String> {
    with_radio(&state, |r| r.ptt_off())
}

#[tauri::command]
pub fn get_frequency(state: State<AppState>) -> Result<f64, String> {
    with_radio(&state, |r| r.get_frequency().map(|f| f.as_hz()))
}

#[tauri::command]
pub fn set_frequency(state: State<AppState>, freq_hz: f64) -> Result<(), String> {
    with_radio(&state, |r| r.set_frequency(Frequency::hz(freq_hz)))
}

#[tauri::command]
pub fn get_mode(state: State<AppState>) -> Result<String, String> {
    with_radio(&state, |r| r.get_mode())
}

#[tauri::command]
pub fn set_mode(state: State<AppState>, mode: String) -> Result<(), String> {
    with_radio(&state, |r| r.set_mode(&mode))
}
