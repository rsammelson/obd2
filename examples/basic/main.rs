use obd2::commands::Obd2DataRetrieval;

use std::time;

fn main() {
    env_logger::init();
    let mut device: obd2::Obd2<obd2::device::Elm327> = obd2::Obd2::default();

    println!("VIN: {:?}", device.get_vin());
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
        println!("Speed: {:?}", device.get_speed());
    }
}
