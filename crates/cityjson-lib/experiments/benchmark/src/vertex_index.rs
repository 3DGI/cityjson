//! Vertex-index architecture.
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use memmap2::{Mmap, MmapOptions};
use serde::de::{EnumAccess, Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

// Deserialize into indexed CityJSON-like structures with serde
#[derive(Deserialize, Debug)]
struct SemanticSurface {
    #[serde(rename = "type")]
    semtype: String,
}

#[derive(Deserialize, Debug)]
struct Semantics {
    surfaces: Vec<SemanticSurface>,
    values: Vec<Vec<usize>>,
}

type Vertices = Vec<[i32; 3]>;

// Indexed geometry
type Vertex = usize;
type Ring = Vec<Vertex>;
type Surface = Vec<Ring>;
type Shell = Vec<Surface>;
type MultiSurface = Vec<Surface>;
type Solid = Vec<Shell>;

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum Geometry {
    MultiSurface {
        lod: String,
        boundaries: MultiSurface,
        semantics: Option<Semantics>,
    },
    Solid {
        lod: String,
        boundaries: Solid,
        semantics: Option<Semantics>,
    },
}

#[derive(Deserialize, Debug)]
struct CityObject {
    #[serde(rename = "type")]
    cotype: String,
    geometry: Vec<Geometry>,
}

#[derive(Deserialize, Default, Debug)]
struct Transform {
    scale: [f64; 3],
    translate: [f64; 3],
}

#[derive(Deserialize, Debug)]
struct CityModel {
    #[serde(rename = "type")]
    cmtype: String,
    version: String,
    transform: Transform,
    #[serde(rename = "CityObjects")]
    cityobjects: HashMap<String, CityObject>,
    vertices: Vertices,
}

#[derive(Deserialize, Debug)]
struct CMCObjects {
    #[serde(rename = "type")]
    #[serde(skip)]
    cmtype: String,
    #[serde(skip)]
    version: String,
    #[serde(skip)]
    transform: Transform,
    #[serde(rename = "CityObjects")]
    cityobjects: HashMap<String, CityObject>,
    #[serde(skip)]
    vertices: Vertices,
}

#[derive(Deserialize, Debug)]
struct CMVertices {
    #[serde(rename = "type")]
    cmtype: String,
    version: String,
    transform: Transform,
    #[serde(skip)]
    cityobjects: HashMap<String, CityObject>,
    vertices: Vertices,
}

fn for_each<'de, D, K, V, F>(deserializer: D, f: F) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
    F: FnMut(K, V),
{
    struct MapVisitor<K, V, F>(F, PhantomData<K>, PhantomData<V>);

    impl<'de, K, V, F> Visitor<'de> for MapVisitor<K, V, F>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
        F: FnMut(K, V),
    {
        type Value = ();

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a nonempty sequence")
        }

        fn visit_map<A>(mut self, mut seq: A) -> Result<(), A::Error>
        where
            A: MapAccess<'de>,
        {
            while let Some((coid, value)) = seq.next_entry::<K, V>()? {
                // println!("we are at {}", coid);
                self.0(coid, value)
            }
            Ok(())
        }
    }
    let visitor = MapVisitor(f, PhantomData, PhantomData);
    deserializer.deserialize_map(visitor)
}

#[derive(Deserialize, Debug)]
struct CMCObjectsTest {
    #[serde(rename = "CityObjects")]
    cityobjects: HashMap<String, CityObject>,
}

pub fn vindex_deserialize(path_in: PathBuf) {
    let mut cm_vertices: CMVertices;
    let mut cm: CMCObjects;

    {
        let path_in = "/home/balazs/Development/cjlib/experiments/data/dummy.json";
        let mut file = File::open(&path_in).expect("Couldn't open CityJSON file");
        let mmap = unsafe { Mmap::map(&file).expect("Cannot memmap the file") };
        // let reader = BufReader::new(&file);
        cm_vertices = serde_json::from_slice(&mmap).expect("Couldn't deserialize the Vertices");
    }

    let mut file = File::open(&path_in).expect("Couldn't open CityJSON file");
    let reader = BufReader::new(&file);
    // cm = serde_json::from_reader(reader).expect("Couldn't deserialize into CityModel");
    let mmap = unsafe { Mmap::map(&file).expect("Cannot memmap the file") };
    let mut deserializer = serde_json::Deserializer::from_slice(&mmap);
    // let mut deserializer = serde_json::Deserializer::from_reader(reader);
    for_each(&mut deserializer, |key: String, value: CityObject| {
        println!("key {} value {:#?}", key, value);
    })
    .expect("error");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_data() -> PathBuf {
        // let p = "/home/balazs/Development/cjlib/experiments/data/3dbag_v210908_fd2cee53_5786_bench.city.json";
        let p = "../data/simple.json";
        Path::new(p)
            .canonicalize()
            .expect("Could not find the INPUT file.")
    }

    #[test]
    fn test_vindex_deserialize() {
        let path_in = get_data();
        vindex_deserialize(path_in)
    }

    #[test]
    fn test_vindex_deserialize_debug() {
        let path_in = get_data();
        let mut file = File::open(path_in).expect("Couldn't open CityJSON file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .expect("Couldn't read CityJSON file contents");
        // let reader = BufReader::new(file);
        let cm: CMCObjects =
            serde_json::from_slice(&buffer[..]).expect("Couldn't deserialize into CityModel");
    }
}
