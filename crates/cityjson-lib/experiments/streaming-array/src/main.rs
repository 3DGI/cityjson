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
    vertices: i32,
}

fn deserialize_vertices<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    struct SeqVisitor(PhantomData<fn() -> i32>);

    impl<'de> Visitor<'de> for SeqVisitor {
        type Value = i32;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            formatter.write_str("the 'vertices' array of a CityJSON file")
        }

        fn visit_seq<S>(mut self, mut seq: S) -> Result<i32, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut len: i32 = 0;
            while let Some(value) = seq.next_element::<[f64; 3]>()? {
                len += 1;
            }
            Ok(len)
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
    // deserialize_vertices(&mut deserializer, |obj: CityJSON| todo!()).unwrap();

    println!("number of vertices: {}", _cm.vertices);
}
