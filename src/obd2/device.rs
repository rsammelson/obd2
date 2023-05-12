use log::{debug, info, trace};
use std::{
    collections::VecDeque,
    io::{Read, Write},
    thread, time,
};

type Result<T> = std::result::Result<T, Error>;

pub struct Obd2Basic {
    device: ftdi::Device,
    buffer: VecDeque<u8>,
}

impl Default for Obd2Basic {
    fn default() -> Self {
        Obd2Basic::new().unwrap()
    }
}

impl Obd2Basic {
    fn new() -> Result<Self> {
        let mut ftdi_device = ftdi::find_by_vid_pid(0x0403, 0x6001)
            .interface(ftdi::Interface::A)
            .open()?;

        ftdi_device.set_baud_rate(38400)?;
        ftdi_device.configure(ftdi::Bits::Eight, ftdi::StopBits::One, ftdi::Parity::None)?;
        // device.set_latency_timer(2).unwrap();

        ftdi_device.usb_reset()?;

        let mut device = Obd2Basic {
            device: ftdi_device,
            buffer: VecDeque::new(),
        };

        device.connect()?;

        Ok(device)
    }

    fn connect(&mut self) -> Result<()> {
        self.flush_buffers()?;
        thread::sleep(time::Duration::from_millis(500));
        self.send_serial_str(" ")?;
        thread::sleep(time::Duration::from_millis(500));

        self.reset()?;

        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        self.flush_buffers()?;
        self.reset_ic()?;
        thread::sleep(time::Duration::from_millis(500));
        self.reset_protocol()?;
        Ok(())
    }

    pub fn cmd(&mut self, cmd: &str) -> Result<Option<String>> {
        self.send_serial_cmd(cmd)?;
        self.get_until_prompt()
            .map(|o| o.and_then(|resp| String::from_utf8(resp).ok()))
    }

    fn reset_ic(&mut self) -> Result<()> {
        info!("Performing IC reset");
        self.send_serial_cmd("atz")?;
        debug!(
            "reset_ic: got response {:?}",
            self.get_until_prompt()?
                .as_ref()
                .map(|l| std::str::from_utf8(l.as_slice()))
        );
        Ok(())
    }

    fn reset_protocol(&mut self) -> Result<()> {
        info!("Performing protocol reset");
        debug!("reset_protocol: got response {:?}", self.cmd("atsp0")?);
        debug!("reset_protocol: got response {:?}", self.cmd("0100")?);
        self.flush_buffers()?;
        Ok(())
    }

    pub fn get_line(&mut self) -> Result<Option<Vec<u8>>> {
        self.get_until(b'\r')
    }

    pub fn get_until_prompt(&mut self) -> Result<Option<Vec<u8>>> {
        self.get_until(b'>')
    }

    fn get_until(&mut self, end_byte: u8) -> Result<Option<Vec<u8>>> {
        const TIMEOUT: time::Duration = time::Duration::from_secs(5);

        trace!("get_until: getting until {}", end_byte);

        let mut buf = Vec::new();
        let start = time::Instant::now();
        while start.elapsed() < TIMEOUT {
            let Some(b) = self.get_byte()? else { continue };
            match b {
                b'\r' => {
                    buf.push(b'\n');
                }
                b'\n' => {}
                _ => buf.push(b),
            }
            if b == end_byte {
                break;
            }
        }

        trace!(
            "get_until: got {:?} ({:?})",
            buf,
            std::str::from_utf8(buf.as_slice())
        );

        match buf.last() {
            Some(b) if b == &end_byte => Ok(Some(buf)), // we got it
            Some(_) => {
                // incomplete line read
                for b in buf.iter().rev() {
                    self.buffer.push_front(*b);
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    fn get_byte(&mut self) -> Result<Option<u8>> {
        self.read_into_queue()?;
        loop {
            let b = self.buffer.pop_front();
            if b != Some(b'\0') {
                return Ok(b);
            }
        }
    }

    fn flush_buffers(&mut self) -> Result<()> {
        self.device.usb_purge_buffers()?;
        Ok(())
    }

    pub fn send_serial_cmd(&mut self, data: &str) -> Result<()> {
        self.device.write_all(data.as_bytes())?;
        self.device.write_all(b"\r\n")?;
        Ok(())
    }

    fn send_serial_str(&mut self, data: &str) -> Result<()> {
        self.device.write_all(data.as_bytes())?;
        Ok(())
    }

    fn read_into_queue(&mut self) -> Result<()> {
        let mut buf = [0u8; 16];
        loop {
            let len = self.device.read(&mut buf)?;
            self.buffer.extend(&buf[0..len]);
            trace!(
                "read_into_queue: values {:?}",
                std::str::from_utf8(&buf[0..len])
            );
            if len == 0 {
                break;
            }
        }
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("FTDI error: `{0:?}`")]
    Ftdi(ftdi::Error),
    #[error("IO error: `{0:?}`")]
    IO(std::io::Error),
}

impl From<ftdi::Error> for Error {
    fn from(e: ftdi::Error) -> Self {
        Error::Ftdi(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}
