//! Crate for communicating with OBD-II (on-board diagnostics) interfaces on cars
//!
//! Currently only the ELM327 is supported (many cheap USB to OBD-II devices you can buy online are
//! compatible with the ELM327). The high-level data retrieval functions can be found in
//! [commands::Obd2DataRetrieval].
//!
//! # Usage
//! ```
//! # #[cfg(feature = "ftdi_comm")] {
//! use obd2::{commands::Obd2DataRetrieval, device::{Elm327, FTDIDevice}, Obd2};
//!
//! fn main() -> Result<(), obd2::Error> {
//!     let mut device = Obd2::new(Elm327::new(FTDIDevice::new()?)?)?;
//!     println!("VIN: {}", device.get_vin()?);
//!     Ok(())
//! }
//! # }
//! ```
//!
//! Alternatively, you could use a serial port provided by your operating system such as /dev/ttyUSB0 on Unix-like
//! systems:
//! ```
//! # #[cfg(feature = "serialport_comm")] {
//! # fn main() -> Result<(), obd2::Error> {
//! use obd2::{device::{Elm327, SerialPort}, Obd2};
//! let mut device = Obd2::new(Elm327::new(SerialPort::new("/dev/ttyUSB0".to_owned())?)?)?;
//! # Ok(()) }}
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::panic)]

pub mod commands;

pub mod device;

mod error;
pub use error::Error;
use error::Result;

mod interface;
pub use interface::Obd2;

mod obd2_device;
pub use obd2_device::Obd2Device;
