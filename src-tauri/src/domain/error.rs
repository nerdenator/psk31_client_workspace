//! Domain error types

use thiserror::Error;

/// Errors that can occur in the PSK-31 application
#[derive(Error, Debug)]
pub enum Psk31Error {
    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Serial port error: {0}")]
    Serial(String),

    #[error("CAT command error: {0}")]
    Cat(String),

    #[error("Modem error: {0}")]
    Modem(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

/// Result type alias for PSK-31 operations
pub type Psk31Result<T> = Result<T, Psk31Error>;
