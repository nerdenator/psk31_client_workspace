//! FT-991A radio adapter using CAT (Computer Aided Transceiver) commands
//!
//! The FT-991A speaks a simple text protocol over serial:
//! - Send a command like `FA;` (get frequency)
//! - Radio replies like `FA00014070000;` (frequency in Hz, zero-padded to 11 digits)
//! - All commands/responses are terminated with `;`
//!
//! Think of this like a Python class that owns a serial connection and
//! translates high-level methods (get_frequency) into low-level serial I/O.

use std::time::{Duration, Instant};

use crate::domain::{Frequency, Psk31Error, Psk31Result};
use crate::ports::{RadioControl, SerialConnection};

/// Minimum delay between CAT commands (FT-991A firmware requirement)
const COMMAND_DELAY_MS: u64 = 50;

/// Size of the read buffer for CAT responses
const READ_BUF_SIZE: usize = 64;

/// Max number of read attempts before giving up (each is ~100ms timeout)
const RESPONSE_TIMEOUT_READS: usize = 5;

/// US amateur radio bands (FCC Part 97) as (low_hz, high_hz) pairs.
/// Only frequencies within these bands are accepted.
const AMATEUR_BANDS_HZ: &[(u64, u64)] = &[
    (1_800_000, 2_000_000),       // 160m
    (3_500_000, 4_000_000),       // 80m
    (5_332_000, 5_405_000),       // 60m
    (7_000_000, 7_300_000),       // 40m
    (10_100_000, 10_150_000),     // 30m
    (14_000_000, 14_350_000),     // 20m
    (18_068_000, 18_168_000),     // 17m
    (21_000_000, 21_450_000),     // 15m
    (24_890_000, 24_990_000),     // 12m
    (28_000_000, 29_700_000),     // 10m
    (50_000_000, 54_000_000),     // 6m
    (144_000_000, 148_000_000),   // 2m
    (420_000_000, 450_000_000),   // 70cm
];

/// Check if a frequency falls within a US amateur band.
fn is_amateur_frequency(hz: u64) -> bool {
    AMATEUR_BANDS_HZ.iter().any(|&(lo, hi)| hz >= lo && hz <= hi)
}

/// Single source of truth for FT-991A mode code ↔ name mapping.
/// Each entry is (CAT code, human-readable name).
const MODE_TABLE: &[(&str, &str)] = &[
    ("1", "LSB"),
    ("2", "USB"),
    ("3", "CW"),
    ("4", "FM"),
    ("5", "AM"),
    ("6", "RTTY-LSB"),
    ("7", "CW-R"),
    ("8", "DATA-LSB"),
    ("9", "RTTY-USB"),
    ("A", "DATA-FM"),
    ("B", "FM-N"),
    ("C", "DATA-USB"),
    ("D", "AM-N"),
    ("E", "C4FM"),
];

/// FT-991A radio adapter. Owns the serial connection and tracks state.
pub struct Ft991aRadio {
    serial: Box<dyn SerialConnection>,
    is_transmitting: bool,
    last_command_time: Option<Instant>,
}

impl Ft991aRadio {
    pub fn new(serial: Box<dyn SerialConnection>) -> Self {
        Self {
            serial,
            is_transmitting: false,
            last_command_time: None,
        }
    }

    /// Send a CAT command and return the response string (up to the `;` terminator).
    /// Enforces the 50ms minimum delay between commands.
    ///
    /// Reads in a loop until the `;` terminator is found, because a single
    /// serial read may return a partial response on slow USB-serial adapters.
    fn send_command(&mut self, cmd: &str) -> Psk31Result<String> {
        self.ensure_command_delay();

        // Write the command
        self.serial
            .write(cmd.as_bytes())
            .map_err(|e| Psk31Error::Cat(format!("Command '{cmd}' write failed: {e}")))?;

        // Read until we see the `;` terminator or hit the overall timeout.
        // Each individual serial.read() has a 100ms timeout, so we loop up to
        // RESPONSE_TIMEOUT_READS times to allow ~500ms total for slow adapters.
        let mut buf = [0u8; READ_BUF_SIZE];
        let mut total = 0;

        for _ in 0..RESPONSE_TIMEOUT_READS {
            match self.serial.read(&mut buf[total..]) {
                Ok(n) if n > 0 => {
                    total += n;
                    if buf[..total].contains(&b';') {
                        break;
                    }
                }
                Ok(_) => {} // Zero bytes — read timed out, try again
                Err(e) => {
                    // Timeout errors are expected between chunks; only
                    // propagate if we haven't received anything yet.
                    if total == 0 {
                        return Err(Psk31Error::Cat(format!(
                            "Command '{cmd}' read failed: {e}"
                        )));
                    }
                }
            }
        }

        if total == 0 {
            return Err(Psk31Error::Cat(format!(
                "Command '{cmd}': no response from radio"
            )));
        }

        self.last_command_time = Some(Instant::now());

        let response = std::str::from_utf8(&buf[..total])
            .map_err(|e| Psk31Error::Cat(format!("Invalid UTF-8 response: {e}")))?;

        // Strip command echo if present (some radios echo the sent command back)
        // e.g. sending "FA;" may return "FA;FA00014070000;" — we want just the response.
        let response = response.strip_prefix(cmd).unwrap_or(response);

        Ok(response.to_string())
    }

    /// Sleep if needed to maintain the minimum inter-command delay.
    fn ensure_command_delay(&self) {
        if let Some(last) = self.last_command_time {
            let elapsed = last.elapsed();
            let min_delay = Duration::from_millis(COMMAND_DELAY_MS);
            if elapsed < min_delay {
                std::thread::sleep(min_delay - elapsed);
            }
        }
    }

    /// Parse a frequency response like `FA00014070000;` → Hz as f64
    fn parse_frequency(response: &str) -> Psk31Result<f64> {
        // Response format: FA followed by 11-digit Hz, then ;
        // e.g. "FA00014070000;" → 14_070_000 Hz
        let trimmed = response.trim().trim_end_matches(';');
        if !trimmed.starts_with("FA") || trimmed.len() < 13 {
            return Err(Psk31Error::Cat(format!(
                "Invalid frequency response: '{response}'"
            )));
        }
        let digits = &trimmed[2..13];
        let hz = digits
            .parse::<u64>()
            .map_err(|e| Psk31Error::Cat(format!("Failed to parse frequency '{digits}': {e}")))?;

        if !is_amateur_frequency(hz) {
            return Err(Psk31Error::Cat(format!(
                "Frequency {hz} Hz is outside US amateur bands"
            )));
        }

        Ok(hz as f64)
    }

    /// Parse a mode response like `MD0C;` → mode name string
    fn parse_mode(response: &str) -> Psk31Result<String> {
        // Response format: MD0 followed by mode code, then ;
        // e.g. "MD0C;" → DATA-USB
        let trimmed = response.trim().trim_end_matches(';');
        if !trimmed.starts_with("MD0") || trimmed.len() < 4 {
            return Err(Psk31Error::Cat(format!(
                "Invalid mode response: '{response}'"
            )));
        }
        let code = &trimmed[3..4];
        MODE_TABLE
            .iter()
            .find(|(c, _)| *c == code)
            .map(|(_, name)| name.to_string())
            .ok_or_else(|| Psk31Error::Cat(format!("Unknown mode code: '{code}'")))
    }

    /// Build a mode code from a mode name (reverse of parse_mode)
    fn mode_to_code(mode: &str) -> Psk31Result<&'static str> {
        MODE_TABLE
            .iter()
            .find(|(_, name)| *name == mode)
            .map(|(code, _)| *code)
            .ok_or_else(|| Psk31Error::Cat(format!("Unknown mode name: '{mode}'")))
    }
}

impl RadioControl for Ft991aRadio {
    fn ptt_on(&mut self) -> Psk31Result<()> {
        self.send_command("TX1;")?;
        self.is_transmitting = true;
        Ok(())
    }

    fn ptt_off(&mut self) -> Psk31Result<()> {
        self.send_command("TX0;")?;
        self.is_transmitting = false;
        Ok(())
    }

    fn is_transmitting(&self) -> bool {
        self.is_transmitting
    }

    fn get_frequency(&mut self) -> Psk31Result<Frequency> {
        let response = self.send_command("FA;")?;
        let hz = Self::parse_frequency(&response)?;
        Ok(Frequency::hz(hz))
    }

    fn set_frequency(&mut self, freq: Frequency) -> Psk31Result<()> {
        let hz = freq.as_hz() as u64;
        let cmd = format!("FA{hz:011};");
        self.send_command(&cmd)?;
        Ok(())
    }

    fn get_mode(&mut self) -> Psk31Result<String> {
        let response = self.send_command("MD0;")?;
        Self::parse_mode(&response)
    }

    fn set_mode(&mut self, mode: &str) -> Psk31Result<()> {
        let code = Self::mode_to_code(mode)?;
        let cmd = format!("MD0{code};");
        self.send_command(&cmd)?;
        Ok(())
    }
}

/// Safety: auto-release PTT if the radio is dropped while transmitting.
/// This prevents leaving the radio keyed up if the app crashes or disconnects.
/// Retries up to 3 times with increasing delays in case the first attempt
/// fails (e.g. USB adapter momentarily busy).
impl Drop for Ft991aRadio {
    fn drop(&mut self) {
        if self.is_transmitting {
            for delay_ms in [0, 10, 50] {
                if delay_ms > 0 {
                    std::thread::sleep(Duration::from_millis(delay_ms));
                }
                if self.ptt_off().is_ok() {
                    return;
                }
            }
            eprintln!("CRITICAL: Failed to release PTT on drop. Radio may still be transmitting!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_frequency_valid() {
        let hz = Ft991aRadio::parse_frequency("FA00014070000;").unwrap();
        assert_eq!(hz, 14_070_000.0);
    }

    #[test]
    fn parse_frequency_seven_mhz() {
        let hz = Ft991aRadio::parse_frequency("FA00007074000;").unwrap();
        assert_eq!(hz, 7_074_000.0);
    }

    #[test]
    fn parse_frequency_invalid_prefix() {
        assert!(Ft991aRadio::parse_frequency("FB00014070000;").is_err());
    }

    #[test]
    fn parse_frequency_too_short() {
        assert!(Ft991aRadio::parse_frequency("FA123;").is_err());
    }

    #[test]
    fn parse_frequency_rejects_non_amateur() {
        // Broadcast AM (1 MHz) — not an amateur band
        assert!(Ft991aRadio::parse_frequency("FA00001000000;").is_err());
        // 0 Hz — nonsense
        assert!(Ft991aRadio::parse_frequency("FA00000000000;").is_err());
        // Between 80m and 60m bands (5 MHz)
        assert!(Ft991aRadio::parse_frequency("FA00005000000;").is_err());
    }

    #[test]
    fn parse_frequency_accepts_amateur_bands() {
        // 20m PSK-31 calling frequency
        assert!(Ft991aRadio::parse_frequency("FA00014070000;").is_ok());
        // 2m FM simplex
        assert!(Ft991aRadio::parse_frequency("FA00146520000;").is_ok());
        // 70cm bottom edge
        assert!(Ft991aRadio::parse_frequency("FA00420000000;").is_ok());
        // 160m bottom edge
        assert!(Ft991aRadio::parse_frequency("FA00001800000;").is_ok());
    }

    #[test]
    fn parse_mode_data_usb() {
        let mode = Ft991aRadio::parse_mode("MD0C;").unwrap();
        assert_eq!(mode, "DATA-USB");
    }

    #[test]
    fn parse_mode_usb() {
        let mode = Ft991aRadio::parse_mode("MD02;").unwrap();
        assert_eq!(mode, "USB");
    }

    #[test]
    fn parse_mode_lsb() {
        let mode = Ft991aRadio::parse_mode("MD01;").unwrap();
        assert_eq!(mode, "LSB");
    }

    #[test]
    fn parse_mode_invalid() {
        assert!(Ft991aRadio::parse_mode("MD0Z;").is_err());
    }

    #[test]
    fn parse_mode_too_short() {
        assert!(Ft991aRadio::parse_mode("MD;").is_err());
    }

    #[test]
    fn mode_roundtrip() {
        let modes = [
            "LSB", "USB", "CW", "FM", "AM", "RTTY-LSB", "CW-R", "DATA-LSB", "RTTY-USB",
            "DATA-FM", "FM-N", "DATA-USB", "AM-N", "C4FM",
        ];
        for mode in modes {
            let code = Ft991aRadio::mode_to_code(mode).unwrap();
            let response = format!("MD0{code};");
            let parsed = Ft991aRadio::parse_mode(&response).unwrap();
            assert_eq!(parsed, mode, "Roundtrip failed for mode '{mode}'");
        }
    }
}
