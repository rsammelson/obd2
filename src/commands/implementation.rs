use crate::{Error, Obd2Device, Result};

use super::{Dtc, DtcsInfo, OxygenSensorData};

pub(super) fn get_vin<T: Obd2Device>(device: &mut T) -> Result<String> {
    let mut result = device.obd_command(0x09, 0x02)?.pop().unwrap();
    result.remove(0); // do not know what this byte is
    Ok(String::from_utf8(result)?)
}

pub(super) trait GetObd2Values<T>
where
    Self: Sized,
{
    fn get_obd2_val(device: &mut T, service: u8, pid: u8) -> Result<Vec<Self>>;
}

impl<T: Obd2Device> GetObd2Values<T> for u8 {
    fn get_obd2_val(device: &mut T, service: u8, pid: u8) -> Result<Vec<Self>> {
        Ok(device
            .obd_command_len::<1>(service, pid)?
            .into_iter()
            .map(|r| r[0])
            .collect())
    }
}

impl<T: Obd2Device, const N: usize> GetObd2Values<T> for [u8; N] {
    fn get_obd2_val(device: &mut T, service: u8, pid: u8) -> Result<Vec<Self>> {
        device.obd_command_len::<N>(service, pid)
    }
}

impl<T: Obd2Device> GetObd2Values<T> for u16 {
    fn get_obd2_val(device: &mut T, service: u8, pid: u8) -> Result<Vec<Self>> {
        Ok(<[u8; 2]>::get_obd2_val(device, service, pid)?
            .into_iter()
            .map(Self::from_be_bytes)
            .collect())
    }
}

impl<T: Obd2Device> GetObd2Values<T> for u32 {
    fn get_obd2_val(device: &mut T, service: u8, pid: u8) -> Result<Vec<Self>> {
        Ok(<[u8; 4]>::get_obd2_val(device, service, pid)?
            .into_iter()
            .map(Self::from_be_bytes)
            .collect())
    }
}

impl<T: Obd2Device> GetObd2Values<T> for DtcsInfo {
    fn get_obd2_val(device: &mut T, service: u8, pid: u8) -> Result<Vec<Self>> {
        let result = device.obd_command(service, pid)?;

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
}

impl<T: Obd2Device> GetObd2Values<T> for Dtc {
    fn get_obd2_val(device: &mut T, service: u8, pid: u8) -> Result<Vec<Self>> {
        let resp = u16::get_obd2_val(device, service, pid)?;
        Ok(resp.into_iter().map(|v| v.into()).collect())
    }
}

impl<T: Obd2Device> GetObd2Values<T> for OxygenSensorData {
    fn get_obd2_val(device: &mut T, service: u8, pid: u8) -> Result<Vec<Self>> {
        let resp = <[u8; 2]>::get_obd2_val(device, service, pid)?;
        Ok(resp
            .into_iter()
            .map(|v| OxygenSensorData {
                voltage: f32::from(v[0]) * 0.005,
                shrft: f32::from(i16::from(v[1]) - 128) / 128.,
            })
            .collect())
    }
}

pub(crate) trait GetObd2ValuesMode<T>
where
    Self: Sized,
{
    fn get_obd2_val_mode(device: &mut T, service: u8) -> Result<Vec<Self>>;
}

impl<T: Obd2Device> GetObd2ValuesMode<T> for Vec<Dtc> {
    fn get_obd2_val_mode(device: &mut T, service: u8) -> Result<Vec<Self>> {
        let result = device.obd_mode_command(service)?;
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
}
