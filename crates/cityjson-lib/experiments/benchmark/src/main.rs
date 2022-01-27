#![allow(unused, irrefutable_let_patterns)]
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use clap::{crate_version, App, Arg};

mod direct_json;
use crate::direct_json::*;

mod dereference;
use crate::dereference::*;

mod vertex_index;
use crate::vertex_index::*;

// CLI -------------------------

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
            Architectures::VertexIndex => vindex_deserialize(path_in),
            Architectures::Dereference => deref_deserialize(path_in),
        }
    }
    fn serialize(&self, path_in: PathBuf) {
        match self {
            Architectures::DirectJson => {
                println!("Not implemented")
            }
            Architectures::VertexIndex => {
                println!("Not implemented")
            }
            Architectures::Dereference => {
                println!("Not implemented")
            }
        }
    }
    fn geometry(&self, path_in: PathBuf) {
        match self {
            Architectures::DirectJson => direct_geometry(path_in),
            Architectures::VertexIndex => {
                println!("Not implemented")
            }
            Architectures::Dereference => deref_geometry(path_in),
        }
    }
    fn semantics(&self, path_in: PathBuf) {
        match self {
            Architectures::DirectJson => direct_semantics(path_in),
            Architectures::VertexIndex => {
                println!("Not implemented")
            }
            Architectures::Dereference => deref_semantics(path_in),
        }
    }
    fn create(&self, path_in: PathBuf) {
        match self {
            Architectures::DirectJson => direct_create(path_in),
            Architectures::VertexIndex => {
                println!("Not implemented")
            }
            Architectures::Dereference => deref_create(path_in),
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
