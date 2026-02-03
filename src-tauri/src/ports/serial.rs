//! Serial port trait

use crate::domain::{Psk31Result, SerialPortInfo};

/// Trait for serial port communication
pub trait SerialConnection: Send + Sync {
    /// List available serial ports
    fn list_ports() -> Psk31Result<Vec<SerialPortInfo>>
    where
        Self: Sized;

    /// Open a connection to a serial port
    fn open(port: &str, baud_rate: u32) -> Psk31Result<Self>
    where
        Self: Sized;

    /// Write bytes to the port
    fn write(&mut self, data: &[u8]) -> Psk31Result<usize>;

    /// Read bytes from the port (with timeout)
    fn read(&mut self, buffer: &mut [u8]) -> Psk31Result<usize>;

    /// Write a string and read the response (convenience for CAT commands)
    fn write_read(&mut self, command: &str, response_buf: &mut [u8]) -> Psk31Result<usize> {
        self.write(command.as_bytes())?;
        self.read(response_buf)
    }

    /// Close the connection
    fn close(&mut self) -> Psk31Result<()>;

    /// Check if connected
    fn is_connected(&self) -> bool;
}
