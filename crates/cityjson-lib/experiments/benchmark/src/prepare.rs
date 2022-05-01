//! Crate for preparing a CityJSON file for the benchmark.
#![feature(path_file_prefix)]

use clap::{crate_version, App, Arg};
use serde::de::Unexpected::Option;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn change_file_name(path: &PathBuf) -> PathBuf {
    // Expecting .city.json extension
    let oldname = path.file_prefix().unwrap().to_owned();
    path.with_file_name(oldname.into_string().unwrap() + "_bench")
        .with_extension("city.json")
}

fn deserialize_cityjson(str_dataset: &String) -> serde_json::Value {
    let re: Result<Value, _> = serde_json::from_str(&str_dataset);
    if re.is_err() {
        println!("errors: {:?}", re.as_ref().err().unwrap());
    }
    re.unwrap()
}

/// Prepare a CityJSON file for benchmarking.
///
/// 1. un-transforming its vertices,
/// 2. removing its attributes,
/// 3. removing its metadata,
/// 4. removing all whitespace and newline characters,
/// 5. removing the geometries with the specified type.
///
/// The results are written to INPUT_bench.city.json
/// Run it as `$ prepare my_file.json`, or `$ prepare -g MultiSurface my_file.json` to remove the
/// MultiSurface geometries from the CityObjects.
fn main() -> std::io::Result<()> {
    // Copied from: https://github.com/cityjson/cjval
    let app = App::new("prepare")
        .about("Prepare a CityJSON file for benchmarking.")
        .version(crate_version!())
        .arg(
            Arg::with_name("INPUT")
                .required(true)
                .help("CityJSON file (<myfile>.city.json) to prepare for benchmarking. "),
        )
        .arg(
            Arg::with_name("GEOMETRY TYPE")
                .short("g")
                .long("geometry_type")
                .required(false)
                .help("The geometry type to delete from the CityObjects")
                .takes_value(true),
        );
    let matches = app.get_matches();

    let p1 = Path::new(matches.value_of("INPUT").unwrap())
        .canonicalize()
        .unwrap();
    let str_dataset = std::fs::read_to_string(&matches.value_of("INPUT").unwrap())
        .expect("Couldn't read CityJSON file");
    let mut cm = deserialize_cityjson(&str_dataset);

    // Transform the vertices
    let transform = cm["transform"].to_owned();
    let mut new_vertices: Vec<Value> = Vec::new();
    for v in cm["vertices"].as_array().unwrap() {
        let point: [f64; 3] = [
            &v[0].as_f64().unwrap() * transform["scale"][0].as_f64().unwrap()
                + transform["translate"][0].as_f64().unwrap(),
            &v[1].as_f64().unwrap() * transform["scale"][1].as_f64().unwrap()
                + transform["translate"][1].as_f64().unwrap(),
            &v[2].as_f64().unwrap() * transform["scale"][2].as_f64().unwrap()
                + transform["translate"][2].as_f64().unwrap(),
        ];
        new_vertices.push(Value::from(point.to_vec()));
    }
    cm["vertices"] = Value::from(new_vertices);

    let del_geomtype = matches.value_of("GEOMETRY TYPE");
    // Remove attributes
    let cos = cm["CityObjects"].as_object_mut().unwrap();
    for coval in cos.values_mut() {
        coval.as_object_mut().unwrap().remove("attributes");

        if let Some(gtype) = del_geomtype {
            println!("Deleting geometries with type {}", gtype);
            let mut to_del: Vec<usize> = Vec::new();
            for (gi, geom) in coval["geometry"].as_array().unwrap().iter().enumerate() {
                if geom["type"].as_str().unwrap() == gtype {
                    to_del.push(gi);
                }
            }
            for gi in to_del {
                coval["geometry"].as_array_mut().unwrap().remove(gi);
            }
        }
    }

    // Remove metadata
    cm.as_object_mut().unwrap().remove("metadata");
    cm.as_object_mut().unwrap().remove("+metadata-extended");

    // Write to file
    let str_dataset = serde_json::to_string(&cm).unwrap();
    let newfile = change_file_name(&p1);
    std::fs::write(newfile, str_dataset)?;
    Ok(())
}
