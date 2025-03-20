use log::{debug, trace};

use super::{device::Obd2BaseDevice, Error, Obd2Device, Result};

/// An OBD-II interface
///
/// Wraps an implementer of [Obd2BaseDevice] to allow for higher-level usage of the OBD-II
/// interface.
#[derive(Default)]
pub struct Obd2<T: Obd2BaseDevice> {
    device: T,
}

impl<T: Obd2BaseDevice> Obd2Device for Obd2<T> {
    fn obd_command(&mut self, mode: u8, pid: u8) -> Result<Vec<Vec<u8>>> {
        let result = self.command(&[mode, pid])?;

        for response in result.iter() {
            if response.first() != Some(&(0x40 | mode)) {
                // mismatch of mode in response
                todo!()
            }
            if response.get(1) != Some(&pid) {
                // mismatch of PID in response
                todo!()
            }
        }

        Ok(result.iter().map(|l| l.split_at(2).1.to_vec()).collect())
    }

    fn obd_mode_command(&mut self, mode: u8) -> Result<Vec<Vec<u8>>> {
        let result = self.command(std::slice::from_ref(&mode))?;

        for response in result.iter() {
            if response.first() != Some(&(0x40 | mode)) {
                todo!()
            }
        }

        Ok(result.iter().map(|l| l.split_at(1).1.to_vec()).collect())
    }
}

impl<T: Obd2BaseDevice> Obd2<T> {
    fn command(&mut self, command: &[u8]) -> Result<Vec<Vec<u8>>> {
        let response = self
            .device
            .cmd(command)?
            .ok_or(Error::Other("no response to command".to_owned()))?;

        trace!(
            "Sent OBD command {:?} and got response {:?}",
            command,
            response
        );

        let data = if response.contains("0:") {
            vec![self.parse_command_multiline(response)?]
        } else {
            self.parse_command(response)?
        };

        debug!("Sent OBD command {:?} and got data {:?}", command, data);

        let result = data
            .iter()
            .map(|l| {
                l.iter()
                    .map(|s| u8::from_str_radix(s, 16).map_err(|e| e.into()))
                    .collect()
            })
            .collect();

        result
    }

    fn parse_command(&mut self, response: String) -> Result<Vec<Vec<String>>> {
        let result: Vec<_> = response
            .split('\n')
            .filter_map(|l| {
                let res: Vec<_> = l
                    .split(' ')
                    .filter_map(|s| {
                        if !s.is_empty() {
                            Some(s.to_owned())
                        } else {
                            None
                        }
                    })
                    .collect();
                if !res.is_empty() {
                    Some(res)
                } else {
                    None
                }
            })
            .collect();

        if !result.is_empty() {
            Ok(result)
        } else {
            Err(Error::Other("parse_command: found no responses".to_owned()))
        }
    }

    fn parse_command_multiline(&mut self, response: String) -> Result<Vec<String>> {
        let mut n_idx = 0;
        Ok(response
            .split('\n')
            .filter_map(|l| l.split_once(':'))
            .flat_map(|(idx, data)| {
                if u8::from_str_radix(idx, 16) != Ok(n_idx) {
                    // got an invalid hex code or values were not already in the correct order
                    todo!("Line index: {}, should be {:X}", idx, n_idx)
                }
                n_idx = (n_idx + 1) % 0x10;
                data.split_whitespace().map(|s| s.to_owned())
            })
            .collect())
    }
}
