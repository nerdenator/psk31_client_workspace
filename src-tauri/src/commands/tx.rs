//! TX commands — start/stop PSK-31 transmission
//!
//! The TX pipeline:
//! 1. Encode text to BPSK-31 samples (upfront, not streaming)
//! 2. Spawn a TX thread that:
//!    - Activates PTT (if radio connected)
//!    - Plays the samples via CpalAudioOutput
//!    - Emits progress events to the frontend
//!    - Deactivates PTT when done

use serde::Serialize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::adapters::cpal_audio::CpalAudioOutput;
use crate::modem::encoder::Psk31Encoder;
use crate::ports::AudioOutput;
use crate::state::AppState;

/// Payload for `tx-status` events sent to the frontend
#[derive(Clone, Serialize)]
struct TxStatusPayload {
    status: String,
    progress: f32,
}

#[tauri::command]
pub fn start_tx(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    text: String,
    device_id: String,
) -> Result<(), String> {
    // Check if already transmitting
    if state.tx_thread.lock().unwrap().is_some() {
        return Err("Already transmitting".into());
    }

    // Read carrier frequency from config
    let carrier_freq = state.config.lock().unwrap().carrier_freq;
    let sample_rate = state.config.lock().unwrap().sample_rate;

    // Encode the entire message upfront
    let encoder = Psk31Encoder::new(sample_rate, carrier_freq);
    let samples = encoder.encode(&text);

    if samples.is_empty() {
        return Err("Nothing to transmit".into());
    }

    // Reset abort flag
    let abort = state.tx_abort.clone();
    abort.store(false, Ordering::SeqCst);

    // Try to activate PTT (ignore errors if no radio connected)
    let ptt_result = state
        .radio
        .lock()
        .ok()
        .and_then(|mut guard| guard.as_mut().map(|r| r.ptt_on()));

    if let Some(Err(e)) = ptt_result {
        log::warn!("PTT ON failed (continuing without PTT): {e}");
    }

    // Shared playback position for progress tracking
    let play_pos = Arc::new(AtomicUsize::new(0));
    let total_samples = samples.len();

    let handle = {
        let abort = abort.clone();
        let play_pos = play_pos.clone();

        thread::spawn(move || {
            run_tx_thread(app, abort, play_pos, samples, device_id, total_samples);
        })
    };

    state.tx_thread.lock().unwrap().replace(handle);

    Ok(())
}

#[tauri::command]
pub fn stop_tx(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Signal abort
    state.tx_abort.store(true, Ordering::SeqCst);

    // Join the thread
    if let Some(handle) = state.tx_thread.lock().unwrap().take() {
        handle.join().map_err(|_| "TX thread panicked".to_string())?;
    }

    // PTT OFF (ignore errors if no radio)
    let ptt_result = state
        .radio
        .lock()
        .ok()
        .and_then(|mut guard| guard.as_mut().map(|r| r.ptt_off()));

    if let Some(Err(e)) = ptt_result {
        log::warn!("PTT OFF failed: {e}");
    }

    Ok(())
}

/// TX thread: plays encoded samples through the audio output device.
fn run_tx_thread(
    app: AppHandle,
    abort: Arc<std::sync::atomic::AtomicBool>,
    play_pos: Arc<AtomicUsize>,
    samples: Vec<f32>,
    device_id: String,
    total_samples: usize,
) {
    // Brief delay after PTT to let the radio switch to TX
    thread::sleep(Duration::from_millis(50));

    let _ = app.emit(
        "tx-status",
        TxStatusPayload {
            status: "transmitting".into(),
            progress: 0.0,
        },
    );

    // Set up audio output with a callback that pulls from our sample buffer
    let mut audio_output = CpalAudioOutput::new();
    let samples_arc = Arc::new(samples);
    let samples_for_callback = samples_arc.clone();
    let pos_for_callback = play_pos.clone();
    let done_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let done_for_callback = done_flag.clone();

    let start_result = audio_output.start(
        &device_id,
        Box::new(move |output_buf: &mut [f32]| {
            let current_pos = pos_for_callback.load(Ordering::Relaxed);
            let remaining = total_samples.saturating_sub(current_pos);

            if remaining == 0 {
                // Fill with silence — we're done
                for sample in output_buf.iter_mut() {
                    *sample = 0.0;
                }
                done_for_callback.store(true, Ordering::SeqCst);
                return;
            }

            let copy_len = output_buf.len().min(remaining);
            output_buf[..copy_len]
                .copy_from_slice(&samples_for_callback[current_pos..current_pos + copy_len]);

            // Fill any remaining buffer with silence
            for sample in output_buf[copy_len..].iter_mut() {
                *sample = 0.0;
            }

            pos_for_callback.store(current_pos + copy_len, Ordering::Relaxed);
        }),
    );

    if let Err(e) = start_result {
        log::error!("Failed to start audio output: {e}");
        let _ = app.emit(
            "tx-status",
            TxStatusPayload {
                status: format!("error: {e}"),
                progress: 0.0,
            },
        );
        return;
    }

    // Wait for playback to finish or abort
    loop {
        if abort.load(Ordering::SeqCst) {
            let _ = audio_output.stop();
            let _ = app.emit(
                "tx-status",
                TxStatusPayload {
                    status: "aborted".into(),
                    progress: play_pos.load(Ordering::Relaxed) as f32 / total_samples as f32,
                },
            );

            // PTT OFF from the calling context (stop_tx), not here
            return;
        }

        if done_flag.load(Ordering::SeqCst) {
            // Give a small buffer for the audio device to actually play the final samples
            thread::sleep(Duration::from_millis(100));
            let _ = audio_output.stop();
            let _ = app.emit(
                "tx-status",
                TxStatusPayload {
                    status: "complete".into(),
                    progress: 1.0,
                },
            );

            // PTT OFF — we need access to the radio, but we're on the TX thread.
            // The frontend will call stop_tx or handle PTT via the status event.
            return;
        }

        // Emit progress
        let pos = play_pos.load(Ordering::Relaxed);
        let progress = pos as f32 / total_samples as f32;
        let _ = app.emit(
            "tx-status",
            TxStatusPayload {
                status: "transmitting".into(),
                progress,
            },
        );

        thread::sleep(Duration::from_millis(50));
    }
}
