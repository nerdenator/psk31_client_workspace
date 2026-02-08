//! Configuration profiles
//!
//! A Configuration is a saved profile containing all settings for a particular
//! radio setup (audio devices, serial port, radio type, modem parameters).

use serde::{Deserialize, Serialize};

/// A saved configuration profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    /// Profile name (e.g., "FT-991A Home", "IC-7300 Portable")
    pub name: String,
    /// Selected audio input device ID
    pub audio_input: Option<String>,
    /// Selected audio output device ID
    pub audio_output: Option<String>,
    /// Selected serial port name
    pub serial_port: Option<String>,
    /// Serial baud rate
    pub baud_rate: u32,
    /// Radio type identifier (e.g., "FT-991A", "IC-7300")
    pub radio_type: String,
    /// Audio carrier frequency in Hz
    pub carrier_freq: f64,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            audio_input: None,
            audio_output: None,
            serial_port: None,
            baud_rate: 38400,
            radio_type: "FT-991A".to_string(),
            carrier_freq: 1000.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_configuration_has_sensible_values() {
        let config = Configuration::default();
        assert_eq!(config.name, "Default");
        assert_eq!(config.baud_rate, 38400);
        assert_eq!(config.carrier_freq, 1000.0);
    }

    #[test]
    fn configuration_serializes_to_json() {
        let config = Configuration::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"name\":\"Default\""));
    }
}
