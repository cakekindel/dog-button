use std::{
    collections::HashMap,
    fs::File,
    io::{Cursor, Read},
    iter::once,
    thread, sync::Barrier,
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

pub struct Sound {
    pub sound: String,
    pub sound_source: Buffered<Decoder<Cursor<Vec<u8>>>>,
}

impl Sound {
    fn buffer(path: &str, loaded: &Barrier) -> Self {
        log::info!("loading sound {}", path);

        let mut file = File::open(path).expect("sound file should exist");
        let mut buf = vec![];
        file.read_to_end(&mut buf).ok();
        let buf = Cursor::new(buf);
        let source = Decoder::new(buf).unwrap().buffered();
        let source_clone = source.clone();
        let path_string = path.to_string();

        // UNSAFETY:
        //   This is safe because the reference is issued by Patch::get()
        //   which /also/ waits on this barrier before returning and dropping the barrier
        //   and invalidating all references to it.
        let loaded = unsafe {std::mem::transmute::<_, &'static Barrier>(loaded)};

        thread::spawn(move || {
            // This count() does more than just count the number
            // of decoded bytes.
            //
            // Iterating until the buffered source is exhausted will cache
            // the entire decoded sample in memory for clones, preventing
            // stuttering when attempting to decode on the fly
            let n = source_clone.count();
            log::info!("buffered {}kb", n / 1000);
            log::info!("loaded sound {}", path_string);

            loaded.wait();
        });

        Self {
            sound: path.to_string(),
            sound_source: source,
        }
    }

    pub fn play(&self, stream_handle: &OutputStreamHandle) {
        log::info!("playing {}", self.sound);
        let stream_handle = stream_handle.clone();
        let sound = self.sound.clone();
        let source = self.sound_source.clone();

        thread::spawn(move || {
            let sink = Sink::try_new(&stream_handle).expect("should be able to create sink");
            sink.append(source);
            sink.set_volume(4.0);
            sink.sleep_until_end();
            log::info!("played {}", sound);
        });
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
        log::info!("loading patch {}", path);

        let mut contents = String::new();
        File::open(path)
            .expect("patch should exist")
            .read_to_string(&mut contents)
            .expect("patch should be valid utf8");

        let raw = toml::from_str::<PatchRaw>(&contents).expect("patch should be valid toml");

        let loaded: Barrier = Barrier::new(raw.gpio.len() + 2);
        let loaded_ref = &loaded;

        let me = Self {
            sounds: raw
                .gpio
                .into_iter()
                .map(|(k, v)| {
                    (
                        SoundKey::Gpio(k.parse::<u16>().expect("gpio lanes must be integers")),
                        Sound::buffer(&v.sound, loaded_ref),
                    )
                })
                .chain(once((SoundKey::PowerOn, Sound::buffer("startup.wav", &loaded))))
                .collect(),
        };

        loaded.wait();

        me
    }
}
