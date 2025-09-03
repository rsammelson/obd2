use std::io::Write as _;

use crate::{
    commands::{self, Obd2DataRetrieval as _},
    device::elm327,
};

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

#[test]
fn test_dtc_info() {
    let mut device = crate::Obd2::new(
        elm327::Elm327::new(elm327::test::TestDevice::with_cmd_process(|cmd, input| {
            assert_eq!(cmd, "0101");
            input.write_all(b"41 01 81 07 65 04\r>").unwrap();
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        device.get_dtc_info().unwrap(),
        vec![commands::DtcsInfo {
            malfunction_indicator_light: true,
            dtc_count: 1,
            common_test_availability: 7,
            is_compression_engine: false,
            specific_test_availability: 0x465,
        }]
    );
}

#[test]
fn test_dtcs_1() {
    let mut device = crate::Obd2::new(
        elm327::Elm327::new(elm327::test::TestDevice::with_cmd_process(|cmd, input| {
            assert_eq!(cmd, "03");
            input.write_all(b"43 01 33 00 00 00 00\r>").unwrap();
        }))
        .unwrap(),
    )
    .unwrap();
    let dtcs = device.get_dtcs().unwrap();
    assert_eq!(dtcs.len(), 1);
    assert_eq!(dtcs[0], vec![commands::Dtc::Powertrain(0x133)]);
}

#[test]
fn test_dtcs_2() {
    let mut device = crate::Obd2::new(
        elm327::Elm327::new(elm327::test::TestDevice::with_cmd_process(|cmd, input| {
            assert_eq!(cmd, "03");
            input.write_all(b"43 01 33 52 34 00 00\r>").unwrap();
        }))
        .unwrap(),
    )
    .unwrap();
    let dtcs = device.get_dtcs().unwrap();
    assert_eq!(dtcs.len(), 1);
    assert_eq!(
        dtcs[0],
        vec![
            commands::Dtc::Powertrain(0x133),
            commands::Dtc::Chassis(0x1234),
        ]
    );
}
