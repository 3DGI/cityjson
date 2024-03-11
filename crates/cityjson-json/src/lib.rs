//! CityJSON serialization library.\
//! `serde_cityjson` provides [serde-serializable](https://serde.rs/) Rust data structures for the
//! complete CityJSON specification, including a support for Extensions.
//!
//! The goals of `serde_cityjson` are,
//! 1. to provide serde-serializable data structures that follow the CityJSON specifications as
//!     closely as possible,
//! 2. to implement the complete CityJSON specifications, including *Extensions*,
//! 3. to support all major and minor (X.Y) versions, starting from `1.0`,
//! 4. to provide a stable API for packages that use `serde_cityjson`.
//!
//! `serde_cityjson` does not and will not provide functions for processing a city model, for instance
//! calculating the surface inclination.
//!
//! Supported CityJSON versions:
//! - [1.0](https://www.cityjson.org/specs/1.0.3/) in module
//! - [1.1](https://www.cityjson.org/specs/1.1.3/) in module [v1_1]
//! - [2.0](https://www.cityjson.org/specs/2.0.0/) in module [v2_0]
//!
//! ### Why not just use [`serde_json::Value`] and be done with it?
//!
//! Undoubtedly, the simplest method to deserialize a CityJSON object with serde is the same as
//! with any other JSON object, to deserialize into the generic [`serde_json::Value`]:
//!
//! ```
//! use serde_json::{Result, Value};
//!
//! fn main() -> Result<()> {
//!    let city_json = r#"{
//!      "type": "CityJSON",
//!      "version": "1.1",
//!      "extensions": {},
//!      "transform": {
//!        "scale": [1.0, 1.0, 1.0],
//!        "translate": [0.0, 0.0, 0.0]
//!      },
//!      "metadata": {},
//!      "CityObjects": {},
//!      "vertices": [],
//!      "appearance": {},
//!      "geometry-templates": {}
//!    }"#;
//!    let cm_value: Value = serde_json::from_str(city_json)?;
//!    println!("CityJSON version: {}", &cm_value["version"]);
//!    Ok(())
//! }
//! ```
//!
//! Using serde_cityjson enables you to write strongly-typed code with CityJSON objects instead of
//! using string keys to extract members of the CityJSON. The talk
//! ["Type-Driven API Design in Rust" by Will Crichton](https://youtu.be/bnnacleqg6k?feature=shared)
//! provides some inspiration for this concept.
//!
//! ### Validation and deserializtion of invalid CityJSON objects
//!
//! `serde_cityjson` does not validate the CityJSON objects in the typical sense, but it tries to
//! parse and deserialize them into the Rust structures that mimic the CityJSON specification.
//! This means that only valid CityJSON are deserialized successfully and invalid objects return an
//! error. This follows the idea behind the great post
//! [Parse, don’t validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/).
//! However, if you would only like to validate a CityJSON file, the
//! [cjval](https://crates.io/crates/cjval) tool is a better option.
//!
//! # Getting started
//!
//! Below is an overview on what can you expect from the library and how to get started. See the
//! documentation of a specific member for more detailed examples.
//!
//! We are going to read and build the following CityJSON in the examples below.
//! The CityJSON is completely random, thus it does not contain valid geometries and you cannot
//! visualise it in a viewer. However, the CityJSON is schema-valid and contains all parts of the
//! specification.
//!
//! ```json
//!     {
//!       "type": "CityJSON",
//!       "version": "1.1",
//!       "extensions": {
//!         "Noise": {
//!           "url" : "https://someurl.orgnoise.json",
//!           "version": "2.0"
//!         }
//!       },
//!       "transform": {
//!         "scale": [1.0, 1.0, 1.0],
//!         "translate": [0.0, 0.0, 0.0]
//!       },
//!       "metadata": {
//!         "geographicalExtent": [ 84710.1, 446846.0, -5.3, 84757.1, 446944.0, 40.9 ],
//!         "identifier": "eaeceeaa-3f66-429a-b81d-bbc6140b8c1c",
//!         "pointOfContact": {
//!           "contactName": "3D geoinformation group, Delft University of Technology",
//!           "emailAddress": "3dgeoinfo-bk@tudelft.nl"
//!         },
//!         "referenceSystem": "https://www.opengis.net/def/crs/EPSG/0/2355"
//!       },
//!       "CityObjects": {
//!         "id-1": {
//!           "type": "BuildingPart",
//!           "geographicalExtent": [ 84710.1, 446846.0, -5.3, 84757.1, 446944.0, 40.9 ],
//!           "attributes": {
//!             "measuredHeight": 22.3,
//!             "roofType": "gable",
//!             "owner": "Elvis Presley"
//!           },
//!           "children": ["id-2"],
//!           "parents": ["id-3"],
//!           "geometry": [
//!             {
//!               "type": "Solid",
//!               "lod": "2.1",
//!               "boundaries": [
//!                 [ [[0, 3, 2, 1]], [[4, 5, 6, 7]], [[0, 1, 5, 4]], [[1, 2, 6, 5]] ]
//!               ],
//!               "semantics": {
//!                 "surfaces": [
//!                   { "type": "RoofSurface" },
//!                   { "type": "+PatioDoor"}
//!                ],
//!                "values": [[0, 0, null, 1]]
//!               },
//!               "material": {
//!                "irradiation": { "values": [[0, 0, 1, null]] },
//!                "red": { "value": 3 }
//!               },
//!               "texture": {
//!                 "summer-textures": {
//!                   "values": [
//!                     [ [[0, 10, 23, 22, 21]], [[0, 1, 2, 6, 5]], [[null]], [[null]] ]
//!                   ]
//!                 }
//!               }
//!             },
//!             {
//!               "type": "GeometryInstance",
//!               "template": 0,
//!               "boundaries": [372],
//!               "transformationMatrix": [
//!                 2.0, 0.0, 0.0, 0.0,
//!                 0.0, 2.0, 0.0, 0.0,
//!                 0.0, 0.0, 2.0, 0.0,
//!                 0.0, 0.0, 0.0, 1.0
//!               ]
//!             }
//!           ]
//!          },
//!         "id-3": {
//!           "type": "+NoiseBuilding"
//!         }
//!       },
//!       "vertices": [
//!         [102, 103, 1],
//!         [11, 910, 43],
//!         [25, 744, 22],
//!         [23, 88, 5],
//!         [8523, 487, 22]
//!       ],
//!       "appearance": {
//!         "materials": [
//!           {
//!             "name": "irradiation",
//!             "ambientIntensity":  0.2000,
//!             "diffuseColor":  [0.9000, 0.1000, 0.7500],
//!             "emissiveColor": [0.9000, 0.1000, 0.7500],
//!             "specularColor": [0.9000, 0.1000, 0.7500],
//!             "shininess": 0.2,
//!             "transparency": 0.5,
//!             "isSmooth": false
//!           }
//!         ],
//!         "textures":[
//!           {
//!             "type": "PNG",
//!             "image": "http://!www.someurl.org/filename.jpg"
//!           }
//!         ],
//!         "vertices-texture": [
//!           [0.0, 0.5],
//!           [1.0, 0.0],
//!           [1.0, 1.0],
//!           [0.0, 1.0]
//!         ],
//!         "default-theme-texture": "summer-textures",
//!         "default-theme-material": "irradiation"
//!       },
//!       "geometry-templates": {
//!         "templates": [
//!           {
//!             "type": "MultiSurface",
//!             "lod": "2.1",
//!             "boundaries": [
//!                [[0, 3, 2, 1]], [[4, 5, 6, 7]], [[0, 1, 5, 4]]
//!             ]
//!           }
//!         ],
//!         "vertices-templates": [
//!           [0.0, 0.5, 0.0],
//!           [1.0, 1.0, 0.0],
//!           [0.0, 1.0, 0.0]
//!         ]
//!       }
//!     }
//! ```
//!
//! ## Deserialize
//! The main function for deserializing a CityJSON is [deserialize_cityjson]. It checks the version
//! of the CityJSON and forwards the deserializer to required version implementation. The returned
//! [CityJSON] enum wraps the deserialized CityModel.
//!
//! We don't know the version of the input CityJSON, and we handle each version.
//! ```rust
//! # use serde_cityjson::{deserialize_cityjson, CityJSON};
//! # use std::fs::File;
//! # use std::io::Read;
//! # use std::path::PathBuf;
//! # fn main() -> Result<(), String> {
//! # let dummy_complete = PathBuf::from("resources").join("data").join("dummy_complete_v11.city.json");
//! # let mut file = File::open(dummy_complete).map_err(|e| e.to_string())?;
//! # let mut cityjson_json = String::new();
//! # file.read_to_string(&mut cityjson_json);
//!
//! if let Ok(cj) = deserialize_cityjson(&cityjson_json) {
//!     match &cj {
//!         CityJSON::V1_1(cm) => {
//!             println!("CityJSON version 1.1 {:?}", &cm);
//!         }
//!         CityJSON::V2_0(cm) => {
//!             println!("CityJSON version 2.0 {:?}", &cm);
//!         }
//!     }
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! We don't know the version of the input CityJSON and we silently ignore all unhandled versions.
//! ```rust
//! use serde_cityjson::{deserialize_cityjson, CityJSON};
//! use serde_cityjson::v1_1;
//! # use std::fs::File;
//! # use std::io::Read;
//! # use std::path::PathBuf;
//! # fn main() -> Result<(), String> {
//! # let dummy_complete = PathBuf::from("resources").join("data").join("dummy_complete_v11.city.json");
//! # let mut file = File::open(dummy_complete).map_err(|e| e.to_string())?;
//! # let mut cityjson_json = String::new();
//! # file.read_to_string(&mut cityjson_json);
//!
//! let mut cm = v1_1::CityModel::default();
//! if let Ok(cj) = deserialize_cityjson(&cityjson_json) {
//!     if let CityJSON::V1_1(c) = cj {
//!         cm = c;
//!     }
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! We do know the version of the input CityJSON.
//! ```rust
//! use serde_cityjson::v1_1;
//! # use std::fs::File;
//! # use std::io::Read;
//! # use std::path::PathBuf;
//! # fn main() -> Result<(), String> {
//! # let dummy_complete = PathBuf::from("resources").join("data").join("dummy_complete_v11.city.json");
//! # let mut file = File::open(dummy_complete).map_err(|e| e.to_string())?;
//! # let mut cityjson_json = String::new();
//! # file.read_to_string(&mut cityjson_json);
//!
//! let cm_v11: v1_1::CityModel = serde_json::from_str(&cityjson_json).map_err(|e| e.to_string())?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Serialize
//!
//!
//!
pub mod v1_1;
pub mod v2_0;
mod errors;
mod datasize;


use std::fmt;
use std::fs::File;
use std::io::{BufReader, Seek};
use std::path::Path;
use serde::{Deserialize};

#[non_exhaustive]
#[derive(Debug)]
struct SupportedFileExtension;

impl SupportedFileExtension {
    pub const JSON: &'static str = "json";
    pub const CITYJSON: &'static str = "cityjson";
    pub const JSONL: &'static str = "jsonl";
}

impl fmt::Display for SupportedFileExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}, {:?}, {:?}",
            Self::JSON,
            Self::CITYJSON,
            Self::JSONL
        )
    }
}


#[derive(Deserialize)]
struct CityJSONVersionString {
    version: String,
}

#[derive(Debug)]
pub enum CityJSON {
    V1_1(v1_1::CityModel),
    V2_0(v2_0::CityModel),
}

pub fn deserialize_cityjson(cj: &str) -> errors::Result<CityJSON> {
    let cm: CityJSONVersionString = serde_json::from_str(cj)?;
    match cm.version.as_str() {
        "1.1" | "1.1.1" | "1.1.2" | "1.1.3" => {
            let cm = serde_json::from_str::<v1_1::CityModel>(cj)?;
            Ok(CityJSON::V1_1(cm))
        }
        "2.0" | "2.0.0" => {
            let cm = serde_json::from_str::<v2_0::CityModel>(cj)?;
            Ok(CityJSON::V2_0(cm))
        }
        _ => { Err(errors::Error::UnsupportedVersion(cm.version, "1.1, 1.1.1, 1.1.2, 1.1.3, 2.0, 2.0.0".to_string())) }
    }
}

pub fn deserialize_from_path<P: AsRef<Path>>(path: P) -> errors::Result<CityJSON> {
    let mut file = File::open(path.as_ref())?;
    let reader = BufReader::new(&file);
    let cm: CityJSONVersionString = serde_json::from_reader(reader)?;
    // Read the file again for the second pass over the data
    file.rewind()?;
    let reader = BufReader::new(&file);
    match cm.version.as_str() {
        "1.1" | "1.1.1" | "1.1.2" | "1.1.3" => {
            let cm: v1_1::CityModel = serde_json::from_reader(reader)?;
            Ok(CityJSON::V1_1(cm))
        }
        "2.0" | "2.0.0" => {
            let cm: v2_0::CityModel = serde_json::from_reader(reader)?;
            Ok(CityJSON::V2_0(cm))
        }
        _ => { Err(errors::Error::UnsupportedVersion(cm.version, "1.1, 1.1.1, 1.1.2, 1.1.3, 2.0, 2.0.0".to_string())) }
    }
}

pub fn serde_value<P: AsRef<Path>>(path: P) -> serde_json::Result<serde_json::Value> {
    let file = File::open(path.as_ref()).unwrap();
    let reader = BufReader::new(&file);
    let cm: serde_json::Value = serde_json::from_reader(reader)?;
    Ok(cm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() -> Result<(), String> {
        let cityjson_str = r#"{
            "type": "CityJSON",
            "version": "1.1",
            "transform": {
                "scale": [1.0, 1.0, 1.0],
                "translate": [0.0, 0.0, 0.0]
            },
            "CityObjects": {},
            "vertices": []
        }"#;

        // We don't know the version of the incoming CityJSON, and we handle each version.
        if let Ok(cj) = deserialize_cityjson(&cityjson_str) {
            match cj {
                CityJSON::V1_1(cm) => {
                    dbg!(cm);
                }
                CityJSON::V2_0(cm) => {
                    dbg!(cm);
                }
            }
        }
        // We don't know the version and we silently ignore all unhandled versions
        let mut cm = v1_1::CityModel::default();
        if let Ok(cj) = deserialize_cityjson(&cityjson_str) {
            if let CityJSON::V1_1(c) = cj {
                cm = c;
            }
        }
        dbg!(cm);
        // We do know the version
        let cm_v11: v1_1::CityModel = serde_json::from_str(&cityjson_str).map_err(|e| e.to_string())?;
        dbg!(cm_v11);

        let cityjson_str = r#"{
            "type": "CityJSON",
            "version": "2.0",
            "transform": {
                "scale": [1.0, 1.0, 1.0],
                "translate": [0.0, 0.0, 0.0]
            },
            "CityObjects": {},
            "vertices": []
        }"#;

        let cms = deserialize_cityjson(&cityjson_str).unwrap();
        dbg!(cms);
        Ok(())
    }
}
