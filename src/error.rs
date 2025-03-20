pub type Result<T> = std::result::Result<T, Error>;

/// An error with OBD-II communication
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error occurred in the [Odb2BaseDevice](crate::device::Obd2BaseDevice)
    #[error("Device error: `{0:?}`")]
    Device(DeviceError),

    /// Some part of the response (described by the `&str`) was not the expected length
    #[error("Incorrect length (`{0}`): expected `{1}`, got `{2}`")]
    IncorrectResponseLength(&'static str, usize, usize),

    /// Another error occurred
    #[error("Other OBD2 error: `{0}`")]
    Other(String),
}

#[derive(Debug)]
pub struct DeviceError(crate::device::Error);

impl From<super::device::Error> for Error {
    fn from(e: super::device::Error) -> Self {
        Error::Device(DeviceError(e))
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::Other(format!("invalid data received: {:?}", e))
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::Other(format!("invalid string received: {:?}", e))
    }
}
