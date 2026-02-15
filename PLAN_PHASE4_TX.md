# Phase 4: PSK-31 TX Path — Implementation Plan

## Context

Phases 1-3 are complete. The app has audio capture, live waterfall, serial/CAT control, and PTT commands. Phase 4 builds the transmit path: type text, press TX, and the app encodes it as BPSK-31, plays the audio out, and controls PTT automatically.

**Decisions made:**
- Automatic PTT: TX button → PTT ON → 50ms delay → preamble → data → postamble → PTT OFF
- Audio-only OK: TX works without a connected radio (PTT skipped); enables testing with a speaker
- TX thread architecture mirrors Phase 3's audio input thread (dedicated thread, AtomicBool shutdown)

## Key Design: TX Pipeline

```
start_tx("CQ CQ CQ DE W1ABC") command
  → encode text to sample buffer (all upfront, not streaming)
    → Varicode::encode() each char → bit stream
    → prepend preamble (32 × '0' bits) + append postamble (32 × '0' bits)
    → for each bit: NCO carrier × RaisedCosineShaper envelope → 1536 samples
    → phase flips 180° on '0' bits (BPSK modulation)
  → spawn TX thread
    → PTT ON (if radio connected, ignore error if not)
    → 50ms delay
    → start CpalAudioOutput with callback pulling from sample buffer
    → emit tx-status events ("transmitting"/"complete"/"aborted"/"error")
    → wait for buffer drain or abort signal
    → PTT OFF
  → AppState stores: Arc<AtomicBool> (abort signal) + JoinHandle
```

**Why encode upfront?** At 31.25 baud with 1536 samples/symbol, a 256-char message is ~130K samples (~2.7s). Small enough to fit in memory. Simpler than streaming, and we can show a progress indicator.

## Build Sequence (13 steps)

### Rust Backend (Steps 1-6) — checkpoint: `cargo check && cargo test`

**Step 1: Implement PSK-31 encoder**
- `src-tauri/src/modem/encoder.rs` — replace stub:
  - `Psk31Encoder::new(sample_rate: u32, carrier_freq: f64)`
  - `encode(&mut self, text: &str) -> Vec<f32>` — full pipeline:
    1. Generate preamble: 32 idle bits (all zeros = 32 phase changes)
    2. For each char: `Varicode::encode()` → bits, append `00` separator
    3. Generate postamble: 32 idle bits
    4. For each bit: NCO sample × raised cosine envelope (1536 samples/symbol)
    5. BPSK: flip NCO phase by π on '0' bits (phase change = binary 0)
  - Returns complete audio buffer ready for playback
  - Constants: `SAMPLES_PER_SYMBOL = 48000 / 31.25 = 1536`, `PREAMBLE_BITS = 32`, `POSTAMBLE_BITS = 32`

**Step 2: Add CpalAudioOutput adapter**
- `src-tauri/src/adapters/cpal_audio.rs` — add alongside existing CpalAudioInput:
  - `CpalAudioOutput::new()`
  - `list_devices()` — enumerate output devices
  - `start(device_id, callback)` — build 48kHz mono f32 output stream
  - `stop()` — drop stream
  - `is_running()` — check flag
  - Same `!Send` constraint as input — lives on dedicated thread

**Step 3: Add TX fields to AppState**
- `src-tauri/src/state.rs`:
  - `tx_abort: Arc<AtomicBool>` — abort signal
  - `tx_thread: Mutex<Option<JoinHandle<()>>>` — for clean join

**Step 4: Implement TX commands**
- `src-tauri/src/commands/tx.rs` — CREATE new file:
  - `start_tx(app, state, text, device_id)`:
    1. Check not already transmitting
    2. Read carrier_freq from `state.config`
    3. Encode text → sample buffer via `Psk31Encoder`
    4. Spawn TX thread:
       - PTT ON (via `state.radio`, ignore if not connected)
       - Sleep 50ms
       - Start CpalAudioOutput, callback pulls from sample buffer
       - Emit `tx-status` events: `{ status: "transmitting", progress: 0.0..1.0 }`
       - Wait for buffer drain or abort
       - Stop audio output
       - PTT OFF (ignore if not connected)
       - Emit `tx-status: "complete"` or `"aborted"`
  - `stop_tx(state)`:
    1. Set abort flag
    2. Join TX thread
  - `get_tx_progress(state)`:
    1. Return current progress (0.0-1.0) from shared state
- `src-tauri/src/commands/mod.rs` — add `pub mod tx;`

**Step 5: Register TX commands**
- `src-tauri/src/lib.rs` — add `start_tx`, `stop_tx` to invoke_handler

**Step 6: Rust unit tests**
- `modem/encoder.rs`:
  - `test_encode_empty_text` — returns only preamble + postamble
  - `test_encode_single_char` — verify sample count: (preamble + char_bits + separator + postamble) × 1536
  - `test_encode_known_text_bit_count` — "CQ" → known number of symbols
  - `test_preamble_has_phase_changes` — first 32 symbols alternate phase
  - `test_samples_in_valid_range` — all samples between -1.0 and 1.0
- `adapters/cpal_audio.rs`:
  - `test_output_new_not_running`
  - `test_output_stop_idempotent`

### Frontend (Steps 7-10) — checkpoint: TX button triggers backend command

**Step 7: Add TX API wrappers**
- `src/services/backend-api.ts`:
  - `startTx(text: string, deviceId: string): Promise<void>`
  - `stopTx(): Promise<void>`

**Step 8: Create TX bridge service**
- CREATE `src/services/tx-bridge.ts`:
  - `listenTxStatus(callbacks)` — `listen("tx-status")` → update UI state
  - `stopTxBridge()` — unlisten

**Step 9: Update control-panel.ts**
- `src/components/control-panel.ts` — replace mock TX with real backend calls:
  - TX button → get selected audio output device → `startTx(text, deviceId)`
  - Abort button → `stopTx()`
  - Listen for `tx-status` events to manage UI state (TX indicator, PTT, button states)
  - On "complete"/"aborted" → reset to RX state
  - Disable TX input during transmission
  - If no audio output selected, show error (don't send)

**Step 10: Wire TX bridge in main.ts**
- Import and initialize `listenTxStatus()` in DOMContentLoaded

### Tests (Steps 11-13) — checkpoint: full green suite

**Step 11: E2E tests**
- `tests/e2e/app.spec.ts` — existing TX tests should still pass (they test the UI state machine which we're preserving)
- Verify: TX button disabled when input empty, triggers TX state, abort returns to RX

**Step 12: Integration test (encoder → decoder loopback)**
- `src-tauri/src/modem/encoder.rs` (or separate integration test):
  - `test_encode_decode_loopback` — encode "HELLO" → extract phase transitions → feed to VaricodeDecoder → verify "HELLO" comes back
  - This validates the full encode chain against the existing decoder

**Step 13: Final verification**
- `cargo test` — all Rust tests pass (26 existing + new)
- `npm test` — all 27 E2E tests pass
- Manual test: select audio output, type text, press TX, hear BPSK tone
- Mark Phase 4 complete in CLAUDE.md, commit

## Files Summary

| Action | File | What |
|--------|------|------|
| MODIFY | `src-tauri/src/modem/encoder.rs` | PSK-31 encoder: text → BPSK samples |
| MODIFY | `src-tauri/src/adapters/cpal_audio.rs` | Add CpalAudioOutput |
| MODIFY | `src-tauri/src/state.rs` | Add tx_abort + tx_thread |
| CREATE | `src-tauri/src/commands/tx.rs` | start_tx, stop_tx commands |
| MODIFY | `src-tauri/src/commands/mod.rs` | Add tx module |
| MODIFY | `src-tauri/src/lib.rs` | Register TX commands |
| MODIFY | `src/services/backend-api.ts` | Add TX wrappers |
| CREATE | `src/services/tx-bridge.ts` | TX status event listener |
| MODIFY | `src/components/control-panel.ts` | Real TX flow (replace mock) |
| MODIFY | `src/main.ts` | Wire TX bridge |

## Key Reuse

- `Varicode::encode()` + `bits_from_str()` — existing in `modem/varicode.rs`
- `Nco::new() / next()` — existing in `dsp/nco.rs`
- `RaisedCosineShaper::generate_envelope()` — existing in `dsp/raised_cosine.rs`
- `CpalAudioInput` pattern — mirror for output in `adapters/cpal_audio.rs`
- `with_radio()` helper — reuse in TX commands for PTT
- Audio thread pattern from Phase 3 — same AtomicBool + JoinHandle approach

## Verification

1. `cargo check` passes after Step 5
2. `cargo test` passes after Step 6 (encoder unit tests + loopback)
3. `npm test` — all E2E tests still green after Step 11
4. Manual: select audio output → type "CQ" → TX → hear BPSK-31 tone from speaker
5. With radio connected: PTT activates automatically during TX
