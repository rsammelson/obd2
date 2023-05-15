#![forbid(unsafe_code)]

mod accessors;
use accessors::Result;
pub use accessors::{Error, Obd2Device};

pub mod device;

mod interface;
pub use interface::Obd2;
