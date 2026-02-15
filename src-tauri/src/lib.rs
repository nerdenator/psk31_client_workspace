//! PSK-31 Desktop Client
//!
//! A cross-platform application for sending and receiving PSK-31 ham radio messages.
//!
//! ## Architecture (Hexagonal / Ports & Adapters)
//!
//! - `domain/` - Pure domain types, no I/O dependencies
//! - `ports/` - Trait definitions (interfaces) for external dependencies
//! - `dsp/` - Signal processing (pure functions, no I/O)
//! - `modem/` - PSK-31 protocol logic (varicode, encoder, decoder)
//! - `adapters/` - Implementations of ports (cpal audio, serialport, FT-991A)
//! - `commands/` - Tauri command handlers (driving adapters)
//! - `state/` - Application state management

// Core domain (pure, no I/O)
pub mod domain;
pub mod dsp;
pub mod modem;
pub mod ports;

// Adapters (external I/O)
pub mod adapters;

// Tauri integration
pub mod commands;
pub mod menu;
pub mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .setup(|app| {
            menu::setup_menu(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Audio commands
            commands::audio::list_audio_devices,
            // Serial commands
            commands::serial::list_serial_ports,
            commands::serial::connect_serial,
            commands::serial::disconnect_serial,
            // Radio commands
            commands::radio::ptt_on,
            commands::radio::ptt_off,
            commands::radio::get_frequency,
            commands::radio::set_frequency,
            commands::radio::get_mode,
            commands::radio::set_mode,
            // Configuration commands
            commands::config::save_configuration,
            commands::config::load_configuration,
            commands::config::list_configurations,
            commands::config::delete_configuration,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
