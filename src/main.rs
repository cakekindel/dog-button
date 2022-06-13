#![allow(clippy::single_match)]
#![allow(clippy::option_map_unit_fn)]
#![allow(clippy::result_map_unit_fn)]

use gpio::{GpioIn, GpioValue};
use patch::SoundKey;
use rodio::OutputStream;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

mod patch;

fn gpio_is_lo(lane: &u16) -> bool {
    gpio::sysfs::SysFsGpioInput::open(*lane)
        .unwrap_or_else(|_| panic!("gpio lane {} should exist", lane))
        .read_value()
        .unwrap_or_else(|_| panic!("gpio lane {} should be an input", lane))
        == GpioValue::Low
}

fn main() {
    std::env::set_var(
        "RUST_LOG",
        "dog_button=info,symphonia_core=error,symphonia_bundle_mp3=error",
    );
    env_logger::init();

    let mut gpio_was_hi = HashMap::<u16, bool>::new();

    let profile = patch::Patch::get();
    let (_stream, stream_handle) = OutputStream::try_default().expect("audio should be available");

    profile
        .sounds
        .get(&SoundKey::PowerOn)
        .map(|s| s.play(&stream_handle));

    loop {
        profile.sounds.iter().for_each(|(key, sound)| {
            if let SoundKey::Gpio(lane) = key {
                if gpio_is_lo(lane) && !gpio_was_hi.get(lane).copied().unwrap_or_default() {
                    gpio_was_hi.insert(*lane, true);
                    log::info!("lane {} high", lane);
                    sound.play(&stream_handle);
                } else if !gpio_is_lo(lane) {
                    gpio_was_hi.insert(*lane, false);
                }
            }
        });

        sleep(Duration::from_millis(10));
    }
}
