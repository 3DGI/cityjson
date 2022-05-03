use std::fmt;
use std::fmt::Formatter;
/// Deserializing an array of values without buffering into a Vec.
/// As it is illustrated by the serde example in https://serde.rs/stream-array.html
/// and in https://github.com/serde-rs/json/issues/160#issuecomment-841344394.
use std::fs::File;
use std::io::BufReader;
use std::marker::PhantomData;
use std::path::Path;

use clap::{crate_version, App, Arg};
use memmap2::MmapOptions;
use serde::de::{EnumAccess, Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
struct CityJSON {
    version: String,
    #[serde(deserialize_with = "deserialize_vertices")]
    vertices: Vec<[f64; 3]>,
}

fn deserialize_vertices<'de, D>(deserializer: D) -> Result<Vec<[f64; 3]>, D::Error>
where
    D: Deserializer<'de>,
{
    struct SeqVisitor(PhantomData<fn() -> Vec<[f64; 3]>>);

    impl<'de> Visitor<'de> for SeqVisitor {
        type Value = Vec<[f64; 3]>;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            formatter.write_str("the 'vertices' array of a CityJSON file")
        }

        fn visit_seq<S>(mut self, mut seq: S) -> Result<Vec<[f64; 3]>, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut v: Vec<[f64; 3]>;
            if let Some(vsize) = seq.size_hint() {
                v = Vec::with_capacity(vsize);
            } else {
                v = Vec::new();
            }
            while let Some(value) = seq.next_element::<[f64; 3]>()? {
                v.push(value);
            }
            v.shrink_to_fit();
            Ok(v)
        }
    }
    let visitor = SeqVisitor(PhantomData);
    deserializer.deserialize_seq(visitor)
}

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

    let file = File::open(path_in).expect("Couldn't read CityJSON file");
    // let reader = BufReader::new(file);
    // let _cm: CityJSON =
    //     serde_json::from_reader(reader).expect("Couldn't deserialize into CityModel");

    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    let _cm: CityJSON = serde_json::from_slice(&mmap).expect("Couldn't deserialize into CityModel");

    println!("number of vertices: {}", _cm.vertices.len());
}
