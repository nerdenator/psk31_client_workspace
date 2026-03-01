//! Audio port traits

use crate::domain::{AudioDeviceInfo, AudioSample, Psk31Result};

/// Trait for audio input (capture from microphone/radio)
///
/// Note: No `Send` bound — cpal::Stream is !Send, so implementations
/// must live on the thread that created them (the dedicated audio thread).
pub trait AudioInput {
    /// List available input devices
    fn list_devices(&self) -> Psk31Result<Vec<AudioDeviceInfo>>;

    /// Start capturing audio, calling the callback with samples
    fn start(
        &mut self,
        device_id: &str,
        callback: Box<dyn FnMut(&[AudioSample]) + Send + 'static>,
    ) -> Psk31Result<()>;

    /// Stop capturing
    fn stop(&mut self) -> Psk31Result<()>;

    /// Check if currently capturing
    fn is_running(&self) -> bool;
}

/// Trait for audio output (playback to speaker/radio)
pub trait AudioOutput {
    /// List available output devices
    fn list_devices(&self) -> Psk31Result<Vec<AudioDeviceInfo>>;

    /// Start playback, calling the callback to get samples
    fn start(
        &mut self,
        device_id: &str,
        callback: Box<dyn FnMut(&mut [AudioSample]) + Send + 'static>,
    ) -> Psk31Result<()>;

    /// Stop playback
    fn stop(&mut self) -> Psk31Result<()>;

    /// Check if currently playing
    fn is_running(&self) -> bool;
}
