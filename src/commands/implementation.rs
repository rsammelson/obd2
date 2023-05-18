use crate::{Error, Obd2Device, Result};

use super::{Dtc, DtcsInfo, Obd2DataRetrieval};

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

    fn get_engine_load(&mut self) -> Result<u8> {
        Ok(self.obd_command_cnt_len::<1, 1>(0x01, 0x0C)?[0][0])
    }

    fn get_engine_coolant_temperature(&mut self) -> Result<i16> {
        Ok((self.obd_command_cnt_len::<1, 1>(0x01, 0x0C)?[0][0] as i16) - 40)
    }

    fn get_fuel_pressure(&mut self) -> Result<i16> {
        todo!()
    }

    fn get_engine_manifold_pressure(&mut self) -> Result<f32> {
        todo!()
    }

    fn get_rpm(&mut self) -> Result<f32> {
        let result = self.obd_command_cnt_len::<1, 2>(0x01, 0x0C)?[0];
        Ok(f32::from(u16::from_be_bytes(result)) / 4.0)
    }

    fn get_speed(&mut self) -> Result<u8> {
        Ok(self.obd_command_cnt_len::<1, 1>(0x01, 0x0C)?[0][0])
    }
}
