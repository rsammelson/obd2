# `obd2`

This library provides a user-friendly interface to automatically configure an
[ELM327](https://github.com/rsammelson/obd2/blob/master/docs/ELM327DSH.pdf)
[OBD-II](https://en.wikipedia.org/wiki/OBD-II) to UART interface through an FTDI UART to USB interface (the entire
setup is easily available online as an OBD-II to USB interface), and then send commands and receive data from a
vehicle.

## Usage

```rs
use obd2::{commands::Obd2DataRetrieval, device::Elm327, Obd2};

fn main() -> Result<(), obd2::Error> {
    let mut device = Obd2::<Elm327>::default();
    println!("VIN: {}", device.get_vin()?);
    Ok(())
}
```

See the docs for more: https://docs.rs/obd2/
