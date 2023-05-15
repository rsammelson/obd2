#![forbid(unsafe_code)]

pub mod device;

mod interface;
pub use interface::Obd2;

mod obd2_device;
use obd2_device::Result;
pub use obd2_device::{Error, Obd2Device};
