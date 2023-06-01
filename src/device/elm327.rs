use log::{debug, info, trace};
use std::{
    collections::VecDeque,
    io::{Read, Write},
    thread, time,
};

use super::{Error, Obd2BaseDevice, Obd2Reader, Result};

/// An ELM327 OBD-II adapter
///
/// It communicates with the computer over UART using an FTDI FT232R USB-to-UART converter.
/// Commands to the device itself are indicated by sending "AT" followed by the command, while
/// plain strings of hex data indicate OBD-II requests to be sent to the vehicle. The responses of
/// the vehicle are echoed back as hex characters. Capitalization and spaces are always ignored.
///
/// [Datasheet for v1.4b](https://github.com/rsammelson/obd2/blob/master/docs/ELM327DSH.pdf), and
/// the [source](https://www.elmelectronics.com/products/dsheets/).
pub struct Elm327 {
    device: ftdi::Device,
    buffer: VecDeque<u8>,
    baud_rate: u32,
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

    fn send_cmd(&mut self, data: &[u8]) -> Result<()> {
        trace!("send_cmd: sending {:?}", std::str::from_utf8(data));
        self.send_serial_str(
            data.into_iter()
                .flat_map(|v| format!("{:02X}", v).chars().collect::<Vec<char>>())
                .collect::<String>()
                .as_str(),
        )
    }
}

impl Obd2Reader for Elm327 {
    fn get_line(&mut self) -> Result<Option<Vec<u8>>> {
        self.get_until(b'\n', false)
    }

    /// Read data until the ELM327's prompt character is printed
    ///
    /// This will receive the entire OBD-II response. The prompt signifies that the ELM327 is ready
    /// for another command. If this is not called after each OBD-II command is sent, the prompt
    /// character will come out of the receive queue later and because it is not valid hex this
    /// could cause problems. If a timeout occurs, `Ok(None)` will be returned.
    fn get_response(&mut self) -> Result<Option<Vec<u8>>> {
        self.get_until(b'>', true)
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
            baud_rate: 38400,
        };

        device.connect(false)?;
        device.flush()?;

        Ok(device)
    }

    /// Flush the device's buffer
    pub fn flush(&mut self) -> Result<()> {
        thread::sleep(time::Duration::from_millis(500));
        self.read_into_queue()?;
        self.buffer.clear();
        thread::sleep(time::Duration::from_millis(500));
        Ok(())
    }

    fn flush_buffers(&mut self) -> Result<()> {
        self.device.usb_purge_buffers()?;
        Ok(())
    }

    fn connect(&mut self, check_baud_rate: bool) -> Result<()> {
        self.flush_buffers()?;
        thread::sleep(time::Duration::from_millis(500));
        self.serial_cmd(" ")?;
        thread::sleep(time::Duration::from_millis(500));

        self.reset()?;

        if check_baud_rate {
            match self.find_baud_rate_divisor()? {
                Some((rate, div)) => info!("Found baud rate {} (divisor {})", rate, div),
                None => info!("Could not find better baud rate"),
            }
        }

        Ok(())
    }

    fn reset_ic(&mut self) -> Result<()> {
        info!("Performing IC reset");
        self.send_serial_str("ATZ")?;
        debug!(
            "reset_ic: got response {:?}",
            self.get_response()?
                .as_ref()
                .map(|l| std::str::from_utf8(l.as_slice()))
        );
        Ok(())
    }

    fn reset_protocol(&mut self) -> Result<()> {
        info!("Performing protocol reset");
        debug!(
            "reset_protocol: got response {:?}",
            self.serial_cmd("ATSP0")?
        );
        debug!(
            "reset_protocol: got OBD response {:?}",
            self.cmd(&[0x01, 0x00])?
        );
        self.flush_buffers()?;
        Ok(())
    }

    fn find_baud_rate_divisor(&mut self) -> Result<Option<(u8, u32)>> {
        for div in 90..104u8 {
            let new_baud = 4000000 / u32::from(div);

            debug!("Trying baud rate {} (divisor {})", new_baud, div);
            self.send_serial_str(&format!("ATBRD{:02X}", div))?;

            if self.get_line()? == Some(b"OK".to_vec()) {
                self.device.set_baud_rate(new_baud)?;

                // validate new baud rate
                let validation_response = self.get_line()?;
                if validation_response == Some(b"ELM327 v1.5".to_vec()) {
                    // reply that it is okay
                    self.send_serial_str("\r")
                        .expect("Device left in unknown state");
                    if self.get_line().expect("Device left in unknown state")
                        == Some(b"OK".to_vec())
                    {
                        self.baud_rate = new_baud;
                        return Ok(Some((div, new_baud)));
                    } else {
                        // our TX is bad
                        self.device.set_baud_rate(self.baud_rate)?;
                        debug!("Baud rate bad - device did not receive response");
                        self.get_response()?;
                    }
                } else {
                    // reset baud rate and keep looking
                    self.device.set_baud_rate(self.baud_rate)?;
                    debug!(
                        "Baud rate bad - did get correct string (got {:?} - {:?})",
                        validation_response,
                        validation_response
                            .as_ref()
                            .map(|r| String::from_utf8_lossy(r))
                    );
                    self.get_response()?;
                }
            } else {
                debug!("Baud rate bad - did not ok initially");
                self.get_response()?;
            }

            thread::sleep(time::Duration::from_millis(200));
        }
        Ok(None)
    }

    fn get_until(&mut self, end_byte: u8, allow_empty: bool) -> Result<Option<Vec<u8>>> {
        const TIMEOUT: time::Duration = time::Duration::from_secs(5);

        trace!("get_until: getting until {}", end_byte);

        let mut buf = Vec::new();
        let start = time::Instant::now();
        while start.elapsed() < TIMEOUT {
            let Some(b) = self.get_byte()? else { continue };
            let b = match b {
                b'\r' => Some(b'\n'),
                b'\n' => None, // no push here
                _ => Some(b),
            };
            if let Some(b) = b {
                buf.push(b);
                if b == end_byte {
                    break;
                }
            }
        }

        trace!(
            "get_until: got {:?} ({:?})",
            buf,
            std::str::from_utf8(buf.as_slice())
        );

        match buf.pop() {
            Some(b) if b == end_byte => {
                if allow_empty || !buf.is_empty() {
                    Ok(Some(buf))
                } else {
                    // empty line, try again
                    self.get_until(end_byte, allow_empty)
                }
            } // we got it
            Some(f) => {
                // incomplete line read
                for b in buf.iter().rev() {
                    self.buffer.push_front(*b);
                }
                self.buffer.push_front(f);
                Ok(None)
            }
            None => Ok(None),
        }
    }

    fn get_byte(&mut self) -> Result<Option<u8>> {
        match self.buffer.pop_front() {
            Some(b'\0') => Ok(None),
            Some(b) => Ok(Some(b)),
            None => {
                self.read_into_queue()?;
                Ok(None)
            }
        }
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

    fn serial_cmd(&mut self, cmd: &str) -> Result<Option<String>> {
        self.send_serial_str(cmd)?;
        self.get_response()
            .map(|o| o.and_then(|resp| String::from_utf8(resp).ok()))
    }

    /// Function for sending a raw string, without encoding into ASCII hex
    fn send_serial_str(&mut self, data: &str) -> Result<()> {
        trace!("send_serial_str: sending {:?}", data);

        let data = data.as_bytes();

        self.device.write_all(data)?;
        self.device.write_all(b"\r\n")?;
        let line = self.get_line()?;
        if line.as_ref().is_some_and(|v| v == data) {
            Ok(())
        } else {
            Err(Error::Communication(format!(
                "send_serial_str: got {:?} instead of echoed command ({:?})",
                line, data
            )))
        }
    }
}
