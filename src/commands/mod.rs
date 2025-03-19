//! High level OBD-II interface
//!
//! Retrieves data from the vehicle, over the OBD-II link. The interface is defined by SAE J1979,
//! and a list of services and PIDs is available [on
//! Wikipedia](https://en.wikipedia.org/wiki/OBD-II_PIDs). This module mostly uses service 1.

mod implementation;
use implementation::{GetObd2Values, GetObd2ValuesMode};

#[macro_use]
mod macros;

mod types;
use types::private;
pub use types::{Dtc, DtcsInfo, OxygenSensorData};

use crate::{Obd2Device, Result};

func! {
    /// Trait for devices that can retrieve data over OBD-II
    ///
    /// Automatically implemented for implementers of [odb2::Obd2Device](crate::Obd2Device), and
    /// currently cannot be otherwise implemented.
    trait Obd2DataRetrieval;

    {
        /// Retrieve the VIN (vehicle identification number)
        ///
        /// This should match the number printed on the vehicle, and is a good command for checking
        /// that the OBD-II interface is working correctly.
        fn get_vin(self, 0x09, 0x02) -> Result<String> {
            implementation::get_vin(self)
        }
    }

    /// Get list of DTCs for each ECU
    fn get_dtcs(0x03) -> Vec<Dtc>;

    /// Get service 1 PID support for $01 to $20
    fn get_service_1_pid_support_1(0x01, 0x00) -> u32;

    /// Get DTC (diagnostic trouble code) metadata for each ECU
    fn get_dtc_info(0x01, 0x01) -> DtcsInfo;

    /// Get DTC that caused the current freeze frame
    fn get_freeze_frame_dtc(0x01, 0x02) -> Dtc;

    /// Get fuel system status (system A and B)
    ///
    /// The first value describes the first fuel system. If there is a secondary fuel system, the
    /// second value can be nonzero. Each value can represent one bank using the bottom four bits,
    /// or two banks using either the bottom five bits or the whole byte, depending on the
    /// configuration of the banks.
    ///
    /// Valid values for single bank systems:
    /// - `0`: engine off (including temporarily for vehicles that turn of the engine at idle)
    /// - `1`: open loop — conditions to go closed loop not yet met
    /// - `2`: closed loop
    /// - `4`: open loop — due to current conditions
    /// - `8`: open loop — due to fault
    fn get_fuel_system_status(0x01, 0x03) -> [u8; 2];

    /// Get the calculated engine load (out of 255)
    fn get_engine_load(0x01, 0x04) -> u8;

    /// Get the temperature of the engine's coolant in ºC
    fn get_engine_coolant_temperature<u8>(0x01, 0x05, |v: i16| v - 40) -> i16;

    /// Get the short term fuel trim for bank 1
    ///
    /// This is for vehicles with closed loop air/fuel ratio control. It ranges from about -1 to 1,
    /// where negative percentages mean the mix is being made more lean. If the fuel system is in
    /// open-loop control, this will read 0.
    fn get_short_term_fuel_trim_1<u8>(0x01, 0x06, |v: f32| (v / 128.) - 1.) -> f32;

    /// Get the long term fuel trim for bank 1
    ///
    /// This is for vehicles with closed loop air/fuel ratio control. It ranges from about -1 to 1,
    /// where negative percentages mean the mix is being made more lean. This long term trim value
    /// represents a value saved between shutdowns of the engine. In open-loop control, if this
    /// value is not used it will read 0.
    fn get_long_term_fuel_trim_1<u8>(0x01, 0x07, |v: f32| (v / 128.) - 1.) -> f32;

    /// Like [get_short_term_fuel_trim_1](Self::get_short_term_fuel_trim_1) but for bank 2
    fn get_short_term_fuel_trim_2<u8>(0x01, 0x08, |v: f32| (v / 128.) - 1.) -> f32;
    /// Like [get_long_term_fuel_trim_1](Self::get_long_term_fuel_trim_1) but for bank 2
    fn get_long_term_fuel_trim_2<u8>(0x01, 0x09, |v: f32| (v / 128.) - 1.) -> f32;

    /// Get the fuel pressure in kPa
    ///
    /// This measurement is gauge pressure (measured relative to the atmosphere).
    fn get_fuel_pressure<u8>(0x01, 0x0A, |v: i16| v * 3) -> i16;

    /// Get the intake manifold pressure in kPa
    ///
    /// This measurement is absolute pressure.
    fn get_engine_manifold_pressure<u16>(0x01, 0x0B, |v: f32| v) -> f32;

    /// Get the RPM of the engine in increments of 0.25
    fn get_rpm<u16>(0x01, 0x0C, |v: f32| v / 4.0) -> f32;

    /// Get the speed of the vehicle in km/h
    fn get_speed(0x01, 0x0D) -> u8;

    /// Get the timing advance in degrees BTDC
    ///
    /// Higher numbers mean the ignition happens earlier; that is, longer before the piston reaches
    /// the top of the cylinder.
    fn get_timing_advance<u8>(0x01, 0x0E, |v: f32| (v - 128.0) / 2.0) -> f32;

    /// Get intake manifold air temperature in ºC
    fn get_intake_air_temperature<u8>(0x01, 0x0F, |v: i16| v - 40) -> i16;

    /// Get air flow rate in g/s
    fn get_air_flow_rate<u16>(0x01, 0x10, |v: f32| v * 0.01) -> f32; // TODO: scaling

    /// Get absolute throttle position (out of 255)
    ///
    /// This is the raw sensor value, so idle throttle will probably be more than 0 and open
    /// throttle will probably be less than 255.
    fn get_throttle_position(0x01, 0x11) -> u8;

    /// Get commanded secondary air status (bitfield)
    ///
    /// This describes where the secondary air system has been commanded to inject air. The valid
    /// values are:
    /// - `1`: Upstream of the first catalytic converter inlet
    /// - `2`: Downstream of the first catalytic converter inlet
    /// - `4`: Off (or atmosphere)
    /// - `8`: On for diagnostics
    ///
    /// This system exists to reduce emissions. By injecting air in front of the catalytic
    /// converter, extra fuel in the exhaust combusts, heating the catalytic converter. Once the
    /// catalytic converter is up to temperature, air is injected into the catalytic converter to
    /// help it catalyze unburned fuel.
    ///
    /// See: <https://en.wikipedia.org/wiki/Secondary_air_injection>
    fn get_commanded_secondary_air_status(0x01, 0x12) -> u8;

    /// Get location of oxygen sensors
    ///
    /// This version (cf. [get_oxygen_sensors_4_bank](Self::get_oxygen_sensors_4_bank)) is
    /// recommended for two bank systems. A vehicle must not support both variants.
    ///
    /// The each nibble represents the sensors of one bank, the less significant nibble is bank 1.
    /// The bits of the nibble represent each of the four possible sensors, with sensor 1 in the
    /// least significant bit.
    fn get_oxygen_sensors_2_bank(0x01, 0x13) -> u8;

    /// Get oxygen sensor 1 voltage and associated air/fuel short term trim
    ///
    /// This is bank 1, sensor 1.
    fn get_oxygen_sensor_1(0x01, 0x14) -> OxygenSensorData;

    /// Get oxygen sensor 2 voltage and associated air/fuel short term trim
    ///
    /// This is for bank 1, sensor 2.
    fn get_oxygen_sensor_2(0x01, 0x15) -> OxygenSensorData;

    /// Get oxygen sensor 3 voltage and associated air/fuel short term trim
    ///
    /// If using two banks, this is for bank 1, sensor 3. If using four banks, this is for bank 2
    /// sensor 1.
    fn get_oxygen_sensor_3(0x01, 0x16) -> OxygenSensorData;

    /// Get oxygen sensor 4 voltage and associated air/fuel short term trim
    ///
    /// If using two banks, this is for bank 1, sensor 4. If using four banks, this is for bank 2
    /// sensor 2.
    fn get_oxygen_sensor_4(0x01, 0x17) -> OxygenSensorData;

    /// Get oxygen sensor 5 voltage and associated air/fuel short term trim
    ///
    /// If using two banks, this is for bank 2, sensor 1. If using four banks, this is for bank 3
    /// sensor 1.
    fn get_oxygen_sensor_5(0x01, 0x18) -> OxygenSensorData;

    /// Get oxygen sensor 6 voltage and associated air/fuel short term trim
    ///
    /// If using two banks, this is for bank 2, sensor 2. If using four banks, this is for bank 3
    /// sensor 2.
    fn get_oxygen_sensor_6(0x01, 0x19) -> OxygenSensorData;

    /// Get oxygen sensor 7 voltage and associated air/fuel short term trim
    ///
    /// If using two banks, this is for bank 2, sensor 3. If using four banks, this is for bank 4
    /// sensor 1.
    fn get_oxygen_sensor_7(0x01, 0x1A) -> OxygenSensorData;

    /// Get oxygen sensor 8 voltage and associated air/fuel short term trim
    ///
    /// If using two banks, this is for bank 2, sensor 4. If using four banks, this is for bank 4
    /// sensor 2.
    fn get_oxygen_sensor_8(0x01, 0x1B) -> OxygenSensorData;

    /// Get which OBD standard this vehicle is designed to support
    fn get_obd_requirements(0x01, 0x1C) -> u8;

    /// Get location of oxygen sensors
    ///
    /// This version (cf. [get_oxygen_sensors_2_bank](Self::get_oxygen_sensors_2_bank)) is
    /// recommended for four bank systems. A vehicle must not support both variants.
    ///
    /// The each pair of bits represents the sensors of one bank, the least significant pair is
    /// bank 1. The bits of the pair represent each of the two possible sensors, with sensor 1 in
    /// the less significant bit.
    fn get_oxygen_sensors_4_bank(0x01, 0x1D) -> u8;

    /// Get auxiliary input status
    ///
    /// The least significant bit indicates whether [power
    /// take-off](https://en.wikipedia.org/wiki/Power_Take_Off) is active.
    fn get_auxiliary_input_status(0x01, 0x1E) -> u8;

    /// Get the amount of time since the engine was started in seconds
    ///
    /// This should saturate—not roll over—after the engine has been running for [u16::MAX] seconds
    /// (≈18.2 hours).
    fn get_run_time(0x01, 0x1F) -> u16;

    /// Get service 1 PID support for $21 to $40
    fn get_service_1_pid_support_2(0x01, 0x20) -> u32;

    // Get the fuel level (out of 255)
    fn get_fuel_level(0x01, 0x2F) -> u8;
}
