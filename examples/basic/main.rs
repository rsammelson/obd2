use obd2::commands::Obd2DataRetrieval;

use std::time;

fn main() -> Result<(), obd2::Error> {
    env_logger::init();
    let mut device: obd2::Obd2<obd2::device::Elm327<obd2::device::FTDIDevice>> =
        obd2::Obd2::new(obd2::device::Elm327::new(obd2::device::FTDIDevice::new()?)?)?;

    println!("VIN: {:?}", device.get_vin());
    for s in device.get_service_1_pid_support_1()?.iter() {
        println!("PID support ($01-$20): {:08X}", s);
    }
    for s in device.get_service_1_pid_support_2()?.iter() {
        println!("PID support ($21-$40): {:08X}", s);
    }

    println!("DTC Info: {:#?}", device.get_dtc_info());

    let dtcs = device.get_dtcs();
    println!("DTCs: {:?}", dtcs);
    if let Ok(dtcs) = dtcs {
        for (i, response) in dtcs.iter().enumerate() {
            println!("DTCs from response {}:", i);
            for dtc in response {
                println!("  - {}", dtc);
            }
        }
    }

    let state = time::Instant::now();
    while state.elapsed() < time::Duration::from_secs(5) {
        println!("");
        println!(
            "Coolant Temperature: {:?}",
            device.get_engine_coolant_temperature()
        );
        println!("RPM: {:?}", device.get_rpm());
        println!("Speed (km/h): {:?}", device.get_speed());
        println!("Timing Advance (º): {:?}", device.get_timing_advance());
        println!(
            "Intake air temp (ºC): {:?}",
            device.get_intake_air_temperature()
        );
        println!("Air flow rate (g/s): {:?}", device.get_air_flow_rate());
        println!(
            "Throttle position (%): {:?}",
            device.get_throttle_position()
        );
    }

    Ok(())
}
