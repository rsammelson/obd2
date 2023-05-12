mod obd2;

fn main() {
    env_logger::init();
    let mut device = obd2::Obd2::default();
    println!("VIN: {:?}", device.get_vin());
}
