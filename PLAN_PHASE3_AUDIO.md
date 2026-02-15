# Phase 3: Audio Subsystem + Waterfall — Implementation Plan

## Status: Ready for Review

## Context

Phase 2 is COMPLETE. The app has:
- Full hexagonal architecture with serial/CAT communication
- FT-991A adapter with frequency, mode, and PTT control
- 23 Playwright E2E tests + 22 Rust unit tests (1 pre-existing failure)
- Existing DSP: `FftProcessor` (4096-point FFT, Hanning window, dB output), `Nco`, `FirFilter`
- Existing port traits: `AudioInput`, `AudioOutput` (with callback-based streaming)
- Existing frontend: `WaterfallDisplay` class (currently renders simulated data)
- Existing audio command stub: `list_audio_devices()` returning empty vec
- `cpal` crate v0.15 already in Cargo.toml
- `ringbuf` crate v0.4 and `crossbeam-channel` v0.5 already in Cargo.toml

## Goal

Replace simulated waterfall data with live FFT from the radio's USB audio input. The user selects an audio input device, starts the audio stream, and the waterfall displays the real spectrum.

---

## Decisions Needed

1. **Streaming mechanism**: Tauri 2.x `AppHandle::emit()` events vs Tauri `Channel<T>` (ipc::Channel for streaming from commands). Events are simpler; Channels are more efficient for high-throughput data.
2. **Audio output**: Phase 3 only needs audio *input* for the waterfall. Audio output is needed for TX (Phase 4). Should we implement both traits now, or just input?
3. **Device persistence**: Should the selected audio device be saved to the configuration profile, or just remembered for the session?
4. **FFT update rate**: Plan says ~23 fps. Is this acceptable, or should it be configurable?
5. **Audio device hot-plug**: Should we handle devices being connected/disconnected while the app runs, or just enumerate on startup?

---

## Threading Model

Per PLAN.md, the architecture is:

```
cpal audio callback thread
  → copies samples to lock-free ring buffer (zero allocations, zero locks)

DSP/FFT thread (spawned on audio start)
  → pulls from ring buffer
  → runs FftProcessor (4096-point, Hanning window)
  → emits FFT magnitude data to frontend via Tauri event
  → ~23 fps (48000 Hz / 4096 samples ≈ 11.7 updates/sec with 50% overlap → ~23 fps)
```

The ring buffer (`ringbuf` crate, lock-free SPSC) decouples the cpal callback from DSP processing. The cpal callback must never block or allocate — it only pushes samples into the ring buffer.

---

## Files to Create (3)

### 1. `src-tauri/src/adapters/cpal_audio.rs` (~200 lines)
- `CpalAudioInput` — implements `AudioInput` trait
- Uses `cpal` to enumerate input devices and start capture streams
- Captures mono f32 at 48kHz
- Pushes samples into a `ringbuf::HeapRb` ring buffer
- Spawns a DSP thread that pulls from the ring buffer, runs FFT, and sends results via a `crossbeam_channel::Sender`

**Key implementation details:**
- Device enumeration: `cpal::default_host().input_devices()`
- Stream config: mono, f32, 48000 Hz
- Ring buffer size: 8192 samples (~170ms at 48kHz)
- FFT overlap: 50% (advance 2048 samples per FFT frame)
- DSP thread reads 2048 new samples, maintains a 4096-sample window

### 2. `src/services/audio-bridge.ts` (~60 lines)
- `setupAudioBridge(waterfall: WaterfallDisplay)`: listens for `fft-data` Tauri events
- Receives `Vec<f32>` (FFT magnitudes in dB) from backend
- Passes to `WaterfallDisplay.drawSpectrum(data)` (new method replacing simulated `drawFrame`)

### 3. `tests/e2e/audio.spec.ts` (~120 lines)
5 tests:
1. Audio input dropdown populates (mocked device list)
2. Audio output dropdown populates (mocked device list)
3. Start audio stream updates waterfall (mocked FFT event)
4. Audio device selection enables connect
5. Stop audio stream stops waterfall updates

---

## Files to Modify (11)

### 4. `src-tauri/src/adapters/mod.rs`
Add: `pub mod cpal_audio;`

### 5. `src-tauri/src/ports/audio.rs`
Potential changes:
- Simplify `AudioInput` trait — the current callback-based design requires `FnMut(&[AudioSample]) + Send + 'static` which is complex. Consider whether the trait should instead expose start/stop and let the adapter handle the internal threading.
- Remove `Sync` bound if not needed (behind Mutex like RadioControl)

### 6. `src-tauri/src/state.rs`
Add fields:
```rust
pub audio_input: Mutex<Option<Box<dyn AudioInput>>>,
pub fft_sender: Mutex<Option<crossbeam_channel::Sender<Vec<f32>>>>,
```

### 7. `src-tauri/src/commands/audio.rs`
Replace stub with:
- `list_audio_devices()` — calls `CpalAudioInput::list_devices()`
- `start_audio_stream(state, app, device_id)` — create CpalAudioInput, start capture, spawn FFT thread, emit events
- `stop_audio_stream(state)` — stop capture, join FFT thread
- `select_input_device(state, device_id)` — store selection (Phase 4 will use for TX too)

### 8. `src-tauri/src/lib.rs`
Register new commands:
```rust
commands::audio::start_audio_stream,
commands::audio::stop_audio_stream,
```

### 9. `src-tauri/src/domain/types.rs`
Add `AudioStreamStatus` struct if needed for frontend state.

### 10. `src/components/waterfall.ts`
- Add `drawSpectrum(data: Float32Array)` method — renders real FFT data instead of simulated signals
- Keep `drawFrame()` as fallback when no audio stream is active (simulated mode)
- Add `setMode(mode: 'simulated' | 'live')` to switch between modes
- Normalize dB values to 0-255 color range (noise floor calibration)

### 11. `src/types/index.ts`
Add:
```typescript
export interface AudioStreamEvent {
  magnitudes: number[];  // FFT dB values, length = fft_size/2
}
```

### 12. `src/services/backend-api.ts`
Add wrappers:
```typescript
startAudioStream(deviceId: string): Promise<void>
stopAudioStream(): Promise<void>
```

### 13. `src/main.ts`
- Import and initialize `setupAudioBridge()`, passing the waterfall instance
- Wire audio device dropdowns to backend

### 14. `index.html`
- Update Audio In/Out dropdowns: remove hardcoded options, keep placeholders
- Add Start/Stop audio button (or auto-start on device selection)
- Update Audio In/Out connection status to start as disconnected

---

## Build Sequence

### Step 1: Audio Port + Adapter (no UI change)
- [ ] Review/update `ports/audio.rs` — simplify trait if needed
- [ ] Create `adapters/cpal_audio.rs` — device enumeration + capture
- [ ] Update `adapters/mod.rs`
- [ ] Verify: `cargo check`

### Step 2: FFT Pipeline
- [ ] Wire ring buffer between cpal callback and FFT thread
- [ ] FFT thread: pull samples → `FftProcessor::compute()` → send via channel
- [ ] Unit test: synthetic sine wave through pipeline produces correct FFT peak
- [ ] Verify: `cargo test`

### Step 3: State + Commands
- [ ] Update `state.rs` — add audio fields
- [ ] Update `commands/audio.rs` — implement list/start/stop
- [ ] Update `lib.rs` — register new commands
- [ ] Verify: `cargo check`

### Step 4: Frontend — Waterfall Integration
- [ ] Update `waterfall.ts` — add `drawSpectrum()` for real FFT data
- [ ] Create `services/audio-bridge.ts` — listen for `fft-data` events
- [ ] Update `types/index.ts` — add AudioStreamEvent
- [ ] Update `backend-api.ts` — add wrappers
- [ ] Update `main.ts` — wire audio bridge to waterfall
- [ ] Verify: TypeScript compiles

### Step 5: Frontend — Device Selection UI
- [ ] Update `index.html` — remove hardcoded audio options, update status dots
- [ ] Wire audio dropdowns to `list_audio_devices()` (similar to serial panel)
- [ ] Auto-start audio stream on device selection (or add start button)
- [ ] Verify: UI works in dev mode

### Step 6: E2E Tests
- [ ] Create `tests/e2e/audio.spec.ts`
- [ ] Verify: `npm test` — all existing + new tests pass

### Step 7: Finalize
- [ ] Run full test suite: `cargo test && npm test`
- [ ] Mark Phase 3 complete in `CLAUDE.md`
- [ ] Commit

---

## Key Constraints

- **23 existing E2E tests must still pass** — DOM structure changes must be additive
- **5 serial E2E tests must still pass** — no regression on serial panel
- **cpal audio callback must NEVER block** — only push to ring buffer
- **Ring buffer overflow**: if DSP can't keep up, drop oldest samples (acceptable for waterfall)
- **48kHz sample rate** — native FT-991A USB audio rate
- **4096-point FFT** — ~11.7 Hz/bin resolution, good enough for PSK-31's ~60 Hz bandwidth
- **Thread cleanup**: audio stream + FFT thread must be properly joined on stop/disconnect

## FFT Data Flow

```
FT-991A USB Audio (48kHz mono)
  → cpal input stream callback
  → ringbuf::HeapRb (8192 samples, lock-free SPSC)
  → DSP thread (spawned on start_audio_stream)
    → reads 2048 new samples (50% overlap with previous frame)
    → FftProcessor::compute(4096 samples) → Vec<f32> (dB magnitudes)
    → AppHandle::emit("fft-data", magnitudes)
  → Frontend: listen("fft-data")
    → audio-bridge.ts → WaterfallDisplay::drawSpectrum(data)
    → Canvas pixel rendering via color map LUT
```

## dB Normalization

The FFT outputs raw dB values (roughly -100 to 0 dB). The waterfall needs 0-255 for the color map. Strategy:

```
normalized = clamp((db - noise_floor) / dynamic_range * 255, 0, 255)
```

Where:
- `noise_floor` ≈ -80 dB (auto-calibrated from minimum of recent frames)
- `dynamic_range` ≈ 60 dB (adjustable)

This auto-calibration ensures the waterfall adapts to different input levels without manual gain adjustment.

## Error Handling

```
cpal::BuildStreamError / cpal::PlayStreamError
  → Psk31Error::Audio(String)
  → commands return Result<T, String>
  → frontend shows error in Audio In status indicator
```

If the audio device disconnects mid-stream:
- cpal fires an error callback
- Set status to disconnected
- Emit `audio-status` event to frontend
- Waterfall falls back to simulated mode
