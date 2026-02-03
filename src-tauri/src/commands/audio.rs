//! Audio commands - stub for Phase 1

use crate::domain::AudioDeviceInfo;

#[tauri::command]
pub fn list_audio_devices() -> Vec<AudioDeviceInfo> {
    // TODO: Implement with cpal in Phase 3
    vec![]
}
