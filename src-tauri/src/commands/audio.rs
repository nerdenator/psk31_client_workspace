//! Audio commands — list devices, start/stop FFT streaming
//!
//! Architecture: `cpal::Stream` is `!Send` (like a Python object bound to one thread).
//! So we can't store it in AppState behind a Mutex. Instead, `start_audio_stream`
//! spawns a dedicated audio thread that owns the CpalAudioInput and ring buffer.
//! AppState only holds an AtomicBool shutdown flag and the thread's JoinHandle.

use ringbuf::HeapRb;
use ringbuf::traits::{Consumer, Producer, Split};
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::adapters::cpal_audio::CpalAudioInput;
use crate::domain::AudioDeviceInfo;
use crate::dsp::fft::FftProcessor;
use crate::ports::AudioInput;
use crate::state::AppState;

/// Payload for the `fft-data` event sent to the frontend
#[derive(Clone, Serialize)]
struct FftPayload {
    magnitudes: Vec<f32>,
}

/// Payload for the `audio-status` event
#[derive(Clone, Serialize)]
struct AudioStatusPayload {
    status: String,
}

#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<AudioDeviceInfo>, String> {
    let input = CpalAudioInput::new();
    input.list_devices().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_audio_stream(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    device_id: String,
) -> Result<(), String> {
    // Check if already running
    if state.audio_running.load(Ordering::SeqCst) {
        return Err("Audio stream already running".into());
    }

    let running = state.audio_running.clone();
    running.store(true, Ordering::SeqCst);

    let handle = thread::spawn(move || {
        run_audio_thread(app, running, device_id);
    });

    state
        .audio_thread
        .lock()
        .unwrap()
        .replace(handle);

    Ok(())
}

#[tauri::command]
pub fn stop_audio_stream(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Signal the thread to stop
    state.audio_running.store(false, Ordering::SeqCst);

    // Join the thread (wait for clean shutdown)
    if let Some(handle) = state.audio_thread.lock().unwrap().take() {
        handle.join().map_err(|_| "Audio thread panicked".to_string())?;
    }

    Ok(())
}

/// The main audio processing loop, runs on its own thread.
///
/// Flow: cpal callback → ring buffer → DSP loop → FFT → emit event
fn run_audio_thread(app: AppHandle, running: std::sync::Arc<std::sync::atomic::AtomicBool>, device_id: String) {
    // Emit status
    let _ = app.emit("audio-status", AudioStatusPayload { status: "running".into() });

    // Create ring buffer — 8192 samples gives ~170ms buffer at 48kHz
    // Think of it like a Python `collections.deque(maxlen=8192)` but lock-free
    let rb = HeapRb::<f32>::new(8192);
    let (mut producer, mut consumer) = rb.split();

    // Create audio input and start capture
    let mut audio_input = CpalAudioInput::new();
    let capture_result = audio_input.start(
        &device_id,
        Box::new(move |samples: &[f32]| {
            // Push samples into ring buffer, dropping oldest if full (never blocks)
            for &sample in samples {
                let _ = producer.try_push(sample);
            }
        }),
    );

    if let Err(e) = capture_result {
        log::error!("Failed to start audio capture: {e}");
        running.store(false, Ordering::SeqCst);
        let _ = app.emit("audio-status", AudioStatusPayload {
            status: format!("error: {e}"),
        });
        return;
    }

    // DSP loop: pull samples, compute FFT, emit to frontend
    let fft_size = 4096;
    let hop_size = 2048; // 50% overlap for smooth waterfall scrolling
    let mut fft = FftProcessor::new(fft_size);
    let mut sample_buf: Vec<f32> = Vec::with_capacity(fft_size);

    while running.load(Ordering::SeqCst) {
        // Drain available samples from ring buffer
        while let Some(sample) = consumer.try_pop() {
            sample_buf.push(sample);
        }

        // When we have enough samples, compute FFT with 50% overlap
        while sample_buf.len() >= fft_size {
            let magnitudes = fft.compute(&sample_buf[..fft_size]);
            let _ = app.emit("fft-data", FftPayload { magnitudes });

            // Advance by hop_size (keep the overlap portion)
            sample_buf.drain(..hop_size);
        }

        // Sleep to avoid busy-waiting (~5ms = well within 42ms frame budget)
        thread::sleep(Duration::from_millis(5));
    }

    // Clean shutdown
    let _ = audio_input.stop();
    let _ = app.emit("audio-status", AudioStatusPayload { status: "stopped".into() });
}
