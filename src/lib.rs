//! Crate for communicating with OBD-II (on-board diagnostics) interfaces on cars
//!
//! Currently only the ELM327 is supported (many cheap USB to OBD-II devices you can buy online are
//! compatible with the ELM327). The high-level data retrieval functions can be found in
//! [commands::Obd2DataRetrieval].
//!
//! # Usage
//! ```
//! use obd2::{commands::Obd2DataRetrieval, device::Elm327, Obd2};
//!
//! fn main() -> Result<(), obd2::Error> {
//!     let mut device = Obd2::<Elm327>::default();
//!     println!("VIN: {}", device.get_vin()?);
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod commands;

pub mod device;

mod error;
pub use error::Error;
use error::Result;

mod interface;
pub use interface::Obd2;

mod obd2_device;
pub use obd2_device::Obd2Device;
