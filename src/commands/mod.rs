//! High level OBD-II interface

mod implementation;
use implementation::GetObd2Values;

#[macro_use]
mod macros;

mod types;
use types::private;
pub use types::{Dtc, DtcsInfo};

use crate::{Obd2Device, Result};

func! {
    /// Trait for devices that can retrieve data over OBD-II
    ///
    /// Automatically implemented for implementors of [Obd2Device](crate::Obd2Device).
    trait Obd2DataRetrieval;

    {
        /// Retreive the VIN (vehicle identification number)
        ///
        /// This should match the number printed on the vehicle, and is a good
        /// command for checking that the OBD-II interface is working correctly.
        fn get_vin(self, 0x09, 0x02) -> Result<String> {
            implementation::get_vin(self)
        }

        /// Get DTCs for each ECU
        fn get_dtcs(self, 0x03) -> Result<Vec<Vec<Dtc>>> {
            implementation::get_dtcs(self)
        }

        /// Get DTC (diagnostic trouble code) metadata for each ECU
        fn get_dtc_info(self, 0x01, 0x01) -> Result<Vec<DtcsInfo>> {
            implementation::get_dtc_info(self)
        }

    }

    /// Get the calculated engine load (out of 255)
    fn get_engine_load(0x01, 0x04) -> u8;

    /// Get the temperature of the engine's coolant in ºC
    fn get_engine_coolant_temperature<u8>(0x01, 0x05, |v: i16| v - 40) -> i16;

    /// Get the fuel pressure in kPa
    ///
    /// This measurement is gauge pressure (measured relative to the atmosphere)
    fn get_fuel_pressure<u8>(0x01, 0x0A, |v: i16| v * 3) -> i16;

    /// Get the intake manifold pressure in kPa
    ///
    /// This measurement is absolute pressure.
    fn get_engine_manifold_pressure<u16>(0x01, 0x0B, |v: f32| v) -> f32;

    /// Get the RPM in increments of 0.25
    fn get_rpm<u16>(0x01, 0x0C, |v: f32| v / 4.0) -> f32;

    /// Get the speed in km/h
    fn get_speed(0x01, 0x0D) -> u8;
}
