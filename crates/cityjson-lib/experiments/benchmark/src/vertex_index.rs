//! Vertex-index architecture.
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
        boundaries: MultiSurface,
        semantics: Option<Semantics>,
    },
    Solid {
        lod: String,
        boundaries: Solid,
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
    let reader = BufReader::new(file);
    let cm: CityModel =
        serde_json::from_reader(reader).expect("Couldn't deserialize into CityModel");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_data() -> PathBuf {
        Path::new("../data/3dbag_v210908_fd2cee53_5786_bench.city.json")
            .canonicalize()
            .expect("Could not find the INPUT file.")
    }

    #[test]
    fn test_vindex_deserialize() {
        let path_in = get_data();
        vindex_deserialize(path_in)
    }
}
