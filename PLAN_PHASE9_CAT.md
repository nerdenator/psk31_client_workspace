# Plan: Phase 9 — CAT Expansion

## Context

The current `RadioControl` trait covers the minimum viable slice: PTT, get/set
frequency, get/set mode. Real PSK-31 operation on the FT-991A requires more:
power limiting, error response handling, signal metering, status sync, and
eventually split/RIT operation.

This plan is divided into three sessions ordered by operational importance.

---

## Current State

**Implemented (`RadioControl` trait):**
| Method | CAT command |
|--------|-------------|
| `ptt_on` | `TX1;` |
| `ptt_off` | `TX0;` |
| `is_transmitting` | (local state) |
| `get_frequency` | `FA;` |
| `set_frequency` | `FA{hz:011};` |
| `get_mode` | `MD0;` |
| `set_mode` | `MD0{code};` |

**Not implemented (this plan):**
- RF power control
- `?` error response parsing
- S-meter / signal strength
- Comprehensive status query (`IF;`)
- TX power guard before transmit
- Auto-mode verification
- Split / VFO-B
- RIT/clarifier
- IF filter width
- Periodic polling

---

## Session A: Safety + Error Handling

### A1: Parse `?` error responses in `send_command`

The FT-991A responds with `?` when a command is unrecognised or inapplicable
(e.g. setting a frequency outside the radio's tuning range, or sending a VHF
command when in HF mode). Currently `send_command` treats any response as
success.

**Change** (`ft991a.rs`, `send_command`):
```rust
let response = response.strip_prefix(cmd).unwrap_or(response);
// Add after stripping echo:
if response.trim() == "?" {
    return Err(Psk31Error::Cat(format!("Radio rejected command '{cmd}'")));
}
```

**Tests:**
- `send_command_returns_err_on_question_mark` — MockSerial returns `"?"`,
  assert `Err`
- `send_command_succeeds_on_semicolon` — MockSerial returns `";"`, assert `Ok`

---

### A2: RF power control

PSK-31 is 100% duty cycle. Running a 100W radio at full power will overheat
the finals. Standard practice: 25–50W max on HF.

**New `RadioControl` methods:**
```rust
/// Get current TX power (watts)
fn get_tx_power(&mut self) -> Psk31Result<u32>;

/// Set TX power (watts, 0–100 for HF; radio clamps to its own limits)
fn set_tx_power(&mut self, watts: u32) -> Psk31Result<()>;
```

**FT-991A CAT:**
- Query: `PC;` → `PC050;` (3-digit watts, zero-padded)
- Set: `PC050;`

**`Ft991aRadio` implementation:**
```rust
fn get_tx_power(&mut self) -> Psk31Result<u32> {
    let resp = self.send_command("PC;")?;
    // resp: "PC050;" → strip "PC" and ";" → parse "050"
    ...
}

fn set_tx_power(&mut self, watts: u32) -> Psk31Result<()> {
    let cmd = format!("PC{watts:03};");
    self.send_command(&cmd)?;
    Ok(())
}
```

**Config field** (`domain/config.rs`):
```rust
/// Maximum TX power in watts (default 25, safety cap for PSK-31)
#[serde(default = "default_tx_power")]
pub tx_power_watts: u32,
```

**Guard in TX command** (`commands/tx.rs`), before `ptt_on`:
```rust
// Apply configured TX power limit before keying up
if let Ok(mut lock) = state.radio.lock() {
    if let Some(radio) = lock.as_mut() {
        let target = state.config.lock().unwrap().tx_power_watts;
        let _ = radio.set_tx_power(target); // log warn on failure, don't abort
    }
}
```

**Settings dialog** (`settings-dialog.ts`): add TX power field (numeric, 1–100W)
to the Radio tab.

**Tests (Rust):**
- `get_tx_power_parses_response` — `"PC050;"` → 50
- `set_tx_power_sends_correct_cat` — 25W → `"PC025;"`
- `tx_power_guard_called_before_ptt` — integration test: after start_tx,
  `PC` command appears in MockSerial log before `TX1;`

**MockRadio:** add `tx_power: u32` field (default 25), implement both methods.

---

### A3: Auto-verify mode before TX

Before PTT, confirm the radio is in DATA-USB (or DATA-LSB for 60m). If not,
set it. This prevents accidentally transmitting PSK-31 audio through an SSB
chain with wrong sideband.

**New helper in `commands/tx.rs`:**
```rust
fn ensure_data_mode(radio: &mut dyn RadioControl, freq_hz: f64) {
    let expected = if freq_hz < 10_000_000.0 { "DATA-LSB" } else { "DATA-USB" };
    if let Ok(mode) = radio.get_mode() {
        if mode != expected {
            let _ = radio.set_mode(expected);
        }
    }
}
```

Call this from `start_tx`, after the TX power guard, before `ptt_on`.

---

## Session B: Signal Metering + Status Sync

### B1: S-meter reading

Adds a signal strength value the UI can display (replaces the removed signal
bars with something real).

**New `RadioControl` method:**
```rust
/// Read S-meter level, returned as 0.0–1.0 (normalised from radio's scale)
fn get_signal_strength(&mut self) -> Psk31Result<f32>;
```

**FT-991A CAT:**
- `SM0;` → `SM00012;` (0–30 scale, where 9 = S9, roughly)
- Parse the 4-digit value after `SM0`, normalise to 0–1: `value / 30.0`

**Polling:** add a `signal_poll` thread in `AppState` (similar to the audio
thread) that queries `SM0;` every 500ms while connected and emits a
`"signal-strength"` event with a `f32` payload. Stop on disconnect.

**Frontend:** reattach a minimal signal indicator in the status bar (a numeric
dB-style readout, not bars).

**Tests:**
- `get_signal_strength_parses_sm0_response` — `"SM00015;"` → ~0.5
- `get_signal_strength_handles_zero` — `"SM00000;"` → 0.0
- `get_signal_strength_clamps_to_one` — `"SM00031;"` → 1.0 (not > 1.0)

---

### B2: Comprehensive status query (`IF;`)

The `IF;` command returns a single 37-character response containing VFO
frequency, mode, PTT state, split state, and more. Useful to sync the UI on
connect (one command instead of several).

**FT-991A response format:**
```
IF00014070000     +0000000000 0000 00 0;
   ^-----------^  freq (11 digits, VFO-A Hz)
                  ...mode, PTT, split flags embedded
```

**New `RadioControl` method:**
```rust
/// Read comprehensive radio status in one round-trip
fn get_status(&mut self) -> Psk31Result<RadioStatus>;
```

**New domain type** (`domain/types.rs`):
```rust
pub struct RadioStatus {
    pub frequency_hz: f64,
    pub mode: String,
    pub is_transmitting: bool,
    pub split: bool,
    pub tx_power_watts: u32,
}
```

**Use in `connect_serial`:** replace the two separate `get_frequency` +
`get_mode` calls with a single `get_status()` call.

**Tests:**
- `parse_if_response_extracts_freq_and_mode` — full IF response → correct fields
- `get_status_uses_if_command` — MockSerial log contains `"IF;"` (not `"FA;"`)

---

## Session C: Advanced Operation

### C1: Split / VFO-B

Split mode transmits on VFO-B while receiving on VFO-A — standard for
answering CQ calls without moving your listening frequency.

**New `RadioControl` methods:**
```rust
fn get_vfo_b_frequency(&mut self) -> Psk31Result<Frequency>;
fn set_vfo_b_frequency(&mut self, freq: Frequency) -> Psk31Result<()>;
fn set_split(&mut self, enabled: bool) -> Psk31Result<()>;
fn is_split(&self) -> bool;
```

**FT-991A CAT:**
- `FB;` / `FB{hz:011};` — VFO-B frequency
- `FT0;` (TX on VFO-A) / `FT1;` (TX on VFO-B) — split TX routing

**UI:** split toggle button in sidebar; VFO-B frequency input appears when
split is enabled.

---

### C2: RIT / Clarifier

RIT lets you offset the RX frequency a few hundred Hz without moving the TX
frequency — useful when the other station is slightly off-frequency.

**New `RadioControl` methods:**
```rust
fn set_rit(&mut self, enabled: bool) -> Psk31Result<()>;
fn set_rit_offset_hz(&mut self, offset: i32) -> Psk31Result<()>;
fn clear_rit(&mut self) -> Psk31Result<()>;
```

**FT-991A CAT:**
- `RT0;` / `RT1;` — RIT off/on
- `RC;` — clear RIT offset
- `RD{offset:04};` / `RU{offset:04};` — RIT down/up in 10Hz steps

---

### C3: IF filter width

PSK-31 occupies ~31 Hz. Setting a narrow IF filter (200–300 Hz) dramatically
improves adjacent-signal rejection. FT-991A DATA mode has adjustable roofing
and DSP bandwidth.

**New `RadioControl` method:**
```rust
fn set_if_bandwidth_hz(&mut self, hz: u32) -> Psk31Result<()>;
```

**FT-991A CAT:** `SH0{code};` — bandwidth codes differ by mode; needs a lookup
table similar to `MODE_TABLE`.

---

## Trait Summary (end state after all sessions)

```rust
pub trait RadioControl: Send {
    // --- existing ---
    fn ptt_on(&mut self) -> Psk31Result<()>;
    fn ptt_off(&mut self) -> Psk31Result<()>;
    fn is_transmitting(&self) -> bool;
    fn get_frequency(&mut self) -> Psk31Result<Frequency>;
    fn set_frequency(&mut self, freq: Frequency) -> Psk31Result<()>;
    fn get_mode(&mut self) -> Psk31Result<String>;
    fn set_mode(&mut self, mode: &str) -> Psk31Result<()>;

    // --- Session A ---
    fn get_tx_power(&mut self) -> Psk31Result<u32>;
    fn set_tx_power(&mut self, watts: u32) -> Psk31Result<()>;

    // --- Session B ---
    fn get_signal_strength(&mut self) -> Psk31Result<f32>;
    fn get_status(&mut self) -> Psk31Result<RadioStatus>;

    // --- Session C ---
    fn get_vfo_b_frequency(&mut self) -> Psk31Result<Frequency>;
    fn set_vfo_b_frequency(&mut self, freq: Frequency) -> Psk31Result<()>;
    fn set_split(&mut self, enabled: bool) -> Psk31Result<()>;
    fn is_split(&self) -> bool;
    fn set_rit(&mut self, enabled: bool) -> Psk31Result<()>;
    fn set_rit_offset_hz(&mut self, offset: i32) -> Psk31Result<()>;
    fn clear_rit(&mut self) -> Psk31Result<()>;
    fn set_if_bandwidth_hz(&mut self, hz: u32) -> Psk31Result<()>;
}
```

Every new method must be implemented in both `Ft991aRadio` and `MockRadio`.
`MockRadio` holds all state in memory and logs every call at INFO.

---

## Testing Strategy

Each session follows the same pattern:

1. **Unit tests** (`ft991a.rs` `#[cfg(test)]`): wire format correctness using
   the existing `MockSerial` + `make_radio` infrastructure
2. **Integration tests** (`tests/cat_integration.rs`): state transitions
   through `AppState` using `MockRadio`
3. **E2E tests** (Playwright): UI elements that reflect new CAT state
   (TX power field in settings, signal strength in status bar, split toggle)
4. **Manual / hardware**: smoke test each new command against the real FT-991A

---

## Status

- [ ] Session A: Safety + error handling (power control, `?` parsing, mode guard)
- [ ] Session B: Signal metering + status sync (S-meter, `IF;` query)
- [ ] Session C: Advanced operation (split, RIT, IF filter)
