use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let file = BufReader::new(File::open("screm.wav").unwrap());
    let sink = stream_handle.play_once(file).unwrap();
    sink.set_volume(0.15);

    std::thread::sleep(std::time::Duration::from_secs(1));
}
