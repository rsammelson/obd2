use std::io::Write as _;

use crate::{commands::Obd2DataRetrieval as _, device::elm327};

#[test]
fn test_coolant_temp() {
    let mut device = crate::Obd2::new(
        elm327::Elm327::new(elm327::test::TestDevice::with_cmd_process(|cmd, input| {
            assert_eq!(cmd, "0105");
            input.write_all(b"41 05 7B\r>").unwrap();
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(device.get_engine_coolant_temperature().unwrap(), vec![83]);
}

#[test]
fn test_coolant_temp_double_response() {
    let mut device = crate::Obd2::new(
        elm327::Elm327::new(elm327::test::TestDevice::with_cmd_process(|cmd, input| {
            assert_eq!(cmd, "0105");
            input.write_all(b"41 05 7B\r\n41 05 01\r>").unwrap();
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        device.get_engine_coolant_temperature().unwrap(),
        vec![83, -39]
    );
}

#[test]
fn test_engine_rpm() {
    let mut device = crate::Obd2::new(
        elm327::Elm327::new(elm327::test::TestDevice::with_cmd_process(|cmd, input| {
            assert_eq!(cmd, "010C");
            input.write_all(b"41 0C 1A F8\r>").unwrap();
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(device.get_rpm().unwrap(), vec![1726.]);
}
