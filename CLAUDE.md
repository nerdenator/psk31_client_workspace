# PSK-31 Desktop Client

Cross-platform desktop application for PSK-31 ham radio keyboard-to-keyboard communication.

## Tech Stack

- **Framework**: Tauri 2.x (Rust backend + web frontend)
- **Frontend**: Vanilla TypeScript + Vite
- **Backend**: Rust with hexagonal architecture
- **Target Radio**: Yaesu FT-991A (USB audio + CAT serial)

## Architecture

Hexagonal (ports & adapters) pattern in the Rust backend:

```
src-tauri/src/
├── domain/      # Pure types (AudioDeviceInfo, Frequency, ModemConfig, errors)
├── ports/       # Trait definitions (AudioInput, AudioOutput, SerialConnection, RadioControl)
├── dsp/         # Signal processing - pure functions (FFT, NCO, filters, Costas loop)
├── modem/       # PSK-31 protocol (varicode, encoder, decoder, pipeline)
├── adapters/    # Implementations (cpal audio, serialport, FT-991A CAT)
├── commands/    # Tauri command handlers
└── state.rs     # AppState with Arc<Mutex<>>
```

## Key Files

- `PLAN.md` — Full implementation plan with 6 phases
- `src-tauri/src/lib.rs` — Tauri app builder, command registration
- `src-tauri/src/modem/varicode.rs` — PSK-31 Varicode tables (complete)
- `src-tauri/src/dsp/fft.rs` — FFT processor with tests
- `src-tauri/src/dsp/nco.rs` — Numerically controlled oscillator with tests

## Design Philosophy

- **Inspired by**: JS8Call, WSJT-X — clean, functional, no clutter
- **Not like**: fldigi — we do ONE thing (PSK-31 keyboard QSOs) well
- Dark theme, waterfall prominent, monospace fonts for RX/TX

## Commands

```bash
npm run tauri dev      # Development with hot reload
cargo check            # Check Rust compilation (in src-tauri/)
cargo test             # Run Rust unit tests (in src-tauri/)
```

## Testing

- **Rust unit tests**: `#[cfg(test)]` modules in each file
- **Playwright**: Visual regression tests with mocked Tauri IPC (Phase 1.5)
- **Integration**: TX→RX loopback test (encoder output → decoder)

## Current Status

- Phase 1 complete: Project scaffolding, hexagonal module structure
- Next: Phase 1.5 (Frontend layout & visual tests)

## License Consideration

All dependencies MIT/Apache-2.0 except `serialport` (MPL-2.0). If we need to modify serialport, consider `tokio-serial` (MIT) instead.
