use super::Result;

#[cfg(any(feature = "serialport_comm", feature = "ftdi_comm"))]
pub const DEFAULT_BAUD_RATE: u32 = 38_400;

/// An API to communicate with a serial device
pub trait SerialComm {
    fn write_all(&mut self, data: &[u8]) -> Result<()>;
    fn read(&mut self, data: &mut [u8]) -> Result<usize>;
    fn set_baud_rate(&mut self, baud_rate: u32) -> Result<()>;
    fn purge_buffers(&mut self) -> Result<()>;
}
