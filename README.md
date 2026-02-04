# PSK-31 Desktop Client

Cross-platform desktop application for PSK-31 ham radio keyboard-to-keyboard communication.

## Features

- PSK-31 BPSK modulation/demodulation
- Spectral waterfall display with click-to-tune
- TX text input with transmit/abort controls
- RX decoded text display
- Audio device selection
- CAT control for Yaesu FT-991A (more radios planned)
- Light/dark theme with system preference detection

## Screenshots

*Coming soon*

## Requirements

### Build Dependencies

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.70+
- Platform-specific dependencies:
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Linux**: `build-essential`, `libwebkit2gtk-4.1-dev`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libasound2-dev`
  - **Windows**: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with C++ workload

### Runtime

- USB audio interface (radio or SignaLink)
- Serial port for CAT control (optional)

## Getting Started

```bash
# Install dependencies
npm install

# Run in development mode (hot reload)
npm run tauri dev

# Build for production
npm run tauri build
```

## Architecture

This project uses **hexagonal architecture** (ports & adapters) in the Rust backend to separate core domain logic from external I/O:

```
src-tauri/src/
├── domain/      # Pure types (AudioDeviceInfo, Frequency, errors)
├── ports/       # Trait definitions (AudioInput, AudioOutput, RadioControl)
├── dsp/         # Signal processing (FFT, NCO, filters, Costas loop)
├── modem/       # PSK-31 protocol (varicode, encoder, decoder)
├── adapters/    # Implementations (cpal audio, serialport, FT-991A CAT)
├── commands/    # Tauri command handlers
└── state.rs     # Application state
```

### Frontend

- Vanilla TypeScript + Vite
- No framework dependencies
- Component-based organization in `src/`

### Key Design Decisions

- **48000 Hz sample rate** — native USB audio rate for FT-991A
- **31.25 baud** — PSK-31 symbol rate (1536 samples/symbol)
- **Lock-free audio** — ring buffers between audio thread and DSP
- **Pure DSP functions** — all signal processing is testable without hardware

## Project Structure

```
psk31_client_workspace/
├── src/                    # Frontend (TypeScript)
│   ├── main.ts             # App entry, UI initialization
│   └── styles.css          # Theming, layout
├── src-tauri/              # Backend (Rust)
│   ├── src/
│   │   ├── domain/         # Core types
│   │   ├── ports/          # Trait interfaces
│   │   ├── dsp/            # Signal processing
│   │   ├── modem/          # PSK-31 protocol
│   │   ├── adapters/       # Hardware implementations
│   │   └── commands/       # Tauri IPC handlers
│   └── Cargo.toml
├── PLAN.md                 # Detailed implementation plan
└── CLAUDE.md               # Development guidelines
```

## Development

```bash
# Check Rust compilation
cd src-tauri && cargo check

# Run Rust tests
cd src-tauri && cargo test

# Format code
cd src-tauri && cargo fmt
```

## Roadmap

- [x] Phase 1: Project scaffolding
- [ ] Phase 1.5: Frontend layout & visual tests
- [ ] Phase 2: Serial / CAT communication
- [ ] Phase 3: Audio subsystem + waterfall
- [ ] Phase 4: PSK-31 TX path
- [ ] Phase 5: PSK-31 RX path
- [ ] Phase 6: Integration + polish

See [PLAN.md](PLAN.md) for detailed implementation phases.

## License

MIT

## Acknowledgments

- Fonts: [IBM Plex Mono](https://github.com/IBM/plex) and [JetBrains Mono](https://github.com/JetBrains/JetBrainsMono) (SIL OFL 1.1)
- Inspired by [JS8Call](http://js8call.com/) and [WSJT-X](https://wsjt.sourceforge.io/)
