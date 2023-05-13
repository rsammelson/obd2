use core::fmt;

use log::{debug, trace};

mod device;

type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct Obd2 {
    device: device::Obd2Basic,
}

impl Obd2 {
    pub fn obd_command(&mut self, mode: u8, pid: u8) -> Result<Vec<Vec<u8>>> {
        let result = self.command(&format!("{:02x}{:02x}", mode, pid))?;

        for response in result.iter() {
            if response.first() != Some(&(0x40 | mode)) {
                todo!()
            }
            if response.get(1) != Some(&pid) {
                todo!()
            }
        }

        Ok(result.iter().map(|l| l.split_at(2).1.to_vec()).collect())
    }

    pub fn obd_mode_command(&mut self, mode: u8) -> Result<Vec<Vec<u8>>> {
        let result = self.command(&format!("{:02x}", mode))?;

        for response in result.iter() {
            if response.first() != Some(&(0x40 | mode)) {
                todo!()
            }
        }

        Ok(result.iter().map(|l| l.split_at(1).1.to_vec()).collect())
    }

    fn command(&mut self, command: &str) -> Result<Vec<Vec<u8>>> {
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
                    todo!()
                }
                n_idx = (n_idx + 1) % 0x10;
                data.split_whitespace().map(|s| s.to_owned())
            })
            .collect())
    }

    pub fn get_vin(&mut self) -> Result<String> {
        let mut result = self.obd_command(0x09, 0x02)?.pop().unwrap();
        result.remove(0); // do not know what this byte is
        Ok(String::from_utf8(result)?)
    }

    pub fn get_dtcs(&mut self) -> Result<Vec<Vec<Dtc>>> {
        let result = self.obd_mode_command(0x03)?;
        result
            .iter()
            .map(|response| match response.first() {
                Some(0) => {
                    if response.len() % 2 == 1 {
                        let mut ret = Vec::new();
                        for i in (1..response.len()).step_by(2) {
                            ret.push(match response[i] >> 6 {
                                0 => Dtc::Powertrain(0),
                                1 => Dtc::Chassis(0),
                                2 => Dtc::Body(0),
                                3 => Dtc::Network(0),
                                _ => unreachable!(),
                            });
                        }
                        Ok(ret)
                    } else {
                        Err(Error::Other(format!("invalid response {:?}", response)))
                    }
                }
                Some(n) if *n <= 3 => todo!(),
                Some(_) => Err(Error::Other(format!(
                    "invalid response {:?} when getting DTCs",
                    response
                ))),
                None => Err(Error::Other(
                    "no response bytes when getting DTCs".to_owned(),
                )),
            })
            .collect::<Result<Vec<Vec<Dtc>>>>()
    }
}

#[derive(Debug)]
pub enum Dtc {
    Powertrain(u16),
    Chassis(u16),
    Body(u16),
    Network(u16),
}

impl fmt::Display for Dtc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (c, n) = match self {
            Self::Powertrain(n) => ('P', n),
            Self::Chassis(n) => ('C', n),
            Self::Body(n) => ('B', n),
            Self::Network(n) => ('U', n),
        };
        f.write_fmt(format_args!("{}{:03X}", c, n))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Communication error: `{0:?}`")]
    Communication(device::Error),
    #[error("Other OBD2 error: `{0}`")]
    Other(String),
}

impl From<device::Error> for Error {
    fn from(e: device::Error) -> Self {
        Error::Communication(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::Other(format!("invalid data recieved: {:?}", e))
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::Other(format!("invalid string recieved: {:?}", e))
    }
}
