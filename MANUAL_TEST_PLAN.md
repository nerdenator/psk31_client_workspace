# Baudacious — Manual Test Plan (Radio Hardware)

**Radio:** Yaesu FT-991A
**Date:** 2026-03-15

---

## Pre-Flight Checklist

- [x] FT-991A powered on
- [x] USB cable connected (provides both CAT serial and audio)
- [x] Radio set to a quiet frequency (not the PSK-31 calling freq — use it for RX tests later)
- [x] RF power set low (5–10W) for initial TX tests
- [x] Antenna connected or dummy load in place

---

## 1. Serial / CAT

**Goal:** App connects to radio and reads state correctly.

- [x] Launch `npm run tauri dev`
- [x] Open Settings → Radio tab; set baud rate to 38400
- [x] Select the FT-991A serial port from the dropdown
- [x] Click Connect — button should turn green briefly
- [x] **Verify:** Band selector pre-fills with the correct band
- [x] **Verify:** Frequency input shows the radio's actual VFO frequency
- [x] Change band in the app — **verify** radio changes frequency (CAT TX)
- [x] Change frequency on the radio manually — **verify** app updates within 2s (polling added 2026-03-15)
- [X] Type a frequency in the input and press Enter — **verify** radio QSYs

---

## 2. Audio Setup

**Goal:** Audio devices are selected and waterfall is live.

- [x] Open Settings → Audio tab
- [x] Select FT-991A USB Audio as both input and output device
- [x] Click Save & Apply
- [x] Click Start Audio (or equivalent)
- [x] **Verify:** Waterfall is black (no signal) or showing noise floor
- [ ] Key the radio manually (MON on) — **verify** waterfall shows activity
- [x] **Verify:** Status bar shows audio device name

---

## 3. RX — Receive PSK-31

**Goal:** App decodes real PSK-31 signals off the air.

- [ ] QSY to 14.070.000 MHz (20m PSK-31 calling freq) — use band selector
- [ ] **Verify:** Waterfall shows PSK-31 signal traces (vertical streaks)
- [ ] Click a signal trace in the waterfall — **verify** carrier marker moves to that signal
- [ ] **Verify:** Decoded text appears in the RX pane
- [ ] Let it run for 30+ seconds — **verify** text continues to accumulate
- [ ] Click the Clear button — **verify** RX pane clears
- [ ] Try 7.070.000 MHz (40m, LSB) if 20m is quiet
- [ ] **Verify:** No garbage characters during band noise (squelch working)

---

## 4. TX — Transmit PSK-31

**Goal:** App keys radio, transmits valid PSK-31, releases PTT cleanly.

> Start with a dummy load or confirm the frequency is clear before transmitting.

### 4a. Basic TX

- [ ] Type "TEST DE [YOUR CALL]" in the TX pane
- [ ] Click Transmit (or press the TX key)
- [ ] **Verify:** Radio keys (TX light on, ALC shows activity)
- [ ] **Verify:** Status bar shows "transmitting"
- [ ] **Verify:** Audio output is audible through the radio monitor
- [ ] **Verify:** PTT releases cleanly after transmission completes
- [ ] **Verify:** Status bar returns to idle

### 4b. Abort TX

- [ ] Type a long message (20+ words)
- [ ] Click Transmit
- [ ] While transmitting, click Abort (or Stop TX)
- [ ] **Verify:** Radio unkeys immediately
- [ ] **Verify:** Status bar shows "aborted" then returns to idle
- [ ] **Verify:** PTT is NOT latched (radio is in RX)

### 4c. TX → RX Loopback (on-air verify)

If you have a second receiver or SDR available:
- [ ] Tune second receiver to same frequency
- [ ] Transmit "TEST" from Baudacious
- [ ] **Verify:** Second receiver decodes the text correctly
- [ ] **Verify:** Signal looks clean on waterfall (narrow, no splatter)

---

## 5. Error Handling

**Goal:** App recovers gracefully from hardware disconnects.

### 5a. Serial disconnect

- [ ] Connect to radio successfully
- [ ] Unplug USB cable mid-session
- [ ] **Verify:** Toast notification appears ("serial disconnected" or similar)
- [ ] **Verify:** Serial panel resets to disconnected state
- [ ] Reconnect USB — **verify** app can reconnect without restart

### 5b. Audio disconnect

- [ ] Start audio successfully
- [ ] Unplug USB audio (or disable device in OS)
- [ ] **Verify:** Toast notification appears ("audio device lost")
- [ ] **Verify:** Audio panel resets
- [ ] Use the refresh button (↻) — **verify** device list updates
- [ ] Reconnect and restart audio — **verify** waterfall resumes

### 5c. TX with no radio connected

- [ ] Disconnect serial (leave audio running)
- [ ] Attempt to transmit
- [ ] **Verify:** TX proceeds (audio-only mode — PTT skipped, no error)
- [ ] **Verify:** No crash, no latched PTT state

---

## 6. Settings Persistence

- [x] Change waterfall palette, noise floor, zoom
- [x] Quit and relaunch app
- [x] **Verify:** All settings restored correctly
- [x] **Verify:** Last-used audio devices pre-selected

---

## Notes

Record any unexpected behavior here during the session.

| # | Symptom | Repro steps | Severity |
|---|---------|-------------|----------|
|   |         |             |          |
