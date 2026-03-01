//! Port traits (interfaces)
//!
//! These traits define the boundaries between the core domain and external I/O.
//! Adapters implement these traits to connect to real hardware.

pub mod audio;
pub mod serial;
pub mod radio;

pub use audio::*;
pub use serial::*;
pub use radio::*;
