use gpio::{GpioIn, GpioValue};
use rodio::source::Buffered;
use rodio::Sink;
use rodio::{source::Source, Decoder, OutputStream};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read};
use std::thread::sleep;
use std::time::Duration;

fn read_sound(path: &str) -> Buffered<Decoder<Cursor<Vec<u8>>>> {
    let mut file = File::open(path).expect("sound file should exist");
    let mut buf = vec![];
    file.read_to_end(&mut buf).ok();
    let buf = Cursor::new(buf);
    let source = Decoder::new(buf).unwrap().buffered();
    source.clone().count();
    source
}

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
        struct ProfileRaw {
            gpio: HashMap<String, GpioConfig>,
        }

        let path = format!(
            "profiles/{}",
            std::env::var("DOG_BTN_PROFILE").unwrap_or(String::from("default.toml"))
        );

        let mut contents = String::new();
        File::open(path)
            .expect("profile should exist")
            .read_to_string(&mut contents)
            .expect("profile should be valid utf8");

        let raw =
            toml::from_str::<ProfileRaw>(&contents).expect("profile should be valid toml");

        Self {
            gpio: raw
                .gpio
                .into_iter()
                .map(|(k, v)| {
                    (
                        u16::from_str_radix(&k, 10).expect("gpio keys must be integers"),
                        v,
                    )
                })
                .collect(),
        }
    }
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let profile = Profile::get();
    let (_stream, stream_handle) = OutputStream::try_default().expect("audio should be available");

    loop {
        profile.gpio.iter().for_each(|(lane, config)| {
            if gpio::sysfs::SysFsGpioInput::open(*lane)
                .expect("gpio lane should exist")
                .read_value()
                .expect("gpio lane should be input")
                == GpioValue::High
            {
                log::info!("lane {} high", lane);

                let sound = read_sound(&config.sound);

                let sink = Sink::try_new(&stream_handle).expect("should be able to create sink");
                sink.append(sound);
                sink.sleep_until_end();
            } else {
                ()
            }
        });

        sleep(Duration::from_millis(10));
    }
}
