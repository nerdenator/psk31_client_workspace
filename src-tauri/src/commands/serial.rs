//! Serial port commands — list, connect, disconnect

use crate::adapters::ft991a::Ft991aRadio;
use crate::adapters::serial_port::SerialPortFactory;
use crate::domain::{RadioInfo, SerialPortInfo};
use crate::ports::{RadioControl, SerialFactory};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn list_serial_ports() -> Result<Vec<SerialPortInfo>, String> {
    SerialPortFactory::list_ports().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn connect_serial(
    state: State<AppState>,
    port: String,
    baud_rate: u32,
) -> Result<RadioInfo, String> {
    // Open serial connection
    let connection =
        SerialPortFactory::open(&port, baud_rate).map_err(|e| e.to_string())?;

    // Create FT-991A radio adapter (owns the serial connection)
    let mut radio = Ft991aRadio::new(connection);

    // Auto-detect: read current frequency and mode from the radio
    let frequency_hz = radio
        .get_frequency()
        .map_err(|e| e.to_string())?
        .as_hz();
    let mode = radio.get_mode().map_err(|e| e.to_string())?;

    let info = RadioInfo {
        port: port.clone(),
        baud_rate,
        frequency_hz,
        mode,
        connected: true,
    };

    // Store radio and port name in app state
    let mut radio_slot = state.radio.lock().map_err(|_| "Radio state corrupted".to_string())?;
    *radio_slot = Some(Box::new(radio));
    *state.serial_port_name.lock().map_err(|_| "Serial port state corrupted".to_string())? =
        Some(port.clone());

    Ok(info)
}

#[tauri::command]
pub fn disconnect_serial(state: State<AppState>) -> Result<(), String> {
    let mut radio_slot = state.radio.lock().map_err(|_| "Radio state corrupted".to_string())?;
    // Drop will auto-release PTT if transmitting
    *radio_slot = None;
    *state.serial_port_name.lock().map_err(|_| "Serial port state corrupted".to_string())? =
        None;
    Ok(())
}
