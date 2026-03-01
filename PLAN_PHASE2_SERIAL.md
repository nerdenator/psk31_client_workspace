# Phase 2: Serial / CAT Communication — Implementation Plan

## Status: Ready for Implementation (User Approved Architecture)

## Context

Phase 1.5 is COMPLETE. The app has:
- Modular frontend (types/, utils/, components/, services/)
- Configuration persistence (4 Rust commands)
- Full UI shell with mocked serial port dropdown, frequency display, connection status
- 18 Playwright E2E tests + 11 Rust unit tests
- Port traits defined: `SerialConnection`, `RadioControl`
- Empty adapters directory

## Decisions Made

1. **Split SerialConnection trait**: Separate `SerialFactory` (list/open) from `SerialConnection` (instance read/write/close)
2. **Ft991a owns serial**: Radio adapter wraps and owns the serial port
3. **Auto-detect on connect**: Open serial -> read frequency -> read mode -> update UI
4. **Separate test file**: `serial.spec.ts` for Phase 2 E2E tests
5. **Concrete state**: `AppState.radio: Mutex<Option<Box<dyn RadioControl>>>`

---

## Files to Create (5)

### 1. `src-tauri/src/adapters/serial_port.rs` (~150 lines)
- `SerialPortFactory` — implements `list_ports()` via `serialport::available_ports()`
- `SerialPortConnection` — wraps `Box<dyn serialport::SerialPort>`
- Map `serialport::Error` → `Psk31Error::Serial`
- Read timeout: 100ms
- Port type formatting: "USB (VID:PID)" for CP210x, "Native" for built-in

### 2. `src-tauri/src/adapters/ft991a.rs` (~250 lines)
- `Ft991aRadio` struct: owns `Box<dyn SerialConnection>`, tracks `is_transmitting`, `last_command_time`
- Implements `RadioControl` trait
- Private helpers: `send_command()`, `ensure_command_delay()`, `parse_frequency()`, `parse_mode()`
- `impl Drop` — auto-releases PTT on disconnect
- Unit tests for CAT parsing

**CAT command mapping**:
| Method | Command | Response | Parse |
|--------|---------|----------|-------|
| `ptt_on()` | `TX1;` | echo | verify |
| `ptt_off()` | `TX0;` | echo | verify |
| `get_frequency()` | `FA;` | `FA00014070000;` | extract 11-digit Hz |
| `set_frequency(f)` | `FA{hz:011};` | echo | verify |
| `get_mode()` | `MD0;` | `MD0C;` | map char → name |
| `set_mode(m)` | `MD0{code};` | echo | verify |

**Mode map**: 1=LSB, 2=USB, 3=CW, 4=FM, C=DATA-USB

### 3. `src-tauri/src/commands/radio.rs` (~80 lines)
6 Tauri commands:
- `ptt_on(state)` / `ptt_off(state)`
- `get_frequency(state)` → `f64` / `set_frequency(state, freq_hz: f64)`
- `get_mode(state)` → `String` / `set_mode(state, mode: String)`

Pattern: Lock `state.radio`, check connected, call trait method, map error to String.

### 4. `src/components/serial-panel.ts` (~120 lines)
- `setupSerialPanel()`: Wire serial dropdown, connect/disconnect button
- On load: call `listSerialPorts()`, populate dropdown (replacing hardcoded options)
- On connect: call `connectSerial()`, update frequency display + mode badge + CAT status dot
- On disconnect: call `disconnectSerial()`, reset UI
- Error handling: show error message on failure

### 5. `tests/e2e/serial.spec.ts` (~150 lines)
5 tests:
1. Serial port dropdown populates (mocked device list)
2. Connect button updates frequency display
3. Connection status indicator shows connected
4. Disconnect button resets UI
5. Error message displays on connection failure

---

## Files to Modify (8)

### 6. `src-tauri/src/ports/serial.rs`
Replace with split traits:
```rust
pub trait SerialFactory {
    fn list_ports() -> Psk31Result<Vec<SerialPortInfo>>;
    fn open(port: &str, baud_rate: u32) -> Psk31Result<Box<dyn SerialConnection>>;
}

pub trait SerialConnection: Send {
    fn write(&mut self, data: &[u8]) -> Psk31Result<usize>;
    fn read(&mut self, buffer: &mut [u8]) -> Psk31Result<usize>;
    fn write_read(&mut self, command: &str, response_buf: &mut [u8]) -> Psk31Result<usize> {
        self.write(command.as_bytes())?;
        self.read(response_buf)
    }
    fn close(&mut self) -> Psk31Result<()>;
    fn is_connected(&self) -> bool;
}
```

### 7. `src-tauri/src/adapters/mod.rs`
```rust
pub mod serial_port;
pub mod ft991a;
```

### 8. `src-tauri/src/state.rs`
Add: `pub radio: Mutex<Option<Box<dyn RadioControl>>>`

### 9. `src-tauri/src/domain/types.rs`
Add `RadioInfo` struct:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioInfo {
    pub port: String,
    pub baud_rate: u32,
    pub frequency_hz: f64,
    pub mode: String,
    pub connected: bool,
}
```

### 10. `src-tauri/src/commands/serial.rs`
Replace stub with:
- `list_serial_ports()` — calls `SerialPortFactory::list_ports()`
- `connect_serial(state, port, baud_rate)` — open, create Ft991a, auto-detect, store in state, return RadioInfo
- `disconnect_serial(state)` — take from state, drop

### 11. `src-tauri/src/commands/mod.rs`
Add: `pub mod radio;`

### 12. `src-tauri/src/lib.rs`
Register 8 new commands:
```rust
commands::serial::connect_serial,
commands::serial::disconnect_serial,
commands::radio::ptt_on,
commands::radio::ptt_off,
commands::radio::get_frequency,
commands::radio::set_frequency,
commands::radio::get_mode,
commands::radio::set_mode,
```

### 13. `src/types/index.ts`
Add:
```typescript
export interface RadioInfo {
  port: string;
  baud_rate: number;
  frequency_hz: number;
  mode: string;
  connected: boolean;
}
```

### 14. `src/services/backend-api.ts`
Add 8 wrappers:
```typescript
// Serial
connectSerial(port, baudRate) → RadioInfo
disconnectSerial() → void

// Radio
pttOn() / pttOff() → void
getFrequency() → number
setFrequency(freqHz) → void
getMode() → string
setMode(mode) → void
```

### 15. `src/main.ts`
Add: `import { setupSerialPanel } from './components/serial-panel';`
Add: `setupSerialPanel();` in DOMContentLoaded

---

## Build Sequence

### Step 1: Ports & Domain (no behavior change)
- [ ] Refactor `ports/serial.rs` — split into SerialFactory + SerialConnection
- [ ] Add `RadioInfo` to `domain/types.rs`
- [ ] Verify: `cargo check`

### Step 2: Serial Port Adapter
- [ ] Create `adapters/serial_port.rs`
- [ ] Update `adapters/mod.rs`
- [ ] Verify: `cargo check`

### Step 3: FT-991A Radio Adapter
- [ ] Create `adapters/ft991a.rs` with unit tests
- [ ] Update `adapters/mod.rs` (add ft991a)
- [ ] Verify: `cargo test` — all tests pass

### Step 4: State + Commands
- [ ] Update `state.rs` — add radio field
- [ ] Update `commands/serial.rs` — implement connect/disconnect
- [ ] Create `commands/radio.rs` — 6 radio commands
- [ ] Update `commands/mod.rs` — add radio module
- [ ] Update `lib.rs` — register 8 new commands
- [ ] Verify: `cargo check && cargo test`

### Step 5: Frontend
- [ ] Update `types/index.ts` — add RadioInfo
- [ ] Update `backend-api.ts` — add 8 command wrappers
- [ ] Create `components/serial-panel.ts`
- [ ] Update `main.ts` — import + setup
- [ ] Verify: TypeScript compiles clean

### Step 6: E2E Tests
- [ ] Create `tests/e2e/serial.spec.ts`
- [ ] Verify: `npm test` — all tests pass (18 existing + new)

### Step 7: Finalize
- [ ] Run full test suite
- [ ] Mark Phase 2 complete in `CLAUDE.md`
- [ ] Commit

---

## Key Constraints

- **18 existing E2E tests must still pass** — DOM structure preserved
- **No behavior changes to existing UI** — serial panel is additive
- **50ms minimum between CAT commands** — FT-991A timing requirement
- **100ms serial read timeout** — prevent UI blocking
- **PTT auto-release on drop** — safety for radio hardware
- **Port name sanitization** — prevent path traversal

## Error Handling Pattern

```
serialport::Error → Psk31Error::Serial(String)
                   → commands return Result<T, String>
                   → frontend shows error message
```

## Connect Data Flow

```
User clicks Connect
  → serial-panel.ts → invoke('connect_serial', {port, baud_rate})
  → commands/serial.rs → SerialPortFactory::open()
  → Creates SerialPortConnection
  → Ft991aRadio::new(connection)
  → radio.get_frequency() [CAT: FA;]
  → radio.get_mode() [CAT: MD0;]
  → Store in state.radio
  → Return RadioInfo
  → Frontend updates: freq display, mode badge, CAT status dot
```
