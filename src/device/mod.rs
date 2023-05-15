mod elm327;
pub use elm327::Elm327;

type Result<T> = std::result::Result<T, Error>;

pub trait Obd2BaseDevice: Obd2Reader {
    fn reset(&mut self) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
    fn send_serial_cmd(&mut self, data: &str) -> Result<()>;
    fn cmd(&mut self, cmd: &str) -> Result<Option<String>> {
        self.send_serial_cmd(cmd)?;
        self.get_until_prompt()
            .map(|o| o.and_then(|resp| String::from_utf8(resp).ok()))
    }
}

pub trait Obd2Reader {
    fn get_line(&mut self) -> Result<Option<Vec<u8>>>;
    fn get_until_prompt(&mut self) -> Result<Option<Vec<u8>>>;
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("FTDI error: `{0:?}`")]
    Ftdi(ftdi::Error),
    #[error("IO error: `{0:?}`")]
    IO(std::io::Error),
    #[error("Communication error: `{0}`")]
    Communication(String),
}

impl From<ftdi::Error> for Error {
    fn from(e: ftdi::Error) -> Self {
        Error::Ftdi(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}
