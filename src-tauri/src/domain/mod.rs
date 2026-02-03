//! Core domain types
//!
//! Pure types with no I/O dependencies. These represent the core concepts
//! of the PSK-31 application.

pub mod types;
pub mod error;

pub use types::*;
pub use error::*;
