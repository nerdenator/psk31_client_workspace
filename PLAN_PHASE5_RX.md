# Phase 5: PSK-31 RX Path — Implementation Plan

## Overview

Implement the full PSK-31 receive path: audio samples → DSP chain → decoded text displayed in the frontend. This completes the core functionality of the application.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| RX threading | Same audio thread as FFT | Simpler, shared ring buffer, decoder runs alongside FFT in existing DSP loop |
| RX lifecycle | Explicit `start_rx` / `stop_rx` commands | Matches plan API spec; allows waterfall-only mode without decoder overhead |
| Bandpass filter | Yes, ~100 Hz BW before Costas Loop | Better selectivity against adjacent signals; standard practice |
| Phase ambiguity | Differential decoding + fallback inversion | Differential as primary (detect phase changes); invert bit sense after 100+ bits with no valid Varicode chars |

## Architecture

### RX Pipeline (per sample)

```
Audio In (48kHz mono f32)
  → [Ring Buffer] → DSP Loop (existing audio thread)
  → AGC (normalize amplitude)
  → Bandpass Filter (FIR, ~100 Hz BW centered on carrier_freq)
  → Costas Loop (BPSK carrier tracking → baseband I)
  → Differential Bit Detection (compare consecutive symbols)
  → Clock Recovery (Mueller-Muller, 1536 samples/symbol)
  → Varicode Decode → emit "rx-text" event → Frontend RX Display

Parallel (existing): Audio In → FFT → emit "fft-data" → Waterfall
```

### Differential Decoding

Instead of making bit decisions on absolute phase (`I > 0 → 1`), we compare consecutive symbols:
- Same sign as previous symbol → bit 1 (no phase change)
- Opposite sign → bit 0 (phase change)

This resolves the Costas Loop's 180° ambiguity because we only care about *changes*, not absolute phase.

**Fallback inversion**: Track consecutive bits without valid Varicode output. After 100+ bits with no decoded character, invert the bit sense (the Costas Loop may have locked 180° off, and differential detection was still confused by noise during acquisition).

### Threading Model

The existing audio thread (`run_audio_thread` in `commands/audio.rs`) already:
1. Owns the ring buffer consumer
2. Runs a DSP loop pulling samples
3. Computes FFT and emits `fft-data` events

We extend this loop to also run the decoder when `rx_running` is true:
```
while running {
    drain ring buffer → sample_buf

    // Existing: FFT processing
    if sample_buf.len() >= fft_size { compute FFT, emit fft-data }

    // NEW: RX decoding (when enabled)
    if rx_running {
        for each new sample:
            if let Some(char) = decoder.process(sample) {
                emit "rx-text" event
            }
    }
}
```

### State Changes

**AppState** additions:
```rust
pub rx_running: Arc<AtomicBool>,     // Flag to enable/disable decoder
pub rx_carrier_freq: Arc<Mutex<f64>>, // Carrier freq (updated by click-to-tune)
```

No need for a separate RX thread handle since decoding runs inside the existing audio thread.

---

## Implementation Steps

### Step 1: Unit Tests for Existing DSP Components

Add tests for the three DSP blocks that exist but are untested.

**File: `src-tauri/src/dsp/agc.rs`** — Add tests:
- Loud signal → gain decreases toward target
- Quiet signal → gain increases toward target
- Gain stays within [min_gain, max_gain]
- Output clamped to [-1.0, 1.0]

**File: `src-tauri/src/dsp/filter.rs`** — Add tests:
- Lowpass impulse response is symmetric
- DC signal passes through unchanged
- High-frequency signal is attenuated
- Add `bandpass()` factory method (lowpass shifted to carrier freq, or use frequency-shifted lowpass pair)

**File: `src-tauri/src/dsp/costas_loop.rs`** — Add tests:
- Clean BPSK at exact carrier freq → locks and demodulates
- BPSK with small frequency offset (~5 Hz) → still locks
- Verify phase tracking (output sign matches input phase)

**File: `src-tauri/src/dsp/clock_recovery.rs`** — Add tests:
- Fixed-interval symbols → outputs at correct rate
- Symbols at correct positions → reasonable timing convergence
- Bug fix: Mueller-Muller error formula on line 43 looks wrong — should be `last_symbol * sample - last_sample * symbol` but both terms use `sample` not a stored previous. Review and fix during testing.

### Step 2: Bandpass Filter

**File: `src-tauri/src/dsp/filter.rs`** — Add `bandpass()` factory:

Approach: Create a bandpass by frequency-shifting a lowpass prototype.
```rust
pub fn bandpass(center_freq: f32, bandwidth: f32, sample_rate: f32, num_taps: usize) -> Self {
    // 1. Design lowpass with cutoff = bandwidth/2
    // 2. Multiply coefficients by cos(2π * center_freq/sample_rate * n) to shift up
    // 3. Apply Hanning window, normalize
}
```

Parameters for PSK-31 RX:
- `center_freq`: carrier_freq (e.g., 1000 Hz)
- `bandwidth`: ~100 Hz (captures ±50 Hz around carrier)
- `sample_rate`: 48000 Hz
- `num_taps`: 127 (per PLAN.md spec)

### Step 3: PSK-31 Decoder

**File: `src-tauri/src/modem/decoder.rs`** — Full implementation:

```rust
pub struct Psk31Decoder {
    agc: Agc,
    bandpass: FirFilter,
    costas_loop: CostasLoop,
    clock_recovery: ClockRecovery,
    varicode_decoder: VaricodeDecoder,

    // Differential decoding state
    last_symbol: f32,

    // Phase ambiguity fallback
    bits_without_char: usize,
    invert_bits: bool,

    sample_rate: u32,
    carrier_freq: f64,
}

impl Psk31Decoder {
    pub fn new(carrier_freq: f64, sample_rate: u32) -> Self { ... }

    /// Process a single audio sample. Returns Some(char) when a character is decoded.
    pub fn process(&mut self, sample: f32) -> Option<char> {
        // 1. AGC
        let normalized = self.agc.process(sample);

        // 2. Bandpass filter (centered on carrier)
        let filtered = self.bandpass.process(normalized);

        // 3. Costas Loop (carrier tracking → baseband I value)
        let baseband = self.costas_loop.process(filtered);

        // 4. Clock Recovery (outputs symbol at decision points)
        if let Some(symbol) = self.clock_recovery.process(baseband) {
            // 5. Differential bit detection
            let raw_bit = (symbol > 0.0) == (self.last_symbol > 0.0);
            self.last_symbol = symbol;

            // 6. Apply inversion if phase ambiguity detected
            let bit = if self.invert_bits { !raw_bit } else { raw_bit };

            // 7. Varicode decode
            self.bits_without_char += 1;
            if let Some(ch) = self.varicode_decoder.push_bit(bit) {
                self.bits_without_char = 0;
                return Some(ch);
            }

            // 8. Phase ambiguity fallback
            if self.bits_without_char > 100 {
                self.invert_bits = !self.invert_bits;
                self.bits_without_char = 0;
                self.varicode_decoder.reset();
            }
        }

        None
    }

    /// Update the carrier frequency (e.g., from waterfall click-to-tune)
    pub fn set_carrier_freq(&mut self, freq: f64) {
        self.carrier_freq = freq;
        self.costas_loop.set_frequency(freq);
        // Rebuild bandpass filter centered on new freq
        self.bandpass = FirFilter::bandpass(freq as f32, 100.0, self.sample_rate as f32, 127);
        self.reset();
    }

    /// Reset decoder state (called on retune)
    pub fn reset(&mut self) {
        self.agc.reset();
        self.bandpass.reset();
        self.costas_loop.reset();
        self.clock_recovery.reset();
        self.varicode_decoder.reset();
        self.last_symbol = 0.0;
        self.bits_without_char = 0;
        self.invert_bits = false;
    }
}
```

**Tests for decoder.rs:**
- Synthetic clean BPSK → correct text decoded
- Loopback: `encoder.encode("HELLO")` → `decoder.process()` each sample → "HELLO"
- Loopback with small frequency offset (±5 Hz)
- Phase ambiguity: start with inverted phase → still decodes after fallback kicks in

### Step 4: Integration into Audio Thread

**File: `src-tauri/src/commands/audio.rs`** — Modify `run_audio_thread`:

1. Accept `rx_running: Arc<AtomicBool>` and `rx_carrier_freq: Arc<Mutex<f64>>` params
2. Create `Psk31Decoder` inside the thread
3. In DSP loop, after draining ring buffer:
   - Track samples consumed for FFT vs decoded (use separate index)
   - When `rx_running` is true, feed each sample to `decoder.process()`
   - On `Some(char)`, emit `"rx-text"` event with the character
   - Periodically check if carrier_freq changed and call `decoder.set_carrier_freq()`

**File: `src-tauri/src/commands/audio.rs`** — Add new commands:
```rust
#[tauri::command]
pub fn start_rx(state: tauri::State<'_, AppState>) -> Result<(), String> {
    if !state.audio_running.load(Ordering::SeqCst) {
        return Err("Audio stream not running. Start audio first.".into());
    }
    state.rx_running.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn stop_rx(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.rx_running.store(false, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn set_carrier_frequency(
    state: tauri::State<'_, AppState>,
    freq_hz: f64,
) -> Result<(), String> {
    if freq_hz < 500.0 || freq_hz > 2500.0 {
        return Err("Carrier frequency must be between 500-2500 Hz".into());
    }
    *state.rx_carrier_freq.lock().unwrap() = freq_hz;
    // Also update config for TX consistency
    state.config.lock().unwrap().carrier_freq = freq_hz;
    Ok(())
}
```

**File: `src-tauri/src/state.rs`** — Add `rx_running` and `rx_carrier_freq` fields.

**File: `src-tauri/src/lib.rs`** — Register new commands: `start_rx`, `stop_rx`, `set_carrier_frequency`.

### Step 5: Frontend Integration

**File: `src/services/backend-api.ts`** — Add wrappers:
```typescript
export async function startRx(): Promise<void> {
  return invoke('start_rx');
}
export async function stopRx(): Promise<void> {
  return invoke('stop_rx');
}
export async function setCarrierFrequency(freqHz: number): Promise<void> {
  return invoke('set_carrier_frequency', { freq_hz: freqHz });
}
```

**File: `src/services/rx-bridge.ts`** — New file (mirrors audio-bridge.ts pattern):
```typescript
// Listen for "rx-text" events from backend, call callback with decoded char
export async function startRxBridge(onChar: (ch: string) => void): Promise<void> { ... }
export async function stopRxBridge(): Promise<void> { ... }
```

**File: `src/components/rx-display.ts`** — Enhance:
- Add `appendText(text: string)` function that appends to `#rx-content`
- Auto-scroll to bottom on new text
- Wire up RX bridge: `startRxBridge((ch) => appendText(ch))`
- Keep existing Clear button behavior

**File: `src/components/waterfall-controls.ts`** — Add backend call:
- On click-to-tune, also call `setCarrierFrequency(freq)` to update the decoder's NCO

**File: `src/main.ts`** — Wire up RX bridge in initialization.

### Step 6: Integration Tests

**File: `src-tauri/tests/rx_loopback.rs`** — Full loopback tests:
- `encoder.encode("TEST CQ") → decoder.process() each sample → "TEST CQ"`
- Loopback with simulated noise (add random samples scaled by SNR)
- Loopback with ±5 Hz frequency offset (encode at 1000 Hz, decode at 1005 Hz)

### Step 7: E2E Tests (Playwright)

**File: `tests/e2e/rx-display.spec.ts`** — New test file:
- RX display shows decoded text (mock `rx-text` events)
- RX display scrolls with new text
- Clear button empties RX display
- Click waterfall calls `set_carrier_frequency` command (mock + verify)

### Step 8: Visual Regression Updates

Update existing Playwright visual snapshots to account for any UI changes.

---

## Files Modified (Summary)

### New Files
| File | Purpose |
|------|---------|
| `src/services/rx-bridge.ts` | Frontend RX event listener bridge |
| `tests/e2e/rx-display.spec.ts` | E2E tests for RX display |
| `src-tauri/tests/rx_loopback.rs` | Integration loopback tests |

### Modified Files
| File | Changes |
|------|---------|
| `src-tauri/src/modem/decoder.rs` | Full decoder implementation |
| `src-tauri/src/dsp/filter.rs` | Add `bandpass()` factory + tests |
| `src-tauri/src/dsp/agc.rs` | Add tests |
| `src-tauri/src/dsp/costas_loop.rs` | Add tests |
| `src-tauri/src/dsp/clock_recovery.rs` | Fix M&M formula bug + add tests |
| `src-tauri/src/commands/audio.rs` | Add decoder to DSP loop, add `start_rx`/`stop_rx`/`set_carrier_frequency` commands |
| `src-tauri/src/state.rs` | Add `rx_running`, `rx_carrier_freq` |
| `src-tauri/src/lib.rs` | Register new commands |
| `src/services/backend-api.ts` | Add RX + carrier freq wrappers |
| `src/components/rx-display.ts` | Enhance with text append + auto-scroll |
| `src/components/waterfall-controls.ts` | Add `setCarrierFrequency()` call on click |
| `src/main.ts` | Wire up RX bridge |
| `CLAUDE.md` | Mark Phase 5 complete |

---

## Testing Strategy

| Test Type | Count (est.) | Coverage |
|-----------|-------------|----------|
| Rust unit (new) | ~15 | AGC, filter, Costas Loop, Clock Recovery, decoder |
| Rust integration | ~3 | TX→RX loopback (clean, noisy, freq offset) |
| Playwright E2E | ~5 | RX display, clear, scroll, click-to-tune |
| Visual regression | Update snapshots | |

---

## Build Sequence

Implement in this order to maintain passing tests at each step:

1. **DSP unit tests** (agc, filter, costas_loop, clock_recovery) — validates existing code, catches bugs early
2. **Bandpass filter** factory method — needed by decoder
3. **Decoder** (`modem/decoder.rs`) — pure logic, no I/O deps, testable in isolation
4. **Loopback integration tests** — validates decoder against encoder output
5. **Backend wiring** (state, commands, audio thread) — connects decoder to real audio
6. **Frontend** (rx-bridge, rx-display, waterfall click, main.ts) — UI integration
7. **E2E tests** — validates full stack with mocked backend
8. **Visual regression** — snapshot updates
