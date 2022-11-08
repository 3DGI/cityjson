#![allow(dead_code, unused_variables)]

use std::borrow::Borrow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

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

type OptionalContainer = Option<Vec<Option<Point>>>;

#[derive(Copy, Clone)]
enum SupportedExtensions {
    Json,
    CityJson,
    Jsonl,
    Unsupported,
}

// impl From<&SupportedExtensions> for &str {
//     fn from(value: &SupportedExtensions) -> Self {
//         match value {
//             SupportedExtensions::Json => "json",
//             SupportedExtensions::CityJson => "cityjson",
//             SupportedExtensions::Jsonl => "jsonl",
//             SupportedExtensions::Unsupported => "",
//         }
//     }
// }

// impl From<&str> for SupportedExtensions {
//     fn from(value: &str) -> Self {
//         match value {
//             "json" => Self::Json,
//             "cityjson" => Self::CityJson,
//             "jsonl" => Self::Jsonl,
//             _ => Self::Unsupported,
//         }
//     }
// }

// impl PartialEq<OsStr> for SupportedExtensions {
//     fn eq(&self, other: &OsStr) -> bool {
//         let a: &str = self.into();
//         other == a
//     }
// }

#[non_exhaustive]
#[derive(Debug)]
struct SupportedFileExtension;
impl SupportedFileExtension {
    pub const JSON: &'static str = "json";
    pub const CITYJSON: &'static str = "cityjson";
    pub const JSONL: &'static str = "jsonl";
}

impl Display for SupportedFileExtension {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}, {:?}, {:?}",
            Self::JSON,
            Self::CITYJSON,
            Self::JSONL
        )
    }
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
    // let mut a: usize = 0;
    // a += 2;
    // println!("{}", a.to_string());
    // a += 2;
    // println!("{}", a.to_string());
    // ----------------------------
    // let mut oc: OptionalContainer = Some(Vec::new());
    // if let Some(ref mut _oc) = oc {
    //     _oc.push(Some([1.0, 1.0, 1.0]));
    // }
    // if let Some(ref mut _oc) = oc {
    //     _oc.push(Some([2.0, 2.0, 2.0]));
    // }
    // if let Some(ref mut _oc) = oc {
    //     _oc.push(None);
    // }
    // if let Some(ref _oc) = oc {
    //     for v in _oc {
    //         println!("{:#?}", v);
    //     }
    // }
    // -------------------
    // let a: SupportedExtensions = SupportedExtensions::Json;
    // let b: &str = a.into();
    // ----------------------
    let path = Path::new("./foo/bar.json");
    match path.extension() {
        None => {}
        Some(extension_os_str) => {
            if extension_os_str == SupportedFileExtension::JSON
                || extension_os_str == SupportedFileExtension::CITYJSON
            {
                println!("json")
            } else if extension_os_str == SupportedFileExtension::JSONL {
                println!("jsonl")
            }
        }
    }
    println!("{}", SupportedFileExtension)
}


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