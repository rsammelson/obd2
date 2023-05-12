mod device;

type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct Obd2 {
    device: device::Obd2Basic,
}

impl Obd2 {
    pub fn get_vin(&mut self) -> Result<String> {
        let response = self
            .device
            .cmd("0902")?
            .ok_or(Error::Other("no response to get vin command".to_owned()))?;
        let mut n_idx = 0;
        let data: Vec<String> = response
            .split('\n')
            .filter_map(|l| l.split_once(':'))
            .flat_map(|(idx, data)| {
                if u8::from_str_radix(idx, 16) != Ok(n_idx) {
                    todo!()
                }
                n_idx = (n_idx + 1) % 0x10;
                data.split_whitespace().map(|s| s.to_owned())
            })
            .collect();

        if data.get(0).map(|s| s.as_str()) != Some("49") {
            todo!()
        }
        if data.get(1).map(|s| s.as_str()) != Some("02") {
            todo!()
        }

        let vin = String::from_utf8(
            data.split_at(3)
                .1
                .iter()
                .map(|s| u8::from_str_radix(s, 16).map_err(|e| e.into()))
                .collect::<Result<_>>()?,
        )?;

        Ok(vin)
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
        Error::Other(format!("Invalid data recieved: {:?}", e))
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::Other(format!("Invalid string recieved: {:?}", e))
    }
}
