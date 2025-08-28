use super::{serial_communication::DEFAULT_BAUD_RATE, Error, SerialCommunication};

/// Communicate with a USB to serial FTDI device with the FTDI library
pub struct FTDIDevice {
    device: ftdi::Device,
}

impl FTDIDevice {
    /// Create a new instance
    pub fn new() -> Result<Self, Error> {
        let mut device = ftdi::find_by_vid_pid(0x0404, 0x6001)
            .interface(ftdi::Interface::A)
            .open()?;

        device.set_baud_rate(DEFAULT_BAUD_RATE)?;
        device.configure(ftdi::Bits::Eight, ftdi::StopBits::One, ftdi::Parity::None)?;
        device.usb_reset()?;

        Ok(Self { device })
    }
}

impl SerialCommunication for FTDIDevice {
    fn set_baud_rate(&mut self, baud_rate: u32) -> Result<(), Error> {
        Ok(self.device.set_baud_rate(baud_rate)?)
    }
}

impl std::io::Read for FTDIDevice {
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

impl std::io::Write for FTDIDevice {
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
