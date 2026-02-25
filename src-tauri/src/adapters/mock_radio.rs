//! Mock radio adapter for development and testing without hardware.
//!
//! Activate by setting MOCK_RADIO=1 in the environment:
//!
//!   MOCK_RADIO=1 RUST_LOG=baudacious_lib=info npm run tauri dev
//!
//! Every RadioControl call is logged at INFO level so you can verify
//! exactly what the UI would send to a real radio.

use crate::domain::{Frequency, Psk31Result};
use crate::ports::RadioControl;

/// Default frequency: 20m PSK-31 calling frequency
const DEFAULT_FREQ_HZ: f64 = 14_070_000.0;
/// Default mode: DATA-USB (standard for PSK-31)
const DEFAULT_MODE: &str = "DATA-USB";

pub struct MockRadio {
    frequency: f64,
    mode: String,
    is_transmitting: bool,
}

impl MockRadio {
    pub fn new() -> Self {
        log::info!("[MOCK RADIO] Initialized at {:.3} MHz, mode={DEFAULT_MODE}", DEFAULT_FREQ_HZ / 1e6);
        Self {
            frequency: DEFAULT_FREQ_HZ,
            mode: DEFAULT_MODE.to_string(),
            is_transmitting: false,
        }
    }
}

impl RadioControl for MockRadio {
    fn ptt_on(&mut self) -> Psk31Result<()> {
        self.is_transmitting = true;
        log::info!("[MOCK RADIO] PTT ON  → TX1;");
        Ok(())
    }

    fn ptt_off(&mut self) -> Psk31Result<()> {
        self.is_transmitting = false;
        log::info!("[MOCK RADIO] PTT OFF → TX0;");
        Ok(())
    }

    fn is_transmitting(&self) -> bool {
        self.is_transmitting
    }

    fn get_frequency(&mut self) -> Psk31Result<Frequency> {
        let hz = self.frequency as u64;
        log::info!("[MOCK RADIO] GET FREQ → FA; → FA{hz:011};  ({:.3} MHz)", self.frequency / 1e6);
        Ok(Frequency::hz(self.frequency))
    }

    fn set_frequency(&mut self, freq: Frequency) -> Psk31Result<()> {
        let hz = freq.as_hz() as u64;
        log::info!("[MOCK RADIO] SET FREQ → FA{hz:011};  ({:.3} MHz)", freq.as_hz() / 1e6);
        self.frequency = freq.as_hz();
        Ok(())
    }

    fn get_mode(&mut self) -> Psk31Result<String> {
        log::info!("[MOCK RADIO] GET MODE → MD0; → {}",  self.mode);
        Ok(self.mode.clone())
    }

    fn set_mode(&mut self, mode: &str) -> Psk31Result<()> {
        log::info!("[MOCK RADIO] SET MODE → MD0?; → {mode}");
        self.mode = mode.to_string();
        Ok(())
    }
}
