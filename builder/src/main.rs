use std::{env, fs, io::Read, path::Path};
use xz2::bufread::XzEncoder;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_filename = args.get(1).unwrap().to_string();
    let payload = fs::read(input_filename).unwrap();

    let mut compressed_payload = Vec::new();
    let mut compressor = XzEncoder::new(&payload[..], 9);
    compressor.read_to_end(&mut compressed_payload).unwrap();

    fs::write(Path::new("compressed_payload.bin"), compressed_payload).unwrap();
}
