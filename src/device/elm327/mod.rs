mod low_level;

use log::{debug, info, trace};
use std::{io::BufRead as _, thread, time};

use super::{Error, Obd2BaseDevice, Obd2Reader, Result, SerialCommunication};

/// An ELM327 OBD-II adapter
///
/// It communicates with the computer over UART using an FTDI FT232R USB-to-UART converter.
/// Commands to the device itself are indicated by sending "AT" followed by the command, while
/// plain strings of hex data indicate OBD-II requests to be sent to the vehicle. The responses of
/// the vehicle are echoed back as hex characters. Capitalization and spaces are always ignored.
///
/// [Datasheet for v1.4b](https://github.com/rsammelson/obd2/blob/master/docs/ELM327DSH.pdf), and
/// the [source](https://www.elmelectronics.com/products/dsheets/).
pub struct Elm327<T: SerialCommunication> {
    device: std::io::BufReader<low_level::Elm327Reader<T>>,
}

impl<T: SerialCommunication> Obd2BaseDevice for Elm327<T> {
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
            data.iter()
                .flat_map(|v| format!("{:02X}", v).chars().collect::<Vec<char>>())
                .collect::<String>()
                .as_str(),
        )
    }
}

impl<T: SerialCommunication> Obd2Reader for Elm327<T> {
    fn get_line(&mut self) -> Result<Option<Vec<u8>>> {
        let mut buffer = Vec::new();
        Ok(self
            .device
            .read_until(b'\r', &mut buffer)
            .map(|_| Some(buffer))
            .or_else(|e| {
                if matches!(e.kind(), std::io::ErrorKind::TimedOut) {
                    Ok(None)
                } else {
                    Err(e)
                }
            })?)
    }

    /// Read data until the ELM327's prompt character is printed
    ///
    /// This will receive the entire OBD-II response. The prompt signifies that the ELM327 is ready
    /// for another command. If this is not called after each OBD-II command is sent, the prompt
    /// character will come out of the receive queue later and because it is not valid hex this
    /// could cause problems. If a timeout occurs, `Ok(None)` will be returned.
    fn get_response(&mut self) -> Result<Option<Vec<u8>>> {
        let mut buffer = Vec::new();
        Ok(self
            .device
            .read_until(b'>', &mut buffer)
            .map(|_| Some(buffer))
            .or_else(|e| {
                if matches!(e.kind(), std::io::ErrorKind::TimedOut) {
                    Ok(None)
                } else {
                    Err(e)
                }
            })?)
    }
}

impl<T: SerialCommunication> Elm327<T> {
    /// Creates a new Elm327 adapter with the given underlying serial device
    pub fn new(device: T) -> Result<Self> {
        let mut device = Elm327 {
            device: std::io::BufReader::new(low_level::Elm327Reader::new(device)),
        };

        device.connect(false)?;
        device.flush()?;

        Ok(device)
    }

    /// Flush the device's buffer
    pub fn flush(&mut self) -> Result<()> {
        thread::sleep(time::Duration::from_millis(500));
        loop {
            match self.device.fill_buf()?.len() {
                0 => break,
                n if n == self.device.capacity() => self.device.consume(n),
                n => {
                    self.device.consume(n);
                    break;
                }
            }
        }
        thread::sleep(time::Duration::from_millis(100));
        Ok(())
    }

    fn flush_buffers(&mut self) -> Result<()> {
        match self.device.buffer().len() {
            0 => {}
            n => self.device.consume(n),
        }
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
        let response = self.get_response()?;
        debug!(
            "reset_ic: got response {:?}",
            response.as_ref().map(|l| std::str::from_utf8(l.as_slice()))
        );
        Ok(())
    }

    fn reset_protocol(&mut self) -> Result<()> {
        info!("Performing protocol reset");

        // set to use automatic protocol selection
        let elm_response = self.serial_cmd("ATSP0")?;
        debug!("reset_protocol: got response {:?}", elm_response);

        // perform the search for ECUs
        let obd_response = self.cmd(&[0x01, 0x00])?;
        debug!("reset_protocol: got OBD response {:?}", obd_response);

        // get rid of extra data hanging around in the buffer
        self.flush_buffers()?;

        Ok(())
    }

    fn find_baud_rate_divisor(&mut self) -> Result<Option<(u8, u32)>> {
        for div in 90..104u8 {
            let new_baud = 4000000 / u32::from(div);

            debug!("Trying baud rate {} (divisor {})", new_baud, div);
            self.send_serial_str(&format!("ATBRD{:02X}", div))?;

            if self.get_line()? == Some(b"OK".to_vec()) {
                let old_baud_rate = self.device.get_mut().get_mut().get_baud_rate()?;
                self.device.get_mut().get_mut().set_baud_rate(new_baud)?;

                // validate new baud rate
                let validation_response = self.get_line()?;
                if validation_response == Some(b"ELM327 v1.5".to_vec()) {
                    // reply that it is okay
                    if self
                        .send_serial_str("\r")
                        .and_then(|()| self.get_line())
                        .inspect_err(|err| {
                            log::warn!("Device left in unknown state (error: {:?})", err)
                        })?
                        == Some(b"OK".to_vec())
                    {
                        return Ok(Some((div, new_baud)));
                    } else {
                        // our TX is bad
                        self.device
                            .get_mut()
                            .get_mut()
                            .set_baud_rate(old_baud_rate)?;
                        debug!("Baud rate bad - device did not receive response");
                        self.get_response()?;
                    }
                } else {
                    // reset baud rate and keep looking
                    self.device
                        .get_mut()
                        .get_mut()
                        .set_baud_rate(old_baud_rate)?;
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

    fn serial_cmd(&mut self, cmd: &str) -> Result<Option<String>> {
        self.send_serial_str(cmd)?;
        self.get_response()
            .map(|o| o.and_then(|resp| String::from_utf8(resp).ok()))
    }

    /// Function for sending a raw string, without encoding into ASCII hex
    fn send_serial_str(&mut self, data: &str) -> Result<()> {
        trace!("send_serial_str: sending {:?}", data);

        let data = data.as_bytes();

        self.device.get_mut().get_mut().write_all(data)?;
        self.device.get_mut().get_mut().write_all(b"\r\n")?;
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
