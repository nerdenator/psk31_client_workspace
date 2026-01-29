# PSK-31 Desktop Client — Implementation Plan

## Overview

Cross-platform desktop application (macOS, Windows, Linux) for sending and receiving PSK-31 ham radio messages via USB-connected transceiver.

## Technology Stack

- **Framework**: Tauri 2.x (Rust backend + web frontend)
- **Frontend**: Vanilla TypeScript + Vite
- **Audio I/O**: `cpal` crate (cross-platform audio)
- **Serial/CAT**: `serialport` crate
- **DSP**: `rustfft`, custom BPSK modem in Rust
- **Sample rate**: 48000 Hz (native FT-991A USB audio rate)

## Target Radio (v1)

**Yaesu FT-991A** — connects via USB, presents USB audio codec + virtual serial port (CP210x).

CAT protocol: ASCII commands terminated with `;` at 38400 baud 8N1.
- PTT on/off: `TX1;` / `TX0;`
- Read frequency: `FA;` → `FA00014070000;`
- Set frequency: `FA00014070000;`
- Read mode: `MD0;` → `MD0C;` (DATA-USB)

**Future expansion**: SignaLink USB interfaces, generic USB serial + separate sound card, other CAT protocols (Icom CI-V, Kenwood).

## Features

### v1 (this plan)
- PSK-31 BPSK modulation/demodulation
- Varicode encoding/decoding
- Spectral waterfall display (click-to-tune)
- TX text input with transmit/abort controls
- RX decoded text display
- Audio device enumeration and selection
- Serial port selection + CAT connection
- PTT control via Yaesu CAT

### Future
- QSO logging (callsign, RST, time, frequency)
- TX macros (CQ, 73, contest exchanges)
- Additional radio support

---

## Project Structure

```
psk31_client_workspace/
├── index.html
├── package.json
├── tsconfig.json
├── vite.config.ts
├── src/                              # Frontend (TypeScript)
│   ├── main.ts                       # App entry, bootstraps UI
│   ├── styles.css
│   ├── components/
│   │   ├── waterfall.ts              # Canvas waterfall spectrogram
│   │   ├── rx-display.ts             # Decoded text display
│   │   ├── tx-input.ts               # Transmit text input
│   │   ├── control-panel.ts          # Device selectors, PTT, freq display
│   │   └── status-bar.ts             # Connection/signal status
│   ├── services/
│   │   ├── backend-api.ts            # Typed invoke() wrappers
│   │   ├── event-handlers.ts         # Tauri event listeners
│   │   └── audio-bridge.ts           # FFT channel handler
│   ├── utils/
│   │   ├── color-map.ts              # dB → RGB for waterfall
│   │   └── formatter.ts              # Frequency formatting
│   └── types/
│       └── index.ts                  # Shared interfaces
├── src-tauri/                        # Backend (Rust)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   └── src/
│       ├── main.rs
│       ├── lib.rs                    # Tauri app builder, command registration
│       ├── commands/                 # Tauri command handlers
│       │   ├── mod.rs
│       │   ├── audio.rs              # Audio device commands
│       │   ├── serial.rs             # Serial port commands
│       │   ├── radio.rs              # PTT, frequency, mode
│       │   └── modem.rs              # RX/TX control
│       ├── audio/                    # Audio I/O via cpal
│       │   ├── mod.rs
│       │   ├── device.rs             # Device enumeration
│       │   ├── input_stream.rs       # Capture stream → ring buffer
│       │   └── output_stream.rs      # Playback stream ← ring buffer
│       ├── dsp/                      # Signal processing
│       │   ├── mod.rs
│       │   ├── fft.rs                # 4096-point FFT for waterfall
│       │   ├── filter.rs             # FIR bandpass/lowpass
│       │   ├── nco.rs                # Numerically controlled oscillator
│       │   ├── costas_loop.rs        # BPSK carrier tracking
│       │   ├── clock_recovery.rs     # Symbol timing (Mueller-Muller)
│       │   ├── agc.rs                # Automatic gain control
│       │   └── raised_cosine.rs      # TX pulse shaping
│       ├── modem/                    # PSK31 protocol
│       │   ├── mod.rs
│       │   ├── varicode.rs           # Varicode tables + state machine
│       │   ├── encoder.rs            # Text → bits → BPSK audio
│       │   ├── decoder.rs            # BPSK audio → bits → text
│       │   └── pipeline.rs           # Full RX/TX orchestration
│       ├── cat/                      # Radio control
│       │   ├── mod.rs
│       │   ├── protocol.rs           # Command formatting/parsing
│       │   ├── serial_io.rs          # Serial port read/write
│       │   └── ft991a.rs             # FT-991A command set
│       └── state/                    # Shared app state
│           ├── mod.rs
│           └── app_state.rs          # Arc<Mutex<>> managed state
```

---

## Tauri Command API (Frontend <-> Backend)

### Commands (frontend -> Rust via `invoke()`)

| Command | Args | Returns | Purpose |
|---------|------|---------|---------|
| `list_audio_devices` | -- | `Vec<AudioDeviceInfo>` | Enumerate audio devices |
| `select_input_device` | `device_id` | -- | Set input device |
| `select_output_device` | `device_id` | -- | Set output device |
| `start_audio_stream` | `on_fft: Channel<Vec<f32>>` | -- | Start capture + FFT streaming |
| `stop_audio_stream` | -- | -- | Stop capture |
| `list_serial_ports` | -- | `Vec<SerialPortInfo>` | Enumerate serial ports |
| `connect_serial` | `port, baud` | -- | Open CAT connection |
| `disconnect_serial` | -- | -- | Close CAT connection |
| `ptt_on` / `ptt_off` | -- | -- | PTT control |
| `get_frequency` | -- | `u64` | Read VFO-A (Hz) |
| `set_frequency` | `freq: u64` | -- | Set VFO-A (Hz) |
| `get_mode` | -- | `String` | Read radio mode |
| `start_rx` | `on_text: Channel<String>` | -- | Start decoder, stream text |
| `stop_rx` | -- | -- | Stop decoder |
| `start_tx` | `text: String` | -- | Encode + transmit |
| `stop_tx` | -- | -- | Abort transmission |
| `set_carrier_frequency` | `freq_hz: f64` | -- | Set audio carrier (waterfall click) |
| `get_modem_status` | -- | `ModemStatus` | RX/TX state + signal info |

### Channels (Rust -> frontend, streaming)

| Channel | Data | Rate | Purpose |
|---------|------|------|---------|
| `on_fft` | `Vec<f32>` | ~23 fps | Waterfall magnitude data |
| `on_text` | `String` | Per char | Decoded RX text |

### Events (Rust -> frontend, notifications)

`serial-status`, `audio-status`, `tx-complete`, `error`

---

## DSP Architecture

### Key Parameters

| Parameter | Value |
|-----------|-------|
| Sample rate | 48000 Hz |
| Symbol rate | 31.25 baud |
| Samples/symbol | 1536 (exact integer) |
| FFT size | 4096 (~11.7 Hz/bin) |
| FFT window | Hanning, 50% overlap |
| FFT update rate | ~23 fps |
| RX bandpass taps | 127 |
| Costas loop BW | ~2 Hz (Bn*Ts ~ 0.06) |
| Audio carrier range | 500-2500 Hz |

### RX Pipeline

```
Audio In (48kHz mono f32)
  -> [Ring Buffer] -> DSP Thread
  -> AGC -> Bandpass Filter (centered on carrier, ~100 Hz BW)
  -> Costas Loop (carrier tracking + downmix to baseband)
  -> Clock Recovery (Mueller-Muller, 1536 samples/symbol)
  -> Bit Decision (sign of I-arm: >0 -> 1, <0 -> 0)
  -> Varicode Decode -> [Channel] -> Frontend RX Display

Parallel: Audio In -> FFT (4096-pt) -> [Channel] -> Frontend Waterfall
```

### TX Pipeline

```
Text Input -> Varicode Encode -> Phase Mapper
  -> Raised Cosine Shaping + NCO (1536 samples/symbol)
  -> [Ring Buffer] -> Audio Output (48kHz)

Sequencing: PTT ON -> 50ms delay -> Preamble -> Data -> Postamble -> PTT OFF
```

### Threading Model

- **cpal audio thread**: Only copies samples to/from lock-free ring buffers (zero allocations, zero locks)
- **DSP thread**: Pulls from input ring buffer, runs full RX pipeline, pushes decoded text via Channel
- **FFT thread**: Parallel to DSP thread, reads same input, sends waterfall data via Channel
- **TX thread**: Pulls from TX text buffer, generates audio, pushes to output ring buffer

Ring buffers: `ringbuf` crate (lock-free SPSC), sized 8192+ samples.

### Phase Ambiguity Resolution

PSK-31 uses differential encoding: bit 0 = phase reversal, bit 1 = phase constant. Detect phase *changes* rather than absolute phase to resolve the Costas loop's 180 degree ambiguity. Fallback: if no valid Varicode characters decoded for 100+ bits, invert bit sense.

---

## Implementation Phases

### Phase 1: Project Scaffolding
- `npm create tauri-app@latest` with vanilla-ts template
- Add Rust dependencies to Cargo.toml
- Create module directory structure with empty mod.rs files
- Configure tauri.conf.json (1200x800 window, app ID)
- Configure capabilities for custom commands
- Verify `npm run tauri dev` launches empty window

### Phase 2: Serial / CAT Communication
- `cat/serial_io.rs`: serial open/close/read/write
- `cat/protocol.rs`: command formatting + response parsing
- `cat/ft991a.rs`: FT-991A PTT, frequency, mode commands
- `commands/serial.rs` + `commands/radio.rs`
- Frontend: serial port selector, frequency display
- **Deliverable**: Connect to FT-991A, read/set freq, toggle PTT

### Phase 3: Audio Subsystem + Waterfall
- `audio/device.rs`: enumerate devices via cpal
- `audio/input_stream.rs`: capture stream -> ring buffer
- `dsp/fft.rs`: 4096-point FFT, Hanning window, dB output
- `commands/audio.rs`: device listing + stream control commands
- `waterfall.ts`: Canvas-based scrolling spectrogram
- `control-panel.ts` (partial): audio device selectors
- **Deliverable**: Live waterfall from selected audio input

### Phase 4: PSK-31 TX Path
- `modem/varicode.rs`: complete encode/decode tables
- `dsp/nco.rs`: numerically controlled oscillator
- `dsp/raised_cosine.rs`: TX pulse shaping
- `modem/encoder.rs`: text -> Varicode -> BPSK audio
- `audio/output_stream.rs`: playback from ring buffer
- `modem/pipeline.rs` (TX portion): full TX chain with PTT sequencing
- `tx-input.ts`: text area with TX/abort buttons
- **Deliverable**: Type text, transmit BPSK-31 via FT-991A

### Phase 5: PSK-31 RX Path (most complex)
- `dsp/filter.rs`: FIR bandpass + lowpass
- `dsp/agc.rs`: automatic gain control
- `dsp/costas_loop.rs`: BPSK carrier tracking
- `dsp/clock_recovery.rs`: Mueller-Muller symbol timing
- `modem/decoder.rs`: full decode chain
- `modem/pipeline.rs` (RX portion): threaded RX pipeline
- `rx-display.ts`: scrolling decoded text
- Waterfall click-to-tune integration
- **Deliverable**: Decode live PSK-31 signals with click-to-tune

### Phase 6: Integration + Polish
- `status-bar.ts`: connection indicators, signal level
- Error handling: serial disconnect recovery, audio hot-plug
- Waterfall controls (color palette, zoom, noise floor)
- Cross-platform testing
- Tauri bundler packaging per platform

---

## Testing Strategy

### Unit Tests (Rust `#[cfg(test)]`)
- **Varicode**: Round-trip every character, decoder state machine edge cases
- **NCO**: Frequency accuracy, phase continuity across frequency changes
- **FIR Filters**: Impulse response, frequency response at passband/stopband
- **FFT**: Known sine wave -> correct bin peak
- **Costas Loop**: Lock acquisition with synthetic BPSK at various SNR/offsets
- **Clock Recovery**: Symbol decisions with timing offsets
- **CAT Protocol**: Command formatting + response parsing (no hardware)

### Integration Tests (`src-tauri/tests/`)
- **TX->RX loopback**: Encode string -> feed audio through RX pipeline -> verify decoded output matches. This is the critical end-to-end validation.
- **CAT protocol parsing**: Full command/response sequences

### Hardware-in-the-Loop
- CAT control with FT-991A (PTT, freq, mode)
- TX: transmit known text, decode on second receiver/fldigi
- RX: decode live PSK-31 signals (on-air or from fldigi)

---

## Key Technical Risks + Mitigations

| Risk | Mitigation |
|------|-----------|
| Costas loop tuning | Use standard 2nd-order PLL formulas (Bn*Ts ~ 0.06, damping 0.707). Build loopback test first and tune against synthetic signals. |
| Phase ambiguity | Use differential encoding detection (phase changes, not absolute phase). |
| Audio thread blocking | Audio callback only copies to/from lock-free ring buffers. All DSP on separate thread. |
| Cross-platform device names | Display device name + USB vendor/product info. Highlight CP210x for FT-991A. |
| CAT command timing | 50ms minimum inter-command delay. 100ms read timeout. Command queue. |
| Waterfall performance | Pre-computed color LUT, `ImageData` pixel writes, `drawImage` scroll blit, `requestAnimationFrame` throttle. |

---

## Rust Dependencies

```toml
tauri = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
cpal = "0.16"
serialport = "4.8"
rustfft = "6.4"
num-complex = "0.4"
crossbeam-channel = "0.5"
ringbuf = "0.4"
log = "0.4"
env_logger = "0.11"
thiserror = "2"
```

### License Note

All dependencies are MIT or Apache-2.0 **except `serialport` which is MPL-2.0**. MPL-2.0 is file-level copyleft — using it unmodified in an MIT/BSD project is fine, but if we ever need to fork/modify the crate's source, those modifications must remain MPL-2.0.

**If we need to modify serial port handling**, consider replacing with:
- `tokio-serial` (MIT) — async serial, requires minor architecture changes
- Direct platform APIs — thin wrappers around termios (Unix) / Win32 (Windows)

## Frontend Dependencies

```json
{ "@tauri-apps/api": "^2" }
```

## Build Prerequisites

- **macOS**: No extras (CoreAudio built-in)
- **Linux**: `libasound2-dev`, `pkg-config`
- **Windows**: No extras for WASAPI (default)
