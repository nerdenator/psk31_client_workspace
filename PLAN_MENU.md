# Plan: Add Native Menu Bar to PSK-31 Client

## Overview

Add native application menus to the PSK-31 desktop client using Tauri 2.x menu API. Inspired by WSJT-X layout.

## Key Concept: Configurations as Profiles

A **Configuration** is a saved profile containing all settings (audio devices, serial port, radio, modem params). Users can:
- Create/edit configurations via **Settings** dialog (opens from app menu)
- Save multiple configurations (e.g., "FT-991A Home", "IC-7300 Portable")
- Switch between saved configurations via **Configurations** menu

This enables quick switching between different radio setups.

## Menu Structure

```
┌─────────────────────────────────────────────────────────────────────────┐
│ PSK-31 │ File │ Configurations │ View │ Help                           │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1. PSK-31 (macOS App Menu)
- About PSK-31
- ─────────────
- Settings... (`Cmd+,`) → Opens settings window
- ─────────────
- Services
- Hide PSK-31 (`Cmd+H`)
- Hide Others
- ─────────────
- Quit PSK-31 (`Cmd+Q`)

### 2. File
- Settings... (`Cmd+,` / `Ctrl+,`)
- ─────────────
- Exit (`Cmd+Q` / `Alt+F4`)

### 3. Configurations (dynamic list of saved profiles)
- Default ✓ (checkmark on active)
- ─────────────
- *[User-created configurations appear here]*
- ─────────────
- Save Current Configuration
- Delete Configuration...

### 4. View
- Theme: Light / Dark (toggle)
- ─────────────
- Waterfall Colors...
- ─────────────
- Zoom In (`Cmd+=`)
- Zoom Out (`Cmd+-`)
- Reset Zoom (`Cmd+0`)

### 5. Help
- Documentation
- ─────────────
- About PSK-31

## Files to Modify

### 1. `src-tauri/src/lib.rs`
- Add menu imports from `tauri::menu`
- Add `.setup()` hook with menu creation
- Add `on_menu_event()` handler
- Store menu handle in AppState for dynamic updates

### 2. `src-tauri/src/state.rs` (or new `src-tauri/src/config.rs`)
- Add Configuration struct (audio, serial, radio, modem settings)
- Add ConfigurationManager to load/save/list configurations
- Store configurations as JSON files in app data directory

### 3. `src/main.ts`
- Add event listeners for menu events
- Handle settings dialog, theme toggle, configuration switching

## Implementation Steps

### Step 1: Create Configuration data model

```rust
// src-tauri/src/domain/config.rs
#[derive(Serialize, Deserialize, Clone)]
pub struct Configuration {
    pub name: String,
    pub audio_input: Option<String>,
    pub audio_output: Option<String>,
    pub serial_port: Option<String>,
    pub baud_rate: u32,
    pub radio_type: String,  // "FT-991A", etc.
    pub carrier_freq: f64,
}
```

### Step 2: Add menu setup to lib.rs

- Create static menus (File, View, Help)
- Create dynamic Configurations menu (rebuilt when configs change)
- Store menu handle for runtime updates

### Step 3: Add Tauri commands for configuration management

- `list_configurations` → Vec<String>
- `get_configuration(name)` → Configuration
- `save_configuration(config)` → ()
- `delete_configuration(name)` → ()
- `set_active_configuration(name)` → ()

### Step 4: Add frontend event handlers

- Settings dialog trigger
- Configuration switch handler
- Theme toggle integration

### Step 5: Rebuild Configurations menu on changes

When configurations are added/deleted/renamed, rebuild the menu dynamically.

## Event Flow

```
Menu Click → Rust on_menu_event() → app.emit("menu:...") → Frontend listen() → UI Action
                                  ↓
                      (for config switch)
                      Update AppState → Rebuild menu
```

## Verification

1. Run `npm run tauri dev`
2. Verify menu bar appears with all 5 menus
3. Open Settings from PSK-31 menu (Cmd+,)
4. Configurations menu shows "Default" with checkmark
5. Theme toggle works from View menu
6. Quit/Exit closes the application
7. Keyboard shortcuts function correctly

## Phase 1.5 Scope (Menu Shell Only)

For now, implement the menu **structure** with placeholder actions:
- Menus appear and are clickable
- Events emit to frontend (logged to console)
- Settings dialog: show alert "Settings dialog coming soon"
- Configuration switching: log to console
- Theme toggle: wire to existing theme system

Actual settings dialog and configuration persistence come in later phases.

## Notes

- No changes needed to `Cargo.toml` - menu API included in Tauri 2.x
- Configurations stored in: `~/.config/psk31-client/configurations/` (Linux), `~/Library/Application Support/psk31-client/configurations/` (macOS)
- On Windows/Linux, the "PSK-31" app menu items merge into File menu
