use std::{
    collections::HashMap,
    fs::File,
    io::{Cursor, Read},
    iter::once,
};

use rodio::{source::Buffered, Decoder, OutputStreamHandle, Sink, Source};
use serde::Deserialize;

#[derive(Deserialize)]
struct SoundRaw {
    sound: String,
}

#[derive(Deserialize)]
struct PatchRaw {
    gpio: HashMap<String, SoundRaw>,
}

impl From<SoundRaw> for Sound {
    fn from(other: SoundRaw) -> Sound {
        Sound::buffer(&other.sound)
    }
}

pub struct Sound {
    pub sound: String,
    pub sound_source: Buffered<Decoder<Cursor<Vec<u8>>>>,
}

impl Sound {
    fn buffer(path: &str) -> Self {
        let mut file = File::open(path).expect("sound file should exist");
        let mut buf = vec![];
        file.read_to_end(&mut buf).ok();
        let buf = Cursor::new(buf);
        let source = Decoder::new(buf).unwrap().buffered();
        source.clone().for_each(|_| ());

        Self {
            sound: path.to_string(),
            sound_source: source,
        }
    }

    pub fn play(&self, stream_handle: &OutputStreamHandle) {
        let sink = Sink::try_new(stream_handle).expect("should be able to create sink");
        sink.append(self.sound_source.clone());
        sink.sleep_until_end();
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SoundKey {
    Gpio(u16),
    PowerOn,
}

pub struct Patch {
    pub sounds: HashMap<SoundKey, Sound>,
}

impl Patch {
    pub fn get() -> Self {
        let path = format!(
            "patches/{}.toml",
            std::env::var("DOG_BTN_PATCH").unwrap_or_else(|_| String::from("default"))
        );

        let mut contents = String::new();
        File::open(path)
            .expect("patch should exist")
            .read_to_string(&mut contents)
            .expect("patch should be valid utf8");

        let raw = toml::from_str::<PatchRaw>(&contents).expect("patch should be valid toml");

        Self {
            sounds: raw
                .gpio
                .into_iter()
                .map(|(k, v)| {
                    (
                        SoundKey::Gpio(k.parse::<u16>().expect("gpio lanes must be integers")),
                        Sound::from(v),
                    )
                })
                .chain(once((SoundKey::PowerOn, Sound::buffer("startup.wav"))))
                .collect(),
        }
    }
}
