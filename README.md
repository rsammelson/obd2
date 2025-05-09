# `obd2`

This library provides a user-friendly interface to automatically configure an
[ELM327](https://github.com/rsammelson/obd2/blob/master/docs/ELM327DSH.pdf)
[OBD-II](https://en.wikipedia.org/wiki/OBD-II) to UART interface through an FTDI UART to USB interface (the entire
setup is easily available online as an OBD-II to USB interface), and then send commands and receive data from a
vehicle.

## Usage

```rs
use obd2::{commands::Obd2DataRetrieval, device::{Elm327, FTDIDevice}, Obd2};

fn main() -> Result<(), obd2::Error> {
    let mut device = Obd2::<Elm327::<FTDIDevice>>::new(Elm327::new(FTDIDevice::new()?)?)?;
    println!("VIN: {}", device.get_vin()?);
    Ok(())
}
```

alternatively, you could use a serial port provided by your operating system such as 
/dev/ttyUSB0 on unix-like systems

```rs
let mut device = Obd2::<Elm327::<SerialPort>>::new(Elm327::new(SerialPort::new("/dev/ttyUSB0")?)?)?;
```

See the docs for more: https://docs.rs/obd2/
