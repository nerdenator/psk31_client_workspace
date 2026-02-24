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

## Architecture: Hexagonal (Ports & Adapters)

The Rust backend uses **hexagonal architecture** to separate core domain logic from external I/O concerns. This enables:
- Unit testing the modem/DSP without hardware
- Easy substitution of adapters (e.g., swap `serialport` crate if needed)
- Clear boundaries between pure logic and side effects

```
                    ┌─────────────────────────────────────┐
                    │           DRIVING ADAPTERS          │
                    │  (Tauri commands, UI events)        │
                    └─────────────────┬───────────────────┘
                                      │ calls
                                      ▼
┌─────────────────────────────────────────────────────────────────────┐
│                              CORE DOMAIN                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │    modem    │  │     dsp     │  │   domain    │  (pure logic,   │
│  │  varicode   │  │  fft, nco   │  │   types     │   no I/O deps)  │
│  │  enc/dec    │  │  filters    │  │             │                  │
│  └─────────────┘  └─────────────┘  └─────────────┘                  │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                         PORTS (traits)                      │    │
│  │  AudioInput, AudioOutput, RadioControl, SerialConnection    │    │
│  └─────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
                                      │ implemented by
                                      ▼
                    ┌─────────────────────────────────────┐
                    │          DRIVEN ADAPTERS            │
                    │  CpalAudio, SerialPortAdapter,      │
                    │  Ft991aRadio                        │
                    └─────────────────────────────────────┘
```

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
│   │   ├── color-map.ts              # dB -> RGB for waterfall
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
│       │
│       ├── domain/                   # CORE: Pure domain types, no I/O
│       │   ├── mod.rs
│       │   ├── types.rs              # AudioSample, Frequency, ModemConfig, etc.
│       │   └── error.rs              # Domain error types
│       │
│       ├── ports/                    # CORE: Port traits (interfaces)
│       │   ├── mod.rs
│       │   ├── audio.rs              # trait AudioInput, trait AudioOutput
│       │   ├── serial.rs             # trait SerialConnection
│       │   └── radio.rs              # trait RadioControl (PTT, freq, mode)
│       │
│       ├── dsp/                      # CORE: Signal processing (pure functions)
│       │   ├── mod.rs
│       │   ├── fft.rs                # 4096-point FFT for waterfall
│       │   ├── filter.rs             # FIR bandpass/lowpass
│       │   ├── nco.rs                # Numerically controlled oscillator
│       │   ├── costas_loop.rs        # BPSK carrier tracking
│       │   ├── clock_recovery.rs     # Symbol timing (Mueller-Muller)
│       │   ├── agc.rs                # Automatic gain control
│       │   └── raised_cosine.rs      # TX pulse shaping
│       │
│       ├── modem/                    # CORE: PSK31 protocol logic
│       │   ├── mod.rs
│       │   ├── varicode.rs           # Varicode tables + state machine
│       │   ├── encoder.rs            # Text -> bits -> BPSK samples
│       │   ├── decoder.rs            # BPSK samples -> bits -> text
│       │   └── pipeline.rs           # RX/TX orchestration (uses port traits)
│       │
│       ├── adapters/                 # ADAPTERS: External I/O implementations
│       │   ├── mod.rs
│       │   ├── cpal_audio.rs         # impl AudioInput/AudioOutput via cpal
│       │   ├── serial_port.rs        # impl SerialConnection via serialport crate
│       │   └── ft991a.rs             # impl RadioControl for FT-991A CAT
│       │
│       ├── commands/                 # DRIVING ADAPTERS: Tauri command handlers
│       │   ├── mod.rs
│       │   ├── audio.rs              # Audio device commands
│       │   ├── serial.rs             # Serial port commands
│       │   ├── radio.rs              # PTT, frequency, mode
│       │   └── modem.rs              # RX/TX control
│       │
│       └── state/                    # Application state, wires adapters to core
│           ├── mod.rs
│           └── app_state.rs          # Arc<Mutex<dyn Port>> managed state
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

## Design Philosophy

**Inspiration**: JS8Call, WSJT-X — clean, functional, no clutter.

**Anti-inspiration**: fldigi — too many modes, too much configuration, overwhelming for newcomers.

**Goal**: A streamlined, single-purpose app for PSK-31 keyboard-to-keyboard communication. Do one thing well.

**Visual principles**:
- Dark theme (easy on eyes during night operating)
- Waterfall display prominent at top
- Clear visual separation: RX text (top), TX text (bottom), controls (sidebar or bottom bar)
- Minimal chrome — no unnecessary borders, shadows, or decorations
- Monospace fonts for RX/TX text (readability of callsigns, signal reports)
- High contrast for important state indicators (TX/RX, PTT, connection status)

---

## Implementation Phases

Each phase includes implementation, unit tests, and E2E tests. Tests are written alongside code, not as an afterthought.

### Phase 1: Project Scaffolding ✓
**Implementation:**
- `npm create tauri-app@latest` with vanilla-ts template
- Add Rust dependencies to Cargo.toml
- Create hexagonal module structure: `domain/`, `ports/`, `dsp/`, `modem/`, `adapters/`, `commands/`, `state/`
- Define port traits: `AudioInput`, `AudioOutput`, `SerialConnection`, `RadioControl`
- Configure tauri.conf.json (1200x800 window, app ID)
- Configure capabilities for custom commands

**Verification:**
- `npm run tauri dev` launches empty window
- `cargo check` passes

### Phase 1.5: Frontend Layout & Visual Tests ✓
**Implementation:**
- Complete UI shell with mocked data (waterfall, RX/TX panels, sidebar, status bar)
- Dark/light theme CSS with JS8Call/WSJT-X aesthetic
- Native menu bar (File, Configurations, View, Help)
- Configuration domain type for saved profiles

**Unit Tests (Rust):**
- `domain/config.rs`: Configuration serialization, default values

**E2E Tests (Playwright):**
- Main layout structure (7 tests)
- Theme toggle functionality (3 tests)
- TX/RX panel interactions (4 tests)
- Waterfall click-to-tune (1 test)
- Menu event handling (2 tests)
- Visual regression - light/dark themes (2 tests)

**Commands:** `npm test`, `cargo test`

### Phase 2: Serial / CAT Communication
**Implementation:**
- `ports/serial.rs`: define `trait SerialConnection`
- `ports/radio.rs`: define `trait RadioControl` (PTT, freq, mode)
- `adapters/serial_port.rs`: impl `SerialConnection` via serialport crate
- `adapters/ft991a.rs`: impl `RadioControl` for FT-991A CAT protocol
- `commands/serial.rs` + `commands/radio.rs`: Tauri command handlers
- Frontend: serial port selector, frequency display, connect/disconnect

**Unit Tests (Rust):**
- `ft991a.rs`: CAT command formatting (`TX1;`, `FA00014070000;`)
- `ft991a.rs`: Response parsing (`FA00014070000;` → 14.070 MHz)
- `ft991a.rs`: Mode parsing (`MD0C;` → DATA-USB)
- `serial_port.rs`: Mock serial read/write with fake bytes

**E2E Tests (Playwright):**
- Serial port dropdown populates (mocked device list)
- Connect button state changes on click
- Frequency display updates from mocked backend
- PTT indicator shows TX/RX state
- Error state when connection fails

**Deliverable:** Connect to FT-991A, read/set freq, toggle PTT

### Phase 3: Audio Subsystem + Waterfall
**Implementation:**
- `ports/audio.rs`: define `trait AudioInput`, `trait AudioOutput`
- `adapters/cpal_audio.rs`: impl audio traits via cpal
- `dsp/fft.rs`: 4096-point FFT, Hanning window, dB output
- `commands/audio.rs`: Tauri command handlers
- Wire live FFT data to waterfall canvas
- Audio device selectors in sidebar

**Unit Tests (Rust):**
- `fft.rs`: Known sine wave → correct bin peak
- `fft.rs`: Windowing function correctness
- `fft.rs`: dB conversion accuracy
- `cpal_audio.rs`: Device enumeration (mock cpal host)

**E2E Tests (Playwright):**
- Audio input/output dropdowns populate
- Waterfall canvas receives and renders data (mocked FFT stream)
- Click-to-tune updates carrier frequency display
- Audio device selection persists

**Deliverable:** Live waterfall from selected audio input

### Phase 4: PSK-31 TX Path
**Implementation:**
- `modem/varicode.rs`: complete encode/decode tables
- `dsp/nco.rs`: numerically controlled oscillator
- `dsp/raised_cosine.rs`: TX pulse shaping
- `modem/encoder.rs`: text → Varicode → BPSK samples
- `modem/pipeline.rs` (TX): orchestrates encoder + audio output + radio control
- TX input with character counter, TX/Abort buttons

**Unit Tests (Rust):**
- `varicode.rs`: Encode every printable ASCII character
- `varicode.rs`: Round-trip encode→decode
- `nco.rs`: Frequency accuracy over time
- `nco.rs`: Phase continuity across frequency changes
- `raised_cosine.rs`: Pulse shape symmetry
- `encoder.rs`: Known text → expected bit sequence
- `encoder.rs`: Preamble/postamble generation

**E2E Tests (Playwright):**
- TX button disabled when input empty
- Character counter updates on typing
- TX button triggers transmit state (mocked)
- Abort button cancels transmission
- PTT indicator shows TX during transmission
- TX input disabled during transmission

**Integration Test (Rust):**
- Encoder output → loopback → decoder input → original text

**Deliverable:** Type text, transmit BPSK-31 via FT-991A

### Phase 5: PSK-31 RX Path
**Implementation:**
- `dsp/filter.rs`: FIR bandpass + lowpass
- `dsp/agc.rs`: automatic gain control
- `dsp/costas_loop.rs`: BPSK carrier tracking
- `dsp/clock_recovery.rs`: Mueller-Muller symbol timing
- `modem/decoder.rs`: full decode chain
- `modem/pipeline.rs` (RX): audio input → DSP → decoder
- RX display with scrolling decoded text
- Waterfall click-to-tune integration

**Unit Tests (Rust):**
- `filter.rs`: Impulse response matches design
- `filter.rs`: Frequency response at passband/stopband edges
- `agc.rs`: Gain adjustment for varying input levels
- `costas_loop.rs`: Lock acquisition with clean BPSK
- `costas_loop.rs`: Lock acquisition with frequency offset
- `costas_loop.rs`: Phase tracking accuracy
- `clock_recovery.rs`: Symbol timing with various offsets
- `decoder.rs`: Synthetic BPSK → correct text
- `decoder.rs`: Handle phase ambiguity (180° flip)

**E2E Tests (Playwright):**
- RX display shows decoded text (mocked decoder stream)
- RX display scrolls with new text
- Clear button empties RX display
- Click waterfall updates carrier frequency
- Signal level indicator responds to input level

**Integration Test (Rust):**
- Full TX→RX loopback: text → encoder → decoder → text
- Loopback with simulated noise
- Loopback with frequency offset

**Deliverable:** Decode live PSK-31 signals with click-to-tune

### Phase 6: Integration + Polish
**Implementation:**
- Status bar: connection indicators, signal level meter
- Error handling: serial disconnect recovery, audio hot-plug
- Waterfall controls (color palette selector, zoom, noise floor adjustment)
- Configuration save/load (persist to app data directory)
- Cross-platform testing (macOS, Windows, Linux)
- Tauri bundler packaging per platform

**Unit Tests (Rust):**
- Configuration file read/write
- Error recovery state machine

**E2E Tests (Playwright):**
- Full application flow: connect → receive → transmit → disconnect
- Settings dialog opens and saves preferences
- Configuration switching works
- Theme persists across restart
- Error messages display correctly
- Visual regression for all UI states

**Deliverable:** Production-ready application with installers

---

### Phase 7: UI Polish
**Implementation:**
- Session A: Status bar cleanup (remove signal bars, Mode/Rate labels), waterfall black on init, clear RX placeholder
- Session B: Band selector + per-band frequency input (replace static frequency display)

**Status:**
- [x] Session A: Status bar cleanup + waterfall black + clear RX placeholder
- [ ] Session B: Band selector + per-band frequency input

---

## Testing Strategy

Tests are written alongside code in each phase, not as an afterthought. Hexagonal architecture enables thorough testing of the **core domain** (dsp/, modem/) without any hardware or I/O dependencies.

### Test Commands

```bash
# Run all Playwright E2E tests
npm test

# Run Playwright with interactive UI
npm run test:ui

# Update visual regression snapshots
npm run test:update-snapshots

# Run Rust unit tests
cargo test

# Run specific Rust test
cargo test test_name
```

### Test Types

| Type | Location | Purpose |
|------|----------|---------|
| **Rust Unit** | `src-tauri/src/**/mod.rs` | Pure function correctness (DSP, modem, domain) |
| **Rust Integration** | `src-tauri/tests/` | Cross-module tests (TX→RX loopback) |
| **Playwright E2E** | `tests/e2e/` | UI behavior, visual regression |

### Playwright Configuration

- **Browser**: WebKit (Safari engine)
- **Base URL**: `http://localhost:1420` (Vite dev server)
- **Visual regression**: Snapshots in `tests/e2e/*.spec.ts-snapshots/`
- **Animated elements**: Masked to prevent flaky comparisons

### Hardware-in-the-Loop (Manual)

After all automated tests pass, verify with real hardware:
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
