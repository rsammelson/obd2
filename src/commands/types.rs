use std::fmt;

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
