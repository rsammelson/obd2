//! High level OBD-II interface

use std::fmt;

use crate::{Error, Obd2Device, Result};

/// Trait for devices that can retrieve data over OBD-II
///
/// Automatically impelemted for implementors of [Obd2Device].
pub trait Obd2DataRetrieval: private::Sealed {
    /// Retreive the VIN (vehicle identification number)
    ///
    /// This should match the number printed on the vehicle, and is a good command for checking
    /// that the OBD-II interface is working correctly.
    fn get_vin(&mut self) -> Result<String>;

    /// Get DTC (diagnostic trouble code) metadata for each ECU
    fn get_dtc_info(&mut self) -> Result<Vec<DtcsInfo>>;

    /// Get DTCs for each ECU
    fn get_dtcs(&mut self) -> Result<Vec<Vec<Dtc>>>;

    /// Get the RPM in increments of 0.25
    fn get_rpm(&mut self) -> Result<f32>;

    /// Get the speed in km/h
    fn get_speed(&mut self) -> Result<u8>;
}

impl<T: Obd2Device> Obd2DataRetrieval for T {
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

/// DTC (diagnostic trouble code) metadata
#[derive(Debug)]
#[non_exhaustive]
pub struct DtcsInfo {
    /// Whether the "check engine" light is illuminated
    pub malfunction_indicator_light: bool,

    /// Number of DTCs for this ECU
    pub dtc_count: u8,

    /// Bit field showing availability of seven common tests; the upper bit is currently unused.
    pub common_test_availability: u8,

    /// Whether the engine is Diesel
    pub is_compression_engine: bool,

    /// Bit field showing availability of sixteen engine-specific tests. What the tests are is
    /// based on the value of `is_compression_engine`.
    pub specific_test_availability: u16,
}

/// An individual trouble code from an ECU
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

mod private {
    pub trait Sealed {}
    impl<T: super::Obd2Device> Sealed for T {}
}
