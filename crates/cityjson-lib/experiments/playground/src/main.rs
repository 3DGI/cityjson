#![allow(dead_code, unused_variables)]

// use std::borrow::Borrow;
// use std::collections::HashMap;
// use std::ffi::OsStr;
// use std::fmt::{Display, Formatter};
// use std::path::{Path, PathBuf};
//
// type Point = [f64; 3];
//
// type Vertices<'vertices> = Vec<Point>;
//
// type LineString<'vertices> = Vec<&'vertices Point>;
//
// struct CityObject<'vertices> {
//     geometry: LineString<'vertices>,
// }
//
// struct CityModel<'vertices> {
//     cityobjects: HashMap<String, CityObject<'vertices>>,
//     vertices: Vertices<'vertices>,
// }
//
// type OptionalContainer = Option<Vec<Option<Point>>>;
//
// #[derive(Copy, Clone)]
// enum SupportedExtensions {
//     Json,
//     CityJson,
//     Jsonl,
//     Unsupported,
// }
//
// // impl From<&SupportedExtensions> for &str {
// //     fn from(value: &SupportedExtensions) -> Self {
// //         match value {
// //             SupportedExtensions::Json => "json",
// //             SupportedExtensions::CityJson => "cityjson",
// //             SupportedExtensions::Jsonl => "jsonl",
// //             SupportedExtensions::Unsupported => "",
// //         }
// //     }
// // }
//
// // impl From<&str> for SupportedExtensions {
// //     fn from(value: &str) -> Self {
// //         match value {
// //             "json" => Self::Json,
// //             "cityjson" => Self::CityJson,
// //             "jsonl" => Self::Jsonl,
// //             _ => Self::Unsupported,
// //         }
// //     }
// // }
//
// // impl PartialEq<OsStr> for SupportedExtensions {
// //     fn eq(&self, other: &OsStr) -> bool {
// //         let a: &str = self.into();
// //         other == a
// //     }
// // }
//
// #[non_exhaustive]
// #[derive(Debug)]
// struct SupportedFileExtension;
// impl SupportedFileExtension {
//     pub const JSON: &'static str = "json";
//     pub const CITYJSON: &'static str = "cityjson";
//     pub const JSONL: &'static str = "jsonl";
// }
//
// impl Display for SupportedFileExtension {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "{:?}, {:?}, {:?}",
//             Self::JSON,
//             Self::CITYJSON,
//             Self::JSONL
//         )
//     }
// }
//
// fn main() {
//     // let mut cm = CityModel {
//     //     cityobjects: HashMap::new(),
//     //     vertices: Vertices::new(),
//     // };
//     // let v: Vertices = vec![[12.0, 12.0, 12.0], [12.0, 12.0, 12.0], [12.0, 12.0, 12.0]];
//     // let mut l: LineString = Vec::new();
//     // l.push(&v[0]);
//     // l.push(&v[1]);
//     // l.push(&v[2]);
//     //
//     // let co: CityObject = CityObject { geometry: l };
//     // let mut cos: HashMap<String, CityObject> = HashMap::new();
//     // cos.insert("id1".to_string(), co);
//     //
//     // for (coid, co) in cm.cityobjects {
//     //     let p1 = co.geometry[0];
//     //     println!("{}", p1[0] * 2.0);
//     // }
//     // ----------------------
//     // fn get_data() -> PathBuf {
//     //     Path::new("/data/3D_basisvoorziening/32cz1_2020_volledig/32cz1_04_bench.city.json")
//     //         .canonicalize()
//     //         .expect("Could not find the INPUT file.")
//     // }
//     //
//     // let cm1 = dereference::parse_dereferece(get_data());
//     // let cm2 = dereference::parse_dereferece(get_data());
//     // let cm3 = dereference::parse_dereferece(get_data());
//     // let cm4 = dereference::parse_dereferece(get_data());
//     // -----------------------
//     // let mut a: usize = 0;
//     // a += 2;
//     // println!("{}", a.to_string());
//     // a += 2;
//     // println!("{}", a.to_string());
//     // ----------------------------
//     // let mut oc: OptionalContainer = Some(Vec::new());
//     // if let Some(ref mut _oc) = oc {
//     //     _oc.push(Some([1.0, 1.0, 1.0]));
//     // }
//     // if let Some(ref mut _oc) = oc {
//     //     _oc.push(Some([2.0, 2.0, 2.0]));
//     // }
//     // if let Some(ref mut _oc) = oc {
//     //     _oc.push(None);
//     // }
//     // if let Some(ref _oc) = oc {
//     //     for v in _oc {
//     //         println!("{:#?}", v);
//     //     }
//     // }
//     // -------------------
//     // let a: SupportedExtensions = SupportedExtensions::Json;
//     // let b: &str = a.into();
//     // ----------------------
//     let path = Path::new("./foo/bar.json");
//     match path.extension() {
//         None => {}
//         Some(extension_os_str) => {
//             if extension_os_str == SupportedFileExtension::JSON
//                 || extension_os_str == SupportedFileExtension::CITYJSON
//             {
//                 println!("json")
//             } else if extension_os_str == SupportedFileExtension::JSONL {
//                 println!("jsonl")
//             }
//         }
//     }
//     println!("{}", SupportedFileExtension)
// }
//
//
// /// #[derive(Debug, Copy, Clone)]
// /// enum SupportedFileExtension {
// ///     Json,
// ///     CityJson,
// ///     Jsonl,
// /// }
// ///
// /// impl From<&SupportedFileExtension> for &str {
// ///     fn from(value: &SupportedFileExtension) -> Self {
// ///         match value {
// ///             SupportedFileExtension::Json => "json",
// ///             SupportedFileExtension::CityJson => "cityjson",
// ///             SupportedFileExtension::Jsonl => "jsonl",
// ///         }
// ///     }
// /// }
// ///
// /// impl SupportedFileExtension {
// ///     fn print_all() -> String {
// ///         format!("{:?}, {:?}, {:?}", Self::Json, Self::CityJson, Self::Jsonl).to_lowercase()
// ///     }
// /// }
// ///
// /// impl PartialEq<&OsStr> for SupportedFileExtension {
// ///     fn eq(&self, other: &&OsStr) -> bool {
// ///         let a: &str = self.into();
// ///         *other == a
// ///     }
// /// }

#[allow(dead_code)]
use serde::{de, Deserialize, Deserializer}; // 1.0.147
use serde_json; // 1.0.87
use std::io::{BufRead, Cursor};
use std::str::FromStr;

#[derive(Deserialize, Debug)]
struct Intermediary {
    id: String,
}

#[derive(Debug)]
struct Final {
    id: String,
}

impl FromStr for Final {
    type Err = serde_json::Error;

    /// Deserialize a string of CityJSON text.
    fn from_str(feature_str: &str) -> std::result::Result<Self, Self::Err> {
        let im: Intermediary = serde_json::from_str(feature_str)?;
        if im.id == "this_is_ok" {
            Ok(Final { id: im.id })
        } else {
            Err(de::Error::custom(format!("error: {:?}", im.id)))
        }
    }
}

#[derive(Debug)]
pub enum GeozeroError {
    Error,
}

impl std::fmt::Display for GeozeroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for GeozeroError {}

pub type Result<T> = std::result::Result<T, GeozeroError>;

#[derive(PartialEq, Debug)]
pub enum ColumnValue {
    Int(i32),
}

/// Get property value as Rust type.
pub trait PropertyReadType<T = Self>
where
    T: PropertyReadType,
{
    /// Get property value as Rust type.
    fn get_value(v: &ColumnValue) -> Result<T>;
}

impl From<&ColumnValue> for Result<i32> {
    fn from(v: &ColumnValue) -> Result<i32> {
        if let ColumnValue::Int(v) = v {
            Ok(*v)
        } else {
            Err(GeozeroError::Error)
        }
    }
}

impl PropertyReadType for i32 {
    fn get_value(v: &ColumnValue) -> Result<Self> {
        v.into()
    }
}

macro_rules! impl_scalar_property_reader {
    ( $t:ty, $e:path ) => {
        impl From<&ColumnValue> for Result<$t> {
            fn from(v: &ColumnValue) -> Result<$t> {
                if let $e(v) = v {
                    Ok(*v)
                } else {
                    Err(GeozeroError::Error)
                }
            }
        }
        impl PropertyReadType for $t {
            fn get_value(v: &ColumnValue) -> Result<$t> {
                v.into()
            }
        }
    };
}

fn test_grid(extent: &[i32; 6]) -> [i32; 2] {
    let dx = extent[3] - extent[0];
    [extent[0], extent[1]]
}

// Implement an iterator for a 3D Vector

type CellId = [usize; 2];
type Cell = Vec<Feature>;
type Feature = usize;

struct Grid {
    data: Vec<Vec<Cell>>,
}

impl Grid {
    fn leaves(&self) -> GridIterator<'_> {
        GridIterator {
            row_index: 0,
            col_index: 0,
            items: &self.data,
        }
    }
}

impl<'nestedvec> IntoIterator for &'nestedvec Grid {
    type Item = (CellId, &'nestedvec Cell);
    type IntoIter = GridIterator<'nestedvec>;

    fn into_iter(self) -> Self::IntoIter {
        GridIterator {
            row_index: 0,
            col_index: 0,
            items: &self.data,
        }
    }
}

struct GridIterator<'nestedvec> {
    row_index: usize,
    col_index: usize,
    items: &'nestedvec Vec<Vec<Cell>>,
}

impl<'nestedvec> Iterator for GridIterator<'nestedvec> {
    type Item = (CellId, &'nestedvec Cell);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(column) = self.items.get(self.col_index) {
            if let Some(cell) = column.get(self.row_index) {
                let item = Some(([self.row_index, self.col_index], cell));
                self.row_index += 1;
                item
            } else {
                // We are at the end of the current column, so jump to the next
                self.col_index += 1;
                self.row_index = 0;
                self.next()
            }
        } else {
            None
        }
    }
}

fn main() {
    // let feature_sequence = r#"{"id":"this_is_ok"}
    // {"id":"this_is_err"}
    // {"id":"this_is_ok"}"#;
    // let stream = Cursor::new(feature_sequence);
    // for s in stream.lines().flatten() {
    //     let res = Final::from_str(&s);
    //     match res {
    //         Ok(cf) => println!("{:?}", cf),
    //         Err(e) => println!("error: {:?}", e),
    //     }
    // }

    // let v = ColumnValue::Int(42);
    // let r = Result::<i32>::from(&v);
    // println!("{:?}", v);
    // let k = i32::get_value(&v);

    // assert_eq!(
    //     std::mem::size_of_val("b3bd7e17c-deb5-11e7-951f-610a7ca84980.city.jsonl"),
    //     48
    // );
    // assert_eq!(std::mem::size_of_val("/data/3DBAGv2/export/cityjson/v210908_fd2cee53/b3bd7e17c-deb5-11e7-951f-610a7ca84980.city.jsonl"), 95);
    //
    // let a = 100i64 as f64 / 70usize as f64;
    // println!("{}", a);
    //
    // let extent = [1, 2, 3, 4, 5, 6];
    // let a = test_grid(&extent);
    // println!("{}", extent[0]);

    // Iterate over a nested vector (3D) and generate the Cell-ids in the loop, using functional notation
    let data: Vec<Vec<Vec<usize>>> = vec![
        vec![vec![11, 12, 13], vec![14, 15, 16], vec![17, 18, 19]],
        vec![vec![21, 22, 23], vec![24, 25, 26], vec![27, 28, 29]],
        vec![vec![31, 32, 33], vec![34, 35, 36], vec![37, 38, 39]],
        vec![vec![41, 42, 43], vec![44, 45, 46], vec![47, 48, 49]],
    ];
    let a: Vec<_> = data.iter().enumerate().collect();
    println!("plain enumerate: {:?}", a);
    let b: Vec<_> = a
        .iter()
        .flat_map(|(c, v)| v.iter().enumerate().map(|(r, cell)| ([r, *c], cell)))
        .collect();
    println!("functional iteration: {:?}", b);

    // // Implemented an iterator returned from the .leaves() method
    // println!("iterator implementation .leaves():");
    let grid = Grid { data };
    for (cellid, cell) in grid.leaves() {
        println!("{:?}, {:?}", cellid, cell);
    }
    // Iterating with a for-loop
    println!("iterator implementation for-loop:");
    for (cellid, cell) in &grid {
        println!("{:?}, {:?}", cellid, cell);
    }
}
