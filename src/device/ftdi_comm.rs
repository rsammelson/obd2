use super::serial_comm::{SerialComm, DEFAULT_BAUD_RATE};
use super::Result;
use std::io::{Read, Write};

/// Communicate with a USB to Serial FTDI device
/// with the FTDI library
pub struct FTDIDevice {
    device: ftdi::Device,
}

impl FTDIDevice {
    /// Creates a new instance of an FTDIDevice
    pub fn new() -> Result<Self> {
        let mut device = ftdi::find_by_vid_pid(0x0404, 0x6001)
            .interface(ftdi::Interface::A)
            .open()?;

        device.set_baud_rate(DEFAULT_BAUD_RATE)?;
        device.configure(ftdi::Bits::Eight, ftdi::StopBits::One, ftdi::Parity::None)?;
        device.usb_reset()?;

        Ok(Self { device })
    }
}

impl SerialComm for FTDIDevice {
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
        Ok(self.device.usb_purge_buffers()?)
    }
}
