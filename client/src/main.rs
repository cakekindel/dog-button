use std::thread::sleep;
use std::time::Duration;

use gpio::{GpioIn, GpioValue};
use kwap::blocking::Client;
use kwap::net::Addrd;
use kwap::platform::Std;
use kwap::req::Req;

fn main() {
    simple_logger::init_with_level(log::Level::Trace).unwrap();

    let Addrd(_, addr) = Client::<Std>::listen_multicast(kwap::std::Clock::new(), 1234).unwrap();
    log::info!("found server: {:?}", addr);

    let mut client = Client::new_std();

    let mut gpio1 = gpio::sysfs::SysFsGpioInput::open(1).unwrap();

    loop {
        match gpio1.read_value().unwrap() {
            GpioValue::High => {
                client.send(Req::post(addr, "pressed")).unwrap();
            }
            GpioValue::Low => sleep(Duration::from_millis(50)),
        }
    }
}
