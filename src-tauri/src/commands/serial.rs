//! Serial commands - stub for Phase 1

use crate::domain::SerialPortInfo;

#[tauri::command]
pub fn list_serial_ports() -> Vec<SerialPortInfo> {
    // TODO: Implement with serialport in Phase 2
    vec![]
}
