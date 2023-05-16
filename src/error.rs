pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Device error: `{0:?}`")]
    Device(DeviceError),
    #[error("Other OBD2 error: `{0}`")]
    Other(String),
    #[error("Incorrect length (`{0}`): expected `{1}`, got `{2}`")]
    IncorrectResponseLength(&'static str, usize, usize),
}

#[derive(Debug)]
pub struct DeviceError(super::device::Error);

impl From<super::device::Error> for Error {
    fn from(e: super::device::Error) -> Self {
        Error::Device(DeviceError(e))
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::Other(format!("invalid data recieved: {:?}", e))
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::Other(format!("invalid string recieved: {:?}", e))
    }
}
