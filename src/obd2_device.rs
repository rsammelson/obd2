use crate::{Error, Result};

/// A higher-level API for using an OBD-II device
pub trait Obd2Device {
    /// Send an OBD-II command with mode and PID and get responses
    ///
    /// The responses are a list with one element for each ECU that responds. The data is decoded
    /// into the ODB-II bytes from the vehicle and the first two bytes of the
    /// response---representing the mode and PID the vehicle received---are validated and removed.
    fn obd_command(&mut self, mode: u8, pid: u8) -> Result<Vec<Vec<u8>>>;

    /// Send an OBD-II command with only mode and get responses
    ///
    /// The responses are a list with one element for each ECU that responds. The data is decoded
    /// into the ODB-II bytes from the vehicle and the first byte of the response---representing
    /// the mode the vehicle received---is validated and removed.
    fn obd_mode_command(&mut self, mode: u8) -> Result<Vec<Vec<u8>>>;

    /// Send command and get list of OBD-II responses as an array
    ///
    /// Like [obd_command](Self::obd_command), but each ECU's response (after removing the first
    /// two bytes) is converted to an array of the specified length. If any response is the wrong
    /// length, and error is returned.
    ///
    /// This function can be used when the response length is known, so that it is easier to index
    /// into the response without causing a panic and without dealing with Options.
    fn obd_command_len<const RESPONSE_LENGTH: usize>(
        &mut self,
        mode: u8,
        pid: u8,
    ) -> Result<Vec<[u8; RESPONSE_LENGTH]>> {
        self.obd_command(mode, pid)?
            .into_iter()
            .map(|v| {
                let l = v.len();
                v.try_into()
                    .map_err(|_| Error::IncorrectResponseLength("length", RESPONSE_LENGTH, l))
            })
            .collect()
    }

    /// Send command and get array of OBD-II responses with each as an array
    ///
    /// Like [obd_command_len](Self::obd_command_len), but also convert the list of ECU responses
    /// to an array. This can be used when the number of ECUs that should respond is known in
    /// advance. Most commonly, this will be when the count of ECUs is one, for values where only a
    /// single ECU should respond like the speed of the vehicle.
    fn obd_command_cnt_len<const RESPONSE_COUNT: usize, const RESPONSE_LENGTH: usize>(
        &mut self,
        mode: u8,
        pid: u8,
    ) -> Result<[[u8; RESPONSE_LENGTH]; RESPONSE_COUNT]> {
        let result = self.obd_command_len::<RESPONSE_LENGTH>(mode, pid)?;
        let count = result.len();
        result
            .try_into()
            .map_err(|_| Error::IncorrectResponseLength("count", RESPONSE_COUNT, count))
    }
}
