use core::fmt;

pub type Result<T> = std::result::Result<T, Error>;

pub trait Obd2Device {
    fn obd_command(&mut self, mode: u8, pid: u8) -> Result<Vec<Vec<u8>>>;
    fn obd_mode_command(&mut self, mode: u8) -> Result<Vec<Vec<u8>>>;

    fn obd_command_len<const RESPONSE_LENGTH: usize>(
        &mut self,
        mode: u8,
        pid: u8,
    ) -> Result<Vec<[u8; RESPONSE_LENGTH]>> {
        self.obd_command(mode, pid)?
            .into_iter()
            .map(|v| {
                let l = v.len();
                v.try_into()
                    .map_err(|_| Error::IncorrectResponseLength("length", RESPONSE_LENGTH, l))
            })
            .collect()
    }

    fn obd_command_cnt_len<const RESPONSE_COUNT: usize, const RESPONSE_LENGTH: usize>(
        &mut self,
        mode: u8,
        pid: u8,
    ) -> Result<[[u8; RESPONSE_LENGTH]; RESPONSE_COUNT]> {
        let result = self.obd_command_len::<RESPONSE_LENGTH>(mode, pid)?;
        let count = result.len();
        result
            .try_into()
            .map_err(|_| Error::IncorrectResponseLength("count", RESPONSE_COUNT, count))
    }

    fn get_vin(&mut self) -> Result<String> {
        let mut result = self.obd_command(0x09, 0x02)?.pop().unwrap();
        result.remove(0); // do not know what this byte is
        Ok(String::from_utf8(result)?)
    }

    fn get_dtc_info(&mut self) -> Result<Vec<DtcsInfo>> {
        let result = self.obd_command(0x01, 0x01)?;

        result
            .iter()
            .map(|response| {
                if response.len() == 4 {
                    Ok(DtcsInfo {
                        malfunction_indicator_light: (response[0] & 0x80) == 0x80,
                        dtc_count: response[0] & 0x7f,
                        common_test_availability: ((response[1] & 0xf0) >> 1)
                            | (response[1] & 0x07),
                        is_compression_engine: (response[1] & 0x08) == 0x08,
                        specific_test_availability: ((response[3] as u16) << 8)
                            | (response[2] as u16),
                    })
                } else {
                    Err(Error::Other(format!(
                        "get_dtc_info: expected length 4, got {}",
                        response.len()
                    )))
                }
            })
            .collect()
    }

    fn get_dtcs(&mut self) -> Result<Vec<Vec<Dtc>>> {
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
                        Err(Error::Other(format!(
                            "invalid response when getting DTCs {:?}",
                            response
                        )))
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

    fn get_rpm(&mut self) -> Result<f32> {
        let result = self.obd_command_cnt_len::<1, 2>(0x01, 0x0C)?[0];
        Ok(f32::from(u16::from_be_bytes(result)) / 4.0)
    }

    fn get_speed(&mut self) -> Result<u8> {
        Ok(self.obd_command_cnt_len::<1, 1>(0x01, 0x0C)?[0][0])
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct DtcsInfo {
    malfunction_indicator_light: bool,
    dtc_count: u8,
    common_test_availability: u8,
    is_compression_engine: bool,
    specific_test_availability: u16,
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
    #[error("Device error: `{0:?}`")]
    Device(DeviceError),
    #[error("Other OBD2 error: `{0}`")]
    Other(String),
    #[error("Incorrect length (`{0}`): expected `{1}`, got `{2}`")]
    IncorrectResponseLength(&'static str, usize, usize),
}

#[derive(Debug)]
pub struct DeviceError(super::device::Error);

impl From<super::device::Error> for Error {
    fn from(e: super::device::Error) -> Self {
        Error::Device(DeviceError(e))
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
