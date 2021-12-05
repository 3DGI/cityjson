use std::borrow::Borrow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use clap::{crate_version, App, Arg};
use serde_json::Value;

/// Parse a CityJSON file as-is, just by using serde_json's generic Value type.
/// Not using any CityJSON specific structure.
/// This is the quickest to get started, but requires lots of code later on in order to unwrap the
/// individual JSON members.
fn direct_deserialize(path_in: PathBuf) {
    let str_dataset = std::fs::read_to_string(path_in).expect("Couldn't read CityJSON file");
    let re: Result<Value, _> = serde_json::from_str(&str_dataset);
    let cm = re.expect("Could not deserialize the CityJSON file");
}

/// Get the boundary coordinates of each surface.
fn direct_geometry(path_in: PathBuf) {
    let mut containter: Vec<[f64; 3]> = Vec::new();
    let str_dataset = std::fs::read_to_string(path_in).expect("Couldn't read CityJSON file");
    let re: Result<Value, _> = serde_json::from_str(&str_dataset);
    let cm = re.expect("Could not deserialize the CityJSON file");
    let cos = cm["CityObjects"].as_object().unwrap();
    let vertices = cm["vertices"].as_array().unwrap();
    for coval in cos.values() {
        let geometry = coval
            .as_object()
            .unwrap()
            .get("geometry")
            .unwrap()
            .as_array()
            .unwrap();
        for geom in geometry {
            // Really need to be careful with the data types in the file
            println!("LoD: {}", geom["lod"].as_f64().unwrap());
            if geom["type"].as_str().unwrap() == "Solid" {
                for shell in geom["boundaries"].as_array().unwrap() {
                    for surface in shell.as_array().unwrap() {
                        for ring in surface.as_array().unwrap() {
                            for vtx_idx in ring.as_array().unwrap() {
                                let v = vertices[vtx_idx.as_u64().unwrap() as usize]
                                    .as_array()
                                    .unwrap();
                                let point: [f64; 3] = [
                                    v[0].as_f64().unwrap().clone(),
                                    v[1].as_f64().unwrap().clone(),
                                    v[2].as_f64().unwrap().clone(),
                                ];
                                containter.push(point);
                            }
                        }
                    }
                }
            } else {
                println!("Not a Solid geometry")
            }
        }
    }
    println!(
        "In total there are {} points in the citymodel and {} vertices",
        containter.len(),
        vertices.len()
    )
}

static USECASES: [&str; 5] = [
    "deserialize",
    "serialize",
    "geometry",
    "semantics",
    "create",
];

enum Architectures {
    DirectJson,
    VertexIndex,
    Dereference,
}

impl Architectures {
    fn run_usecase(&self, case: &str, path_in: PathBuf) {
        match case {
            "deserialize" => self.deserialize(path_in),
            "serialize" => self.serialize(path_in),
            "geometry" => self.geometry(path_in),
            "semantics" => self.semantics(path_in),
            "create" => self.create(path_in),
            _ => {}
        }
    }
    fn deserialize(&self, path_in: PathBuf) {
        match self {
            Architectures::DirectJson => direct_deserialize(path_in),
            Architectures::VertexIndex => {}
            Architectures::Dereference => {}
        }
    }
    fn serialize(&self, path_in: PathBuf) {
        match self {
            Architectures::DirectJson => {}
            Architectures::VertexIndex => {}
            Architectures::Dereference => {}
        }
    }
    fn geometry(&self, path_in: PathBuf) {
        match self {
            Architectures::DirectJson => direct_geometry(path_in),
            Architectures::VertexIndex => {}
            Architectures::Dereference => {}
        }
    }
    fn semantics(&self, path_in: PathBuf) {
        match self {
            Architectures::DirectJson => {}
            Architectures::VertexIndex => {}
            Architectures::Dereference => {}
        }
    }
    fn create(&self, path_in: PathBuf) {
        match self {
            Architectures::DirectJson => {}
            Architectures::VertexIndex => {}
            Architectures::Dereference => {}
        }
    }
}

fn main() {
    let dispatch_architecture = HashMap::from([
        ("direct-json", Architectures::DirectJson),
        ("vertex-index", Architectures::VertexIndex),
        ("dereference", Architectures::Dereference),
    ]);
    let archs: Vec<&str> = dispatch_architecture.keys().cloned().collect();

    let app = App::new("benchmark")
        .about("Benchmark the potential cjlib architectures")
        .version(crate_version!())
        .arg(
            Arg::with_name("ARCH")
                .short("a")
                .long("architecture")
                .required(true)
                .help("The cjlib architecture")
                .takes_value(true)
                .possible_values(&archs),
        )
        .arg(
            Arg::with_name("CASE")
                .short("c")
                .long("case")
                .required(true)
                .help("The use case")
                .takes_value(true)
                .possible_values(&USECASES),
        )
        .arg(
            Arg::with_name("INPUT")
                .required(true)
                .help("CityJSON file to benchmark."),
        );
    let matches = app.get_matches();

    let path_in = Path::new(matches.value_of("INPUT").unwrap())
        .canonicalize()
        .expect("Could not find the INPUT file.");

    let arch = dispatch_architecture
        .get(&matches.value_of("ARCH").unwrap())
        .unwrap();
    arch.run_usecase(matches.value_of("CASE").unwrap(), path_in)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_data() -> PathBuf {
        Path::new("../data/cluster_bench.city.json")
            .canonicalize()
            .expect("Could not find the INPUT file.")
    }

    #[test]
    fn test_direct_geometry() {
        let path_in = get_data();
        direct_geometry(path_in)
    }
}

// fn main() -> Result<(), serde_json::Error> {
//     let file_path = "../data/cluster.city.json";
//     let filesize = fs::metadata(file_path).unwrap().len();
//     println!("size of citjson file: {}", filesize);
//     let str_dataset = fs::read_to_string(&file_path)
//         .expect("Couldn't read CityJSON file");
//     println!("str_dataset alignment: {}, size: {}", mem::size_of_val(&str_dataset), mem::size_of_val(&*str_dataset));
//     let j: serde_json::Value = serde_json::from_str(&str_dataset)?;
//     println!("j alignment: {}, size: {}", mem::size_of_val(&j), mem::size_of_val(&j));
//     let cos = j.get("CityObjects").unwrap().as_object().unwrap();
//     println!("cos alignment: {}, size: {}", mem::size_of_val(&cos), mem::size_of_val(&cos));
//     for coid in cos.keys() {
//         println!("CityObject {} is of type {}", coid, j["CityObjects"][coid]["type"])
//     }
//     Ok(())
// }
