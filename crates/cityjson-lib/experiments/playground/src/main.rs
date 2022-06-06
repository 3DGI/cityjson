#![allow(dead_code, unused_variables)]
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use dereference::*;

type Point = [f64; 3];

type Vertices<'vertices> = Vec<Point>;

type LineString<'vertices> = Vec<&'vertices Point>;

struct CityObject<'vertices> {
    geometry: LineString<'vertices>,
}

struct CityModel<'vertices> {
    cityobjects: HashMap<String, CityObject<'vertices>>,
    vertices: Vertices<'vertices>,
}

fn main() {
    // let mut cm = CityModel {
    //     cityobjects: HashMap::new(),
    //     vertices: Vertices::new(),
    // };
    // let v: Vertices = vec![[12.0, 12.0, 12.0], [12.0, 12.0, 12.0], [12.0, 12.0, 12.0]];
    // let mut l: LineString = Vec::new();
    // l.push(&v[0]);
    // l.push(&v[1]);
    // l.push(&v[2]);
    //
    // let co: CityObject = CityObject { geometry: l };
    // let mut cos: HashMap<String, CityObject> = HashMap::new();
    // cos.insert("id1".to_string(), co);
    //
    // for (coid, co) in cm.cityobjects {
    //     let p1 = co.geometry[0];
    //     println!("{}", p1[0] * 2.0);
    // }
    // ----------------------
    // fn get_data() -> PathBuf {
    //     Path::new("/data/3D_basisvoorziening/32cz1_2020_volledig/32cz1_04_bench.city.json")
    //         .canonicalize()
    //         .expect("Could not find the INPUT file.")
    // }
    //
    // let cm1 = dereference::parse_dereferece(get_data());
    // let cm2 = dereference::parse_dereferece(get_data());
    // let cm3 = dereference::parse_dereferece(get_data());
    // let cm4 = dereference::parse_dereferece(get_data());
    // -----------------------
    let mut a: usize = 0;
    a += 2;
    println!("{}", a.to_string());
    a += 2;
    println!("{}", a.to_string());
}
