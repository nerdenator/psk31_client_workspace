//! CatSession: owns a serial connection and drives CAT I/O timing.
//!
//! This is analogous to a Python class that wraps a serial.Serial object and
//! adds the FT-991A protocol layer: command delay, read-until-semicolon loop,
//! echo stripping, and error detection.
//!
//! Pure translation lives in `encode` / `decode`. CatSession only handles I/O.

use std::time::{Duration, Instant};

use crate::domain::{Psk31Error, Psk31Result};
use crate::ports::SerialConnection;

use super::{decode, encode, CatCommand, CatResponse};

/// Minimum delay between CAT commands (FT-991A firmware requirement)
const COMMAND_DELAY_MS: u64 = 50;

/// Read buffer size for CAT responses
const READ_BUF_SIZE: usize = 64;

/// Max read attempts before giving up (~100ms per attempt → ~500ms total)
const RESPONSE_TIMEOUT_READS: usize = 5;

/// Owns a serial connection and executes CAT commands against the FT-991A.
pub struct CatSession {
    serial: Box<dyn SerialConnection>,
    last_command_time: Option<Instant>,
}

impl CatSession {
    pub fn new(serial: Box<dyn SerialConnection>) -> Self {
        Self {
            serial,
            last_command_time: None,
        }
    }

    /// Send a CAT command and return the parsed response.
    ///
    /// Enforces the 50ms inter-command delay, writes the wire string,
    /// reads bytes until the `;` terminator, strips any command echo,
    /// then delegates to `decode()`.
    pub fn execute(&mut self, cmd: &CatCommand) -> Psk31Result<CatResponse> {
        self.ensure_command_delay();

        let wire = encode(cmd);
        log::debug!("CAT TX: {wire}");

        self.serial
            .write(wire.as_bytes())
            .map_err(|e| Psk31Error::Cat(format!("Command '{wire}' write failed: {e}")))?;

        let raw = self.read_until_semicolon(&wire)?;
        self.last_command_time = Some(Instant::now());

        log::debug!("CAT RX: {raw}");

        // Strip command echo if present (some USB-serial adapters echo the TX)
        // e.g. "FA;FA00014070000;" → "FA00014070000;"
        let raw = raw.strip_prefix(&wire).unwrap_or(&raw);

        decode(raw, cmd)
    }

    /// Read bytes from the serial port until a `;` appears or timeout.
    ///
    /// Each serial.read() has a 100ms hardware timeout. We retry up to
    /// RESPONSE_TIMEOUT_READS times to handle slow USB-serial adapters
    /// that may return partial responses.
    fn read_until_semicolon(&mut self, cmd_wire: &str) -> Psk31Result<String> {
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
                Ok(_) => {} // Zero bytes: read timed out, try again
                Err(e) => {
                    // Timeout mid-response is fine; only propagate if nothing received yet
                    if total == 0 {
                        return Err(Psk31Error::Cat(format!(
                            "Command '{cmd_wire}' read failed: {e}"
                        )));
                    }
                }
            }
        }

        if total == 0 {
            return Err(Psk31Error::Cat(format!(
                "Command '{cmd_wire}': no response from radio"
            )));
        }

        std::str::from_utf8(&buf[..total])
            .map(|s| s.to_string())
            .map_err(|e| Psk31Error::Cat(format!("Invalid UTF-8 response: {e}")))
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::SerialConnection;
    use std::sync::{Arc, Mutex};

    // ---------------------------------------------------------------------------
    // MockSerial for CatSession tests
    // ---------------------------------------------------------------------------

    struct MockSerial {
        log: Arc<Mutex<Vec<String>>>,
        response: String,
    }

    impl SerialConnection for MockSerial {
        fn write(&mut self, data: &[u8]) -> Psk31Result<usize> {
            self.log
                .lock()
                .unwrap()
                .push(String::from_utf8_lossy(data).into());
            Ok(data.len())
        }
        fn read(&mut self, buf: &mut [u8]) -> Psk31Result<usize> {
            let bytes = self.response.as_bytes();
            let n = bytes.len().min(buf.len());
            buf[..n].copy_from_slice(&bytes[..n]);
            Ok(n)
        }
        fn close(&mut self) -> Psk31Result<()> {
            Ok(())
        }
        fn is_connected(&self) -> bool {
            true
        }
    }

    fn make_session(response: &str) -> (CatSession, Arc<Mutex<Vec<String>>>) {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mock = MockSerial {
            log: Arc::clone(&log),
            response: response.to_string(),
        };
        (CatSession::new(Box::new(mock)), log)
    }

    // --- Basic I/O ---

    #[test]
    fn execute_get_frequency_sends_fa_query() {
        let (mut session, log) = make_session("FA00014070000;");
        session.execute(&CatCommand::GetFrequencyA).unwrap();
        assert_eq!(log.lock().unwrap()[0], "FA;");
    }

    #[test]
    fn execute_ptt_on_sends_tx1() {
        let (mut session, log) = make_session(";");
        session.execute(&CatCommand::PttOn).unwrap();
        assert_eq!(log.lock().unwrap()[0], "TX1;");
    }

    #[test]
    fn execute_ptt_off_sends_tx0() {
        let (mut session, log) = make_session(";");
        session.execute(&CatCommand::PttOff).unwrap();
        assert_eq!(log.lock().unwrap()[0], "TX0;");
    }

    // --- NAK ---

    #[test]
    fn nak_response_returns_err() {
        let (mut session, _) = make_session("?");
        let result = session.execute(&CatCommand::GetFrequencyA);
        assert!(result.is_err(), "? response should be Err");
    }

    // --- Ack ---

    #[test]
    fn semicolon_response_returns_ack() {
        let (mut session, _) = make_session(";");
        let resp = session.execute(&CatCommand::PttOn).unwrap();
        assert_eq!(resp, CatResponse::Ack);
    }

    // --- Echo stripping ---

    #[test]
    fn echo_stripped_before_decode() {
        // Some adapters echo the sent command: "FA;FA00014070000;"
        let (mut session, _) = make_session("FA;FA00014070000;");
        let resp = session.execute(&CatCommand::GetFrequencyA).unwrap();
        assert_eq!(resp, CatResponse::FrequencyHz(14_070_000));
    }

    // --- TX power ---

    #[test]
    fn get_tx_power_returns_watts() {
        let (mut session, log) = make_session("PC025;");
        let resp = session.execute(&CatCommand::GetTxPower).unwrap();
        assert_eq!(resp, CatResponse::TxPower(25));
        assert_eq!(log.lock().unwrap()[0], "PC;");
    }

    #[test]
    fn set_tx_power_sends_correct_wire() {
        let (mut session, log) = make_session(";");
        session.execute(&CatCommand::SetTxPower(50)).unwrap();
        assert_eq!(log.lock().unwrap()[0], "PC050;");
    }
}
