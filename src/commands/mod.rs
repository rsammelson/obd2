//! High level OBD-II interface

mod implementation;

mod types;
pub use types::{Dtc, DtcsInfo};

use crate::Result;

/// Trait for devices that can retrieve data over OBD-II
///
/// Automatically implemented for implementors of [Obd2Device](crate::Obd2Device).
pub trait Obd2DataRetrieval: private::Sealed {
    /// Check which getters are supported by the current vehicle
    // fn get_support() -> Obd2FunctionSupport;

    /// Retreive the VIN (vehicle identification number)
    ///
    /// Service 0x09, PID 0x01. This should match the number printed on the vehicle, and is a good
    /// command for checking that the OBD-II interface is working correctly.
    fn get_vin(&mut self) -> Result<String>;

    /// Get DTC (diagnostic trouble code) metadata for each ECU
    fn get_dtc_info(&mut self) -> Result<Vec<DtcsInfo>>;

    /// Get DTCs for each ECU
    fn get_dtcs(&mut self) -> Result<Vec<Vec<Dtc>>>;

    /// Get the calculated engine load (out of 255)
    fn get_engine_load(&mut self) -> Result<u8>;

    /// Get the temperature of the engine's coolant in ºC
    fn get_engine_coolant_temperature(&mut self) -> Result<i16>;

    /// Get the fuel pressure in kPa
    ///
    /// This measurement is gauge pressure (measured relative to the atmosphere)
    fn get_fuel_pressure(&mut self) -> Result<i16>;

    /// Get the intake manifold pressure in kPa
    ///
    /// This measurement is absolute pressure.
    fn get_engine_manifold_pressure(&mut self) -> Result<f32>;

    /// Get the RPM in increments of 0.25
    fn get_rpm(&mut self) -> Result<f32>;

    /// Get the speed in km/h
    fn get_speed(&mut self) -> Result<u8>;
}

mod private {
    pub trait Sealed {}
    impl<T: crate::Obd2Device> Sealed for T {}
}
