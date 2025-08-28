use super::Error;

#[cfg(any(feature = "serialport_comm", feature = "ftdi_comm"))]
pub const DEFAULT_BAUD_RATE: u32 = 38_400;

/// An API to communicate with a serial device
pub trait SerialCommunication: std::io::Read + std::io::Write {
    /// Get the baud rate of the device
    fn get_baud_rate(&mut self) -> Result<u32, Error>;

    /// Set the baud rate of the device
    fn set_baud_rate(&mut self, baud_rate: u32) -> Result<(), Error>;
}
