//! Lower level OBD-II interfacing structures

mod elm327;
pub use elm327::Elm327;

mod serial_communication;
pub use serial_communication::SerialCommunication;

#[cfg(feature = "ftdi_comm")]
mod ftdi_comm;
#[cfg(feature = "ftdi_comm")]
pub use ftdi_comm::FTDIDevice;

#[cfg(feature = "serialport_comm")]
mod serialport_comm;
#[cfg(feature = "serialport_comm")]
pub use serialport_comm::SerialPort;

type Result<T> = std::result::Result<T, Error>;

/// A lower-level API for using an OBD-II device
pub trait Obd2BaseDevice: Obd2Reader {
    /// Reset the device and the OBD-II interface
    ///
    /// First the device is reset, if it is stateful. Then the OBD-II interface is reinitialized,
    /// which resets the selected protocol on the device and rechecks the vehicle manufacturer if
    /// needed.
    fn reset(&mut self) -> Result<()>;

    /// Send an OBD-II command
    fn send_cmd(&mut self, data: &[u8]) -> Result<()>;

    /// Send an OBD-II command and get the reply
    ///
    /// The reply is decoded into a String of mostly hex data. Depending on the format of the
    /// response, some other characters may be included like line numbers for multiline responses
    /// (of the format "0: AB CD ...").
    fn cmd(&mut self, cmd: &[u8]) -> Result<Option<String>> {
        self.send_cmd(cmd)?;
        self.get_response()
            .map(|o| o.and_then(|resp| String::from_utf8(resp).ok()))
    }
}

/// An API for reading OBD-II response data
pub trait Obd2Reader {
    /// Try to get a single line of data from the device
    ///
    /// The trailing newline is not included. This function will never return an empty line, it
    /// will retry until a line with data is found. If no data is available after a reasonable
    /// timeout, `Ok(None)` will be returned.
    fn get_line(&mut self) -> Result<Option<Vec<u8>>>;

    /// Get an entire OBD-II response
    ///
    /// Empty vectors are allowed to be returned. This function should always be called after a
    /// command is sent, possibly after calling [get_line](Self::get_line) to read the first lines,
    /// so that any metadata sent by the device after the response from the vehicle can be dealt
    /// with.
    fn get_response(&mut self) -> Result<Option<Vec<u8>>>;
}

/// Error type for low-level ODB-II communication issues
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error with the underlying FTDI device
    #[cfg(feature = "ftdi_comm")]
    #[error("FTDI error: `{0:?}`")]
    Ftdi(#[from] ftdi::Error),

    /// An error with the underlying [serialport::SerialPort]
    #[cfg(feature = "serialport_comm")]
    #[error("Serialport error: `{0:?}`")]
    Serialport(#[from] serialport::Error),

    /// An I/O error in a stream operation
    #[error("IO error: `{0:?}`")]
    IO(#[from] std::io::Error),

    /// An OBD-II or interface device protocol error
    #[error("Communication error: `{0}`")]
    Communication(String),
}
