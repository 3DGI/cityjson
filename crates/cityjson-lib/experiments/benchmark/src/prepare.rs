#![feature(path_file_prefix)]

use clap::{crate_version, App, Arg};
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

fn main() -> std::io::Result<()> {
    // Copied from: https://github.com/cityjson/cjval
    let app = App::new("prepare")
        .about("Prepare a CityJSON file for benchmarking by:\n1) un-transforming its vertices,\n2) removing its attributes,\n3) removing its metadata,\n4) removing all whitespace and newline characters.\nThe results are written to INPUT_bench.city.json")
        .version(crate_version!())
        .arg(
            Arg::with_name("INPUT")
                .required(true)
                .help("CityJSON file (<myfile>.city.json) to prepare for benchmarking. "),
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

    // Remove attributes
    let cos = cm["CityObjects"].as_object_mut().unwrap();
    for coval in cos.values_mut() {
        coval.as_object_mut().unwrap().remove("attributes");
        let mut to_del: Vec<usize> = Vec::new();
        for (gi, geom) in coval["geometry"].as_array().unwrap().iter().enumerate() {
            if geom["type"].as_str().unwrap() == "MultiSurface" {
                to_del.push(gi);
            }
        }
        for gi in to_del {
            coval["geometry"].as_array_mut().unwrap().remove(gi);
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
