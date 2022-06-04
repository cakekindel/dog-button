use gpio::{GpioIn, GpioValue};
use rodio::{source::Source, Decoder, OutputStream};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Cursor};
use std::thread::sleep;
use std::time::Duration;

#[derive(Deserialize)]
pub struct GpioConfig {
    pub sound: String,
}

pub struct Profile {
    pub gpio: HashMap<u16, GpioConfig>,
}

impl Profile {
    pub fn get() -> Self {
#[derive(Deserialize)]
struct ProfileRaw {gpio: HashMap<String, GpioConfig>}

        let path = format!(
            "profiles/{}",
            std::env::var("DOG_BTN_PROFILE").unwrap_or(String::from("default.toml"))
        );

        let mut contents = String::new();
        File::open(path)
            .expect("profile should exist")
            .read_to_string(&mut contents)
            .expect("profile should be valid utf8");

        let mut raw = toml::from_str::<ProfileRaw>(&contents).expect("profile should be valid toml");

Self {gpio:
        raw.gpio
            .into_iter()
            .map(|(k, v)| (u16::from_str_radix(&k, 10).expect("gpio keys must be integers"), v))
            .collect()
}
    }
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let profile = Profile::get();
    let (_stream, stream_handle) = OutputStream::try_default().expect("audio should be available");

    loop {
        profile
            .gpio
            .iter()
            .find_map(|(lane, config)| {
                if gpio::sysfs::SysFsGpioInput::open(*lane)
                    .expect("gpio lane should exist")
                    .read_value()
                    .expect("gpio lane should be input")
                    == GpioValue::High
                {
                    log::info!("lane {} high", lane);
                    Some(File::open(&config.sound).expect("sound should exist"))
                } else {
                    None
                }
            })
            .map(|mut file| {
                let mut buf = vec![];
                file.read_to_end(&mut buf).ok();
                let buf = Cursor::new(buf);
                let sink = stream_handle.play_once(buf).expect("sound should be valid");
                sink.sleep_until_end();
            });

        sleep(Duration::from_millis(10));
    }
}
