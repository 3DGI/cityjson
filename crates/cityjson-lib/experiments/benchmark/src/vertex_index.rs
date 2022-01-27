use memmap2::Mmap;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use serde::Deserialize;

// Deserialize into indexed CityJSON-like structures with serde
#[derive(Deserialize)]
struct SemanticSurface {
    #[serde(rename = "type")]
    semtype: String,
}

#[derive(Deserialize)]
struct Semantics {
    surfaces: Vec<SemanticSurface>,
    values: Vec<Vec<usize>>,
}

type Vertices = Vec<[f64; 3]>;

// Indexed geometry
type Vertex = usize;
type Ring = Vec<Vertex>;
type Surface = Vec<Ring>;
type Shell = Vec<Surface>;
type MultiSurface = Vec<Surface>;
type Solid = Vec<Shell>;

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Geometry {
    MultiSurface {
        lod: String,
        boundaries: Vec<Vec<Vec<usize>>>,
        semantics: Option<Semantics>,
    },
    Solid {
        lod: String,
        boundaries: Vec<Vec<Vec<Vec<usize>>>>,
        semantics: Option<Semantics>,
    },
}

#[derive(Deserialize)]
struct CityObject {
    #[serde(rename = "type")]
    cotype: String,
    geometry: Vec<Geometry>,
}

#[derive(Deserialize)]
struct Transform {
    scale: [f64; 3],
    translate: [f64; 3],
}

#[derive(Deserialize)]
struct CityModel {
    #[serde(rename = "type")]
    cmtype: String,
    version: String,
    transform: Transform,
    #[serde(rename = "CityObjects")]
    cityobjects: HashMap<String, CityObject>,
    vertices: Vertices,
}

pub fn vindex_deserialize(path_in: PathBuf) {
    let file = File::open(path_in).expect("Couldn't read CityJSON file");
    let mmap = unsafe { memmap2::Mmap::map(&file) }.unwrap();
    let cm: CityModel = serde_json::from_slice(&mmap).expect("Couldn't deserialize into CityModel");
}
