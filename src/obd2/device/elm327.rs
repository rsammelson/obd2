use log::{debug, info, trace};
use std::{
    collections::VecDeque,
    io::{Read, Write},
    thread, time,
};

use super::{Error, Obd2BaseDevice, Obd2Reader, Result};

pub struct Elm327 {
    device: ftdi::Device,
    buffer: VecDeque<u8>,
}

impl Default for Elm327 {
    fn default() -> Self {
        Elm327::new().unwrap()
    }
}

impl Obd2BaseDevice for Elm327 {
    fn reset(&mut self) -> Result<()> {
        self.flush_buffers()?;
        self.reset_ic()?;
        thread::sleep(time::Duration::from_millis(500));
        self.reset_protocol()?;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        thread::sleep(time::Duration::from_millis(500));
        self.read_into_queue()?;
        self.buffer.clear();
        thread::sleep(time::Duration::from_millis(500));
        Ok(())
    }

    fn send_serial_cmd(&mut self, data: &str) -> Result<()> {
        self.device.write_all(data.as_bytes())?;
        self.device.write_all(b"\r\n")?;
        let line = self.get_line()?;
        if line.as_ref().is_some_and(|v| v == data.as_bytes()) {
            Ok(())
        } else {
            Err(Error::Communication(format!(
                "send_serial_cmd: got {:?} instead of echoed command ({})",
                line, data
            )))
        }
    }
}

impl Obd2Reader for Elm327 {
    fn get_line(&mut self) -> Result<Option<Vec<u8>>> {
        let result = self.get_until(b'\n')?;
        let Some(mut line) = result else {
            return Ok(result);
        };
        if line.pop() == Some(b'\n') {
            Ok(Some(line))
        } else {
            Err(Error::Communication("get_line no line ending".to_owned()))
        }
    }

    fn get_until_prompt(&mut self) -> Result<Option<Vec<u8>>> {
        let result = self.get_until(b'>')?;
        let Some(mut line) = result else {
            return Ok(result);
        };
        if line.pop() == Some(b'>') {
            Ok(Some(line))
        } else {
            Err(Error::Communication(
                "get_until_prompt no ending".to_owned(),
            ))
        }
    }
}

impl Elm327 {
    fn new() -> Result<Self> {
        let mut ftdi_device = ftdi::find_by_vid_pid(0x0403, 0x6001)
            .interface(ftdi::Interface::A)
            .open()?;

        ftdi_device.set_baud_rate(38400)?;
        ftdi_device.configure(ftdi::Bits::Eight, ftdi::StopBits::One, ftdi::Parity::None)?;
        // device.set_latency_timer(2).unwrap();

        ftdi_device.usb_reset()?;

        let mut device = Elm327 {
            device: ftdi_device,
            buffer: VecDeque::new(),
        };

        device.connect()?;
        device.flush()?;

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

    fn get_until(&mut self, end_byte: u8) -> Result<Option<Vec<u8>>> {
        const TIMEOUT: time::Duration = time::Duration::from_secs(5);

        trace!("get_until: getting until {}", end_byte);

        let mut buf = Vec::new();
        let start = time::Instant::now();
        while start.elapsed() < TIMEOUT {
            let Some(b) = self.get_byte()? else { continue };
            let b = match b {
                b'\r' => {
                    buf.push(b'\n');
                    b'\n'
                }
                b'\n' => b, // no push here
                _ => {
                    buf.push(b);
                    b
                }
            };
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

    fn send_serial_str(&mut self, data: &str) -> Result<()> {
        self.device.write_all(data.as_bytes())?;
        Ok(())
    }

    fn read_into_queue(&mut self) -> Result<()> {
        let mut buf = [0u8; 16];
        loop {
            let len = self.device.read(&mut buf)?;
            if len > 0 {
                self.buffer.extend(&buf[0..len]);
                trace!(
                    "read_into_queue: values {:?}",
                    std::str::from_utf8(&buf[0..len])
                );
            } else {
                trace!("read_into_queue: no values left to read");
                break;
            }
        }
        Ok(())
    }
}
