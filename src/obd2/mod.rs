use log::{debug, trace};

mod device;

type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct Obd2 {
    device: device::Obd2Basic,
}

impl Obd2 {
    pub fn obd_command(&mut self, mode: u8, pid: u8) -> Result<Vec<u8>> {
        let result = self.command(&format!("{:02x}{:02x}", mode, pid))?;

        if result.first() != Some(&(0x40 | mode)) {
            todo!()
        }
        if result.get(1) != Some(&pid) {
            todo!()
        }

        Ok(result.split_at(2).1.to_vec())
    }

    fn command(&mut self, command: &str) -> Result<Vec<u8>> {
        let response = self
            .device
            .cmd(command)?
            .ok_or(Error::Other("no response to command".to_owned()))?;

        trace!(
            "Sent OBD command {:?} and got response {:?}",
            command,
            response
        );

        let data = self.parse_command_multiline(response)?;

        debug!("Sent OBD command {:?} and got data {:?}", command, data);

        let result = data
            .iter()
            .map(|s| u8::from_str_radix(s, 16).map_err(|e| e.into()))
            .collect();

        result
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
        let mut result = self.obd_command(0x09, 0x02)?;
        result.remove(0); // do not know what this byte is
        Ok(String::from_utf8(result)?)
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
