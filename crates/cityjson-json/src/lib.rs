#![cfg_attr(docsrs, feature(doc_cfg))]
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
//! ### Validation and deserialization of invalid CityJSON objects
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
//! # use serde_cityjson::{from_str, CityJSON};
//! # use std::fs::File;
//! # use std::io::Read;
//! # use std::path::PathBuf;
//! # fn main() -> Result<(), String> {
//! # let dummy_complete = PathBuf::from("tests").join("data").join("v1_1").join("cityjson_dummy_complete.city.json");
//! # let mut file = File::open(dummy_complete).map_err(|e| e.to_string())?;
//! # let mut cityjson_json = String::new();
//! # file.read_to_string(&mut cityjson_json).map_err(|e| e.to_string())?;
//!
//! let cj = from_str(&cityjson_json).map_err(|e| e.to_string())?;
//! match &cj {
//!     CityJSON::V1_1(cm) => {
//!         println!("CityJSON version 1.1 {:?}", &cm);
//!     }
//!     CityJSON::V2_0(cm) => {
//!         println!("CityJSON version 2.0 {:?}", &cm);
//!     }
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! We don't know the version of the input CityJSON and we silently ignore all unhandled versions.
//! ```rust
//! use serde_cityjson::{from_str, CityJSON};
//! use serde_cityjson::v1_1;
//! # use std::fs::File;
//! # use std::io::Read;
//! # use std::path::PathBuf;
//! # fn main() -> Result<(), String> {
//! # let dummy_complete = PathBuf::from("tests").join("data").join("v1_1").join("cityjson_dummy_complete.city.json");
//! # let mut file = File::open(dummy_complete).map_err(|e| e.to_string())?;
//! # let mut cityjson_json = String::new();
//! # file.read_to_string(&mut cityjson_json).map_err(|e| e.to_string())?;
//!
//! let mut cm = v1_1::CityModel::default();
//! let cj = from_str(&cityjson_json).map_err(|e| e.to_string())?;
//! if let CityJSON::V1_1(c) = cj {
//!     cm = c;
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
//! # let dummy_complete = PathBuf::from("tests").join("data").join("v1_1").join("cityjson_dummy_complete.city.json");
//! # let mut file = File::open(dummy_complete).map_err(|e| e.to_string())?;
//! # let mut cityjson_json = String::new();
//! # file.read_to_string(&mut cityjson_json).map_err(|e| e.to_string())?;
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
use crate::errors::Error;
#[cfg(feature = "datasize")]
use datasize::DataSize;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::Path;

pub mod attributes;
pub mod boundary;
#[cfg_attr(docsrs, doc(cfg(feature = "datasize")))]
#[cfg(feature = "datasize")]
pub mod datasize;
pub mod errors;
pub mod indices;
pub mod labels;
pub mod v1_1;
pub mod v2_0;

/// A register of what file extensions are supported.
/// It allows comparison for equality with an [`std::ffi::OsStr`](std::ffi::OsStr), which we get when working with
/// [`std::path::Path`](Path)s.
/// There are two concepts that are important in this implementation,
/// [associated constants](https://doc.rust-lang.org/reference/items/associated-items.html#associated-constants)
/// and the [non_exhaustive attribute](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute)
/// which indicates that this type may have more fields or variants added in the future.
///
///
/// Alternative implementations that I considered:
///
/// The array of strings. However, I need to distinguish between 'json' and 'jsonl' and probably
/// other extensions in the future too. Thus, a simple containment test is not enough.
/// ```
/// static SUPPORTED_FILE_EXTENSION: [&str; 3] = [ "json", "cityjson", "jsonl" ];
/// let extension_of_input_file = "json";
/// let does_contain = SUPPORTED_FILE_EXTENSION.contains(&extension_of_input_file);
/// ```
///
/// An enum. While it achieves the same purpose as the struct implementation, it is much more
/// verbose.
/// ```
/// use std::ffi::OsStr;
///
/// #[derive(Debug, Copy, Clone)]
/// enum SupportedFileExtension {
///     Json,
///     CityJson,
///     Jsonl,
/// }
///
/// impl From<&SupportedFileExtension> for &str {
///     fn from(value: &SupportedFileExtension) -> Self {
///         match value {
///             SupportedFileExtension::Json => "json",
///             SupportedFileExtension::CityJson => "cityjson",
///             SupportedFileExtension::Jsonl => "jsonl",
///         }
///     }
/// }
///
/// impl SupportedFileExtension {
///     fn print_all() -> String {
///         format!("{:?}, {:?}, {:?}", Self::Json, Self::CityJson, Self::Jsonl).to_lowercase()
///     }
/// }
///
/// impl PartialEq<&OsStr> for SupportedFileExtension {
///     fn eq(&self, other: &&OsStr) -> bool {
///         let a: &str = self.into();
///         *other == a
///     }
/// }
/// ```
#[non_exhaustive]
#[derive(Debug)]
pub struct SupportedFileExtension;

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
struct TypeAndVersion {
    #[serde(rename = "type")]
    type_model: CityModelType,
    version: Option<CityJSONVersion>,
}

/// CityModel type.
///
/// Marks if the [CityModel] represents a CityJSON object or a CityJSONFeature object.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub enum CityModelType {
    #[default]
    CityJSON,
    CityJSONFeature,
}

impl Display for CityModelType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            CityModelType::CityJSON => {
                write!(f, "CityJSON")
            }
            CityModelType::CityJSONFeature => {
                write!(f, "CityJSONFeature")
            }
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Hash, Deserialize, Serialize)]
#[serde(tag = "version", try_from = "String", into = "String")]
pub enum CityJSONVersion {
    V1_0,
    #[default]
    V1_1,
    V2_0,
}

impl CityJSONVersion {
    fn _from_str(value: &str) -> errors::Result<CityJSONVersion> {
        match value {
            "1.0" | "1.0.0" | "1.0.1" | "1.0.2" | "1.0.3" => Ok(CityJSONVersion::V1_0),
            "1.1" | "1.1.0" | "1.1.1" | "1.1.2" | "1.1.3" => Ok(CityJSONVersion::V1_1),
            "2.0" | "2.0.0" | "2.0.1" => Ok(CityJSONVersion::V2_0),
            _ => Err(Error::UnsupportedVersion(
                value.to_string(),
                "1.0, 1.0.0, 1.0.1, 1.0.2, 1.0.3, 1.1, 1.1.0, 1.1.1, 1.1.2, 1.1.3, 2.0, 2.0.0, 2.0.1".to_string(),
            )),
        }
    }
}

impl Display for CityJSONVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            CityJSONVersion::V1_0 => {
                write!(f, "1.0")
            }
            CityJSONVersion::V1_1 => {
                write!(f, "1.1")
            }
            CityJSONVersion::V2_0 => {
                write!(f, "2.0")
            }
        }
    }
}

impl TryFrom<&str> for CityJSONVersion {
    type Error = Error;

    fn try_from(value: &str) -> errors::Result<Self> {
        CityJSONVersion::_from_str(value)
    }
}

impl TryFrom<String> for CityJSONVersion {
    type Error = Error;

    fn try_from(value: String) -> errors::Result<Self> {
        CityJSONVersion::_from_str(value.as_ref())
    }
}

/// This implementation is only used for serializing the CityJSON version, because serde cannot
/// serialize from 'try_into' (which is provided by the 'try_from' implementations).
/// So we need this Into, even though [std says that one should avoid implementing Into](https://doc.rust-lang.org/std/convert/trait.Into.html).
#[allow(clippy::from_over_into)]
impl Into<String> for CityJSONVersion {
    fn into(self) -> String {
        match self {
            CityJSONVersion::V1_0 => String::from("1.0"),
            CityJSONVersion::V1_1 => String::from("1.1"),
            CityJSONVersion::V2_0 => String::from("2.0"),
        }
    }
}

#[derive(Debug)]
pub enum CityJSON<'cm> {
    V1_1(v1_1::CityModel<'cm>),
    V2_0(v2_0::CityModel<'cm>),
}

pub fn from_str(cj: &str) -> errors::Result<CityJSON> {
    let type_version: TypeAndVersion = serde_json::from_str(cj)?;
    match type_version.type_model {
        CityModelType::CityJSON => {
            if let Some(version) = type_version.version {
                match version {
                    CityJSONVersion::V1_0 => {
                        todo!()
                    }
                    CityJSONVersion::V1_1 => {
                        let cm = serde_json::from_str::<v1_1::CityModel>(cj)?;
                        Ok(CityJSON::V1_1(cm))
                    }
                    CityJSONVersion::V2_0 => {
                        todo!()
                    }
                }
            } else {
                Err(Error::MalformedCityJSON(
                    serde::de::Error::custom("CityJSON object must contain a version member"),
                    None,
                ))
            }
        }
        CityModelType::CityJSONFeature => feature_form_str(cj, &CityJSONVersion::V2_0),
    }
}

pub fn feature_form_str<'a>(cf: &'a str, version: &'a CityJSONVersion) -> errors::Result<CityJSON<'a>> {
    match version {
        CityJSONVersion::V2_0 => {
            if let Ok(cm) = serde_json::from_str::<v2_0::CityModel>(cf) {
                Ok(CityJSON::V2_0(cm))
            } else if let Ok(cm) = serde_json::from_str::<v1_1::CityModel>(cf) {
                Ok(CityJSON::V1_1(cm))
            } else {
                Err(Error::ExpectedCityJSONFeature("could not deserialize object as CityJSONFeature with CityJSON version 1.1 or 2.0".to_string()))
            }
        }
        CityJSONVersion::V1_1 => {
            if let Ok(cm) = serde_json::from_str::<v1_1::CityModel>(cf) {
                Ok(CityJSON::V1_1(cm))
            } else if let Ok(cm) = serde_json::from_str::<v2_0::CityModel>(cf) {
                Ok(CityJSON::V2_0(cm))
            } else {
                Err(Error::ExpectedCityJSONFeature("could not deserialize object as CityJSONFeature with CityJSON version 1.1 or 2.0".to_string()))
            }
        }
        CityJSONVersion::V1_0 => Err(Error::UnsupportedVersion(
            "1.1, 2.0".to_string(),
            "1.0".to_string(),
        )),
    }
}

impl From<v1_1::CityModel<'_>> for v2_0::CityModel<'_> {
    fn from(value: v1_1::CityModel) -> Self {
        todo!()
    }
}
