//! Audio port traits

use crate::domain::{AudioDeviceInfo, AudioSample, Psk31Result};

/// Trait for audio input (capture from microphone/radio)
pub trait AudioInput: Send + Sync {
    /// List available input devices
    fn list_devices(&self) -> Psk31Result<Vec<AudioDeviceInfo>>;

    /// Start capturing audio, calling the callback with samples
    fn start<F>(&mut self, device_id: &str, callback: F) -> Psk31Result<()>
    where
        F: FnMut(&[AudioSample]) + Send + 'static;

    /// Stop capturing
    fn stop(&mut self) -> Psk31Result<()>;

    /// Check if currently capturing
    fn is_running(&self) -> bool;
}

/// Trait for audio output (playback to speaker/radio)
pub trait AudioOutput: Send + Sync {
    /// List available output devices
    fn list_devices(&self) -> Psk31Result<Vec<AudioDeviceInfo>>;

    /// Start playback, calling the callback to get samples
    fn start<F>(&mut self, device_id: &str, callback: F) -> Psk31Result<()>
    where
        F: FnMut(&mut [AudioSample]) + Send + 'static;

    /// Stop playback
    fn stop(&mut self) -> Psk31Result<()>;

    /// Check if currently playing
    fn is_running(&self) -> bool;
}
