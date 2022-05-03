/// Deserializing an array of values without buffering into a Vec.
/// As it is illustrated by the serde example in https://serde.rs/stream-array.html
/// and in https://github.com/serde-rs/json/issues/160#issuecomment-841344394.
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use clap::{crate_version, App, Arg};
use memmap2::MmapOptions;

fn main() {
    let app = App::new("streaming-array")
        .about("Streaming JSON array deserialization test")
        .version(crate_version!())
        .arg(
            Arg::with_name("INPUT")
                .required(true)
                .help("CityJSON file to deserialize."),
        );
    let matches = app.get_matches();

    let path_in = Path::new(matches.value_of("INPUT").unwrap())
        .canonicalize()
        .expect("Could not find the INPUT file.");

    let mut file = File::open(path_in).expect("Couldn't read CityJSON file");
    // let mut buffer = Vec::new();
    // file.read_to_end(&mut buffer)
    //     .expect("Couldn't read CityJSON file");

    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
}
