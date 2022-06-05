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
    source.clone().for_each(|_| ());
    source
}

pub struct GpioConfig {
    pub sound: String,
    pub sound_source: Buffered<Decoder<Cursor<Vec<u8>>>>,
}

pub struct Profile {
    pub gpio: HashMap<u16, GpioConfig>,
}

impl Profile {
    pub fn get() -> Self {
        #[derive(Deserialize)]
        struct GpioConfigRaw {sound: String}
        #[derive(Deserialize)]
        struct ProfileRaw {
            gpio: HashMap<String, GpioConfigRaw>,
        }

        let path = format!(
            "profiles/{}",
            std::env::var("DOG_BTN_PROFILE").unwrap_or_else(|_| String::from("default.toml"))
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
                        k.parse::<u16>().expect("gpio keys must be integers"),
                        GpioConfig {
                            sound_source: read_sound(&v.sound),
                            sound: v.sound,
                        },
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

                let sink = Sink::try_new(&stream_handle).expect("should be able to create sink");
                sink.append(config.sound_source.clone());
                sink.sleep_until_end();
            }
        });

        sleep(Duration::from_millis(10));
    }
}
