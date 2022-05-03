//! Vertex-index architecture.
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use memmap2::MmapOptions;
use serde::Deserialize;
use zerovec::*;

// Deserialize into indexed CityJSON-like structures with serde
#[derive(Deserialize)]
struct SemanticSurface<'a> {
    #[serde(rename = "type")]
    semtype: &'a str,
}

#[derive(Deserialize)]
struct Semantics<'a> {
    #[serde(borrow)]
    surfaces: Vec<SemanticSurface<'a>>,
    #[serde(borrow)]
    values: Vec<ZeroVec<'a, u32>>,
}

type Vertices = Vec<[f64; 3]>;

// Indexed geometry
type Vertex = usize;
type Ring<'a> = ZeroVec<'a, u32>;
type Surface<'a> = Vec<Ring<'a>>;
type Shell<'a> = Vec<Surface<'a>>;
type MultiSurface<'a> = Vec<Surface<'a>>;
type Solid<'a> = Vec<Shell<'a>>;

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Geometry<'a> {
    MultiSurface {
        lod: &'a str,
        #[serde(borrow)]
        boundaries: MultiSurface<'a>,
        #[serde(borrow)]
        semantics: Option<Semantics<'a>>,
    },
    Solid {
        lod: &'a str,
        #[serde(borrow)]
        boundaries: Solid<'a>,
        semantics: Option<Semantics<'a>>,
    },
}

#[derive(Deserialize)]
struct CityObject<'a> {
    #[serde(rename = "type")]
    cotype: &'a str,
    #[serde(borrow)]
    geometry: Vec<Geometry<'a>>,
}

#[derive(Deserialize)]
struct Transform {
    scale: [f64; 3],
    translate: [f64; 3],
}

#[derive(Deserialize)]
struct CityModel<'a> {
    #[serde(rename = "type")]
    cmtype: &'a str,
    version: &'a str,
    transform: Transform,
    #[serde(borrow)]
    #[serde(rename = "CityObjects")]
    cityobjects: HashMap<&'a str, CityObject<'a>>,
    vertices: Vertices,
}

pub fn vindex_deserialize(path_in: PathBuf) {
    let mut file = File::open(path_in).expect("Couldn't open CityJSON file");
    // let mut buffer = Vec::new();
    // file.read_to_end(&mut buffer)
    //     .expect("Couldn't read CityJSON file contents");
    let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    let cm: CityModel = serde_json::from_slice(&mmap).expect("Couldn't deserialize into CityModel");
    // let reader = BufReader::new(file);
    // let cm: CityModel = serde_json::from_reader(reader).expect("Couldn't deserialize into CityModel");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_data() -> PathBuf {
        Path::new("/home/balazs/Development/cjlib/experiments/data/3dbag_v210908_fd2cee53_5786_bench.city.json")
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
        let cm: CityModel =
            serde_json::from_slice(&buffer[..]).expect("Couldn't deserialize into CityModel");
    }
}
