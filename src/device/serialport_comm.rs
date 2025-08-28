use std::time::Duration;

use super::{serial_communication::DEFAULT_BAUD_RATE, Error, SerialCommunication};

/// Communicate with a serial device using the [serialport] library
///
/// /dev/tty* or similar on Unix-like systems or COM devices on Windows systems
pub struct SerialPort {
    device: Box<dyn serialport::SerialPort>,
}

impl SerialPort {
    /// Create a new instance from the path to the device
    pub fn new(path: String) -> Result<SerialPort, Error> {
        let device = serialport::new(path, DEFAULT_BAUD_RATE)
            .timeout(Duration::from_millis(10))
            .parity(serialport::Parity::None)
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One)
            .open()?;

        Ok(Self { device })
    }
}

impl SerialCommunication for SerialPort {
    fn set_baud_rate(&mut self, baud_rate: u32) -> Result<(), Error> {
        Ok(self.device.set_baud_rate(baud_rate)?)
    }
}

impl std::io::Read for SerialPort {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.device.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        self.device.read_vectored(bufs)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.device.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.device.read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.device.read_exact(buf)
    }
}

impl std::io::Write for SerialPort {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.device.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.device.flush()
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.device.write_vectored(bufs)
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.device.write_all(buf)
    }

    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.device.write_fmt(args)
    }
}
