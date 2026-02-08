# Phase 1.5 Completion: Frontend Refactor & Config Persistence

## Status: COMPLETE

## Context

Phase 1.5 is ~85% complete. What exists:
- Full UI shell (waterfall, RX/TX panels, sidebar, status bar) with mocked data
- Dark/light theme system with localStorage persistence
- Native menu bar (4 menus, 13 items, keyboard shortcuts)
- Configuration domain type (`config.rs`) with 2 unit tests
- 19 Playwright E2E tests including visual regression
- 8 Rust unit tests across DSP and domain modules

## Remaining Work

### 1. Refactor `main.ts` into modular structure
### 2. Add Rust config persistence commands
### 3. Verify all tests pass, mark Phase 1.5 complete

---

## Architecture: Pragmatic Modularization

Split `main.ts` (367 lines) into planned directory structure using simple ES module exports. No framework abstractions needed.

### Frontend File Map

| New File | Extracted From (main.ts) | Purpose |
|----------|--------------------------|---------|
| `src/types/index.ts` | New | Shared interfaces (Configuration, MenuEvent, AudioDeviceInfo, SerialPortInfo) |
| `src/utils/color-map.ts` | WaterfallDisplay.buildColorMap() (lines 31-58) | dB → RGB palette lookup table |
| `src/utils/formatter.ts` | New | Frequency formatting helpers |
| `src/components/waterfall.ts` | Lines 16-154 | WaterfallDisplay class, imports color-map |
| `src/components/theme-toggle.ts` | Lines 257-288 | setTheme(), getCurrentTheme(), setupThemeToggle() |
| `src/components/rx-display.ts` | Lines 215-224 | setupRxDisplay() + appendRxText(), clearRxDisplay() |
| `src/components/tx-input.ts` | Lines 157-166 | setupTxInput() + getTxText(), clearTxInput(), setTxInputEnabled() |
| `src/components/control-panel.ts` | Lines 169-212 | setupTxButtons() + setTxState(), setPttState() |
| `src/components/waterfall-controls.ts` | Lines 227-254 | setupWaterfallClick() + setCarrierFrequency(), getCarrierFrequency() |
| `src/services/backend-api.ts` | New | Typed invoke() wrappers for all Tauri commands |
| `src/services/event-handlers.ts` | Lines 291-349 | setupMenuEvents(), imports theme-toggle + backend-api |

### Refactored `main.ts` (~40 lines)

After extraction, main.ts becomes just imports + DOMContentLoaded:

```typescript
import { WaterfallDisplay } from './components/waterfall';
import { setupRxDisplay } from './components/rx-display';
import { setupTxInput } from './components/tx-input';
import { setupTxButtons } from './components/control-panel';
import { setupWaterfallClick } from './components/waterfall-controls';
import { setupThemeToggle } from './components/theme-toggle';
import { setupMenuEvents } from './services/event-handlers';

window.addEventListener('DOMContentLoaded', () => {
  const canvas = document.getElementById('waterfall-canvas') as HTMLCanvasElement;
  if (canvas) {
    const waterfall = new WaterfallDisplay(canvas);
    waterfall.start();
  }

  setupTxInput();
  setupTxButtons();
  setupRxDisplay();
  setupWaterfallClick();
  setupThemeToggle();
  setupMenuEvents();
});
```

---

## Rust Config Persistence

### New file: `src-tauri/src/commands/config.rs`

4 Tauri commands for configuration file I/O:

| Command | Signature | Purpose |
|---------|-----------|---------|
| `save_configuration` | `(app: AppHandle, config: Configuration) → Result<(), String>` | Write JSON to app data dir |
| `load_configuration` | `(app: AppHandle, name: String) → Result<Configuration, String>` | Read profile by name |
| `list_configurations` | `(app: AppHandle) → Result<Vec<String>, String>` | List saved profile names (sorted) |
| `delete_configuration` | `(app: AppHandle, name: String) → Result<(), String>` | Remove profile (protect "Default") |

### Storage
- Path: `app_data_dir()/configs/<name>.json` (platform-appropriate)
- macOS: `~/Library/Application Support/com.psk31client.app/configs/`
- Filename sanitization to prevent path traversal
- JSON pretty-printed with serde_json

### Helper functions
```rust
fn config_dir(app: &AppHandle) -> Result<PathBuf, String>  // Get/create configs dir
fn sanitize_name(name: &str) -> Result<String, String>     // Reject ../ and empty names
```

### Unit tests for config.rs
- `sanitize_name_rejects_path_traversal`
- `sanitize_name_accepts_valid_names`
- (File I/O tests with temp directories if feasible)

### Wiring changes
- `src-tauri/src/commands/mod.rs`: Add `pub mod config;`
- `src-tauri/src/lib.rs`: Register all 4 config commands in `invoke_handler!`

---

## Frontend Service: `src/services/backend-api.ts`

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { Configuration, AudioDeviceInfo, SerialPortInfo } from '../types';

// Audio commands (existing stubs)
export async function listAudioDevices(): Promise<AudioDeviceInfo[]>

// Serial commands (existing stubs)
export async function listSerialPorts(): Promise<SerialPortInfo[]>

// Configuration commands (new)
export async function saveConfiguration(config: Configuration): Promise<void>
export async function loadConfiguration(name: string): Promise<Configuration>
export async function listConfigurations(): Promise<string[]>
export async function deleteConfiguration(name: string): Promise<void>
```

---

## Types: `src/types/index.ts`

```typescript
export interface AudioDeviceInfo {
  id: string;
  name: string;
  is_default: boolean;
}

export interface SerialPortInfo {
  port_name: string;
  usb_vendor_id?: number;
  usb_product_id?: number;
}

export interface Configuration {
  name: string;
  audio_input: string | null;
  audio_output: string | null;
  serial_port: string | null;
  baud_rate: number;
  radio_type: string;
  carrier_freq: number;
}

export interface MenuEvent {
  id: string;
}
```

---

## Implementation Order (Build Sequence)

### Step 1: Utilities & Types (no behavior change)
- [ ] Create `src/types/index.ts`
- [ ] Create `src/utils/color-map.ts`
- [ ] Create `src/utils/formatter.ts`
- [ ] Verify: `npm run tauri dev` works

### Step 2: Extract Components (preserve behavior)
- [ ] Create `src/components/waterfall.ts`
- [ ] Create `src/components/theme-toggle.ts`
- [ ] Create `src/components/rx-display.ts`
- [ ] Create `src/components/tx-input.ts`
- [ ] Create `src/components/control-panel.ts`
- [ ] Create `src/components/waterfall-controls.ts`
- [ ] Refactor `src/main.ts` to import all components
- [ ] Verify: `npm test` — all 19 E2E tests pass

### Step 3: Add Services
- [ ] Create `src/services/backend-api.ts`
- [ ] Create `src/services/event-handlers.ts`
- [ ] Update `src/main.ts` to use event-handlers service
- [ ] Verify: `npm test` — all 19 E2E tests pass

### Step 4: Rust Config Commands
- [ ] Create `src-tauri/src/commands/config.rs`
- [ ] Update `src-tauri/src/commands/mod.rs`
- [ ] Update `src-tauri/src/lib.rs` to register commands
- [ ] Verify: `cargo test` — all tests pass
- [ ] Verify: `cargo check` — compiles clean

### Step 5: Finalize
- [ ] Run full test suite (`npm test` + `cargo test`)
- [ ] Update visual regression snapshots if needed (`npm run test:update-snapshots`)
- [ ] Mark Phase 1.5 complete in `CLAUDE.md`
- [ ] Commit

---

## Key Constraints

- **All 19 E2E tests must pass unchanged** — DOM structure (IDs, classes) is preserved
- **No behavior changes** — pure structural refactor on frontend
- **Config UI wiring deferred** — menu items keep "coming soon" alerts; commands are available for Phase 2+
- **User confirmed**: config persistence needed now so Phase 2 serial settings can be saved
