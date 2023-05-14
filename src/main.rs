mod obd2;

fn main() {
    env_logger::init();
    let mut device: obd2::Obd2<obd2::Elm327> = obd2::Obd2::default();

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
}
