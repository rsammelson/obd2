use super::serial_comm::{SerialComm, DEFAULT_BAUD_RATE};
use super::Result;
use std::io::{Read, Write};
use std::time::Duration;

/// Communicate with a serial device using the
/// serialport library
///
/// /dev/tty* or similar on unix-like systems
/// COM devices on Windows systems
pub struct SerialPort {
    device: Box<dyn serialport::SerialPort>,
}

impl SerialPort {
    /// Creates a new instance of a SerialPort
    pub fn new(path: &str) -> Result<Self> {
        let device = serialport::new(path, DEFAULT_BAUD_RATE)
            .timeout(Duration::from_millis(10))
            .parity(serialport::Parity::None)
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One)
            .path(path)
            .open()?;

        Ok(Self { device })
    }
}

impl SerialComm for SerialPort {
    fn write_all(&mut self, data: &[u8]) -> Result<()> {
        Ok(self.device.write_all(data)?)
    }

    fn read(&mut self, data: &mut [u8]) -> Result<usize> {
        Ok(self.device.read(data)?)
    }

    fn set_baud_rate(&mut self, baud_rate: u32) -> Result<()> {
        Ok(self.device.set_baud_rate(baud_rate)?)
    }

    fn purge_buffers(&mut self) -> Result<()> {
        Ok(self.device.clear(serialport::ClearBuffer::All)?)
    }
}
