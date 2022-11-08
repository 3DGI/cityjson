//! A software library for working with semantic 3D city models, based on the
//! [CityJSON](https://cityjson.org) data model.

mod errors;

use crate::errors::{Error, Result};

use serde::{Deserialize, Serialize};
use serde_json::{from_reader, from_str, to_string};
use std::collections::{hash_map, HashMap};
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader, LineWriter, Read};
use std::path::Path;
use std::str::FromStr;

///```rust
///let cm = cjlib::CityModel::new();
///let cm2 = cjlib::CityModel::default();
/// ```
pub struct CityModel {
    version: CityJSONVersion,
    transform: Option<Transform>,
    cityobjects: CityObjects,
}

impl CityModel {
    pub fn new() -> Self {
        Self {
            version: CityJSONVersion::V1_1,
            transform: None,
            cityobjects: Default::default(),
        }
    }

    /// Read from a file, either a regular JSON file or [JSON Lines text](https://jsonlines.org/).
    /// The parsing strategy is based on the file extension. Uses [`io::BufReader`](std::io::BufReader)
    /// for reading.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(&file);
        return match path.as_ref().extension() {
            None => Err(Error::InvalidExtension(path.as_ref().to_path_buf())),
            Some(extension_os_str) => {
                // todo: Make sure that when reading jsonl file, the reader reads a whole line at
                //  once (until the \n). Otherwise use the default bufreader.
                if extension_os_str == SupportedFileExtension::JSON
                    || extension_os_str == SupportedFileExtension::CITYJSON
                {
                    Self::from_reader(reader)
                } else if extension_os_str == SupportedFileExtension::JSONL {
                    Self::from_stream(reader)
                } else {
                    // let's try parsing as a regular CityJSON file
                    Self::from_reader(reader)
                }
            }
        };
    }

    /// Create a CityModel from a stream of CityJSONFeatures, aggregating them into the CityModel's
    /// CityObjects. Assumes that the first item in the stream is a CityJSON.
    pub fn from_stream<R>(cursor: R) -> Result<Self>
    where
        R: BufRead,
    {
        let mut stream_iter = cursor.lines();

        let mut cm: CityModel;
        // Do return an error if we cannot process the first item of the stream into a CityModel,
        // because the subsequent steps depend on it.
        if let Some(res) = stream_iter.next() {
            let cityjson_str = res?;
            cm = CityModel::from_str(&cityjson_str)?;
        } else {
            return Err(Error::StreamingError(String::from("empty stream")));
        }

        for res in stream_iter {
            // Don't break if for some reason we cannot process one feature from the stream,
            // but do notify about it.
            if let Ok(cityjsonfeature_str) = res {
                let cf = CityFeature::from_str(&cityjsonfeature_str)?;
                cm.cityobjects.insert(cf.id, CityObject);
            } else {
                // todo: log error
            }
        }
        Ok(cm)
    }

    /// Create a CityModel from an IO stream of CityJSON, by calling [`serde_json::from_reader`](serde_json::from_reader).
    pub fn from_reader<R>(reader: R) -> Result<Self>
    where
        R: Read,
    {
        let icm: ICityModel = from_reader(reader)?;
        Ok(Self {
            version: icm.version.unwrap(),
            transform: icm.transform,
            cityobjects: Default::default(),
        })
    }

    /// Convert the CityModel to a CityJSON document string.
    pub fn to_string(&self) -> Result<String> {
        Ok(to_string(&ICityModel::from(self))?)
    }

    /// Write the CityModel to a CityJSON file.
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file_out = File::create(path.as_ref())?;
        return match path.as_ref().extension() {
            None => Err(Error::InvalidExtension(path.as_ref().to_path_buf())),
            Some(extension_os_str) => {
                if extension_os_str == SupportedFileExtension::JSON
                    || extension_os_str == SupportedFileExtension::CITYJSON
                {
                    Ok(serde_json::to_writer(&file_out, &ICityModel::from(self))?)
                } else if extension_os_str == SupportedFileExtension::JSONL {
                    let cityjson = self.to_features_cityjson().unwrap();
                    let cityjsonfeatures = self
                        .to_features()
                        .flat_map(|cityfeature| cityfeature.to_string());
                    let mut file = LineWriter::new(file_out);
                    // The file must contain at least the first CityJSON object.
                    writeln!(file, "{}", cityjson)?;
                    for cf in cityjsonfeatures {
                        if writeln!(file, "{}", cf).is_err() {
                            todo!() // log error message here and skip the feature
                        }
                    }
                    Ok(())
                } else {
                    Err(Error::UnsupportedExtension)
                }
            }
        };
    }

    /// Convert the CityModel to a CityJSON object string, for passing as the first item in a
    /// CityJSONFeature stream. The new CityJSON object has empty "CityObjects" and "vertices"
    /// members, because these are supposed to be passed in subsequent CityJSONFeatures.
    pub fn to_features_cityjson(&self) -> Result<String> {
        Ok(to_string(&ICityModel {
            id: None,
            type_cm: CityModelType::CityJSON,
            version: Some(self.version),
            transform: Some(self.transform.unwrap_or_default()),
            cityobjects: ICityObjects::new(),
            vertices: IVertices::new(),
        })?)
    }

    pub fn to_features(&self) -> CityFeatureIterator {
        CityFeatureIterator {
            cityobjects_iter: self.cityobjects.iter(),
        }
    }

    pub fn version(&self) -> &CityJSONVersion {
        &self.version
    }

    fn set_version(&mut self, version: CityJSONVersion) {
        self.version = version;
    }

    pub fn transform(&self) -> Option<&Transform> {
        self.transform.as_ref()
    }

    pub fn set_transform(&mut self, transform: &Transform) {
        self.transform = Some(*transform);
    }
}

impl Default for CityModel {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for CityModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CityModel")
            .field("version", &self.version)
            .field("transform", &self.transform)
            .field("cityobjects", &self.cityobjects)
            .finish()
    }
}

impl fmt::Display for CityModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(\n\tversion: {}\n\tnr. cityobjects: {})",
            &self.version,
            &self.cityobjects.len()
        )
    }
}

impl FromStr for CityModel {
    type Err = Error;

    /// Parse a CityJSON document from a string, by calling [`serde_json::from_str`](serde_json::from_str).
    fn from_str(cityjson: &str) -> Result<Self> {
        let icm: ICityModel = from_str(cityjson)?;
        match icm.type_cm {
            CityModelType::CityJSON => Ok(Self {
                version: icm.version.unwrap(),
                transform: icm.transform,
                cityobjects: Default::default(),
            }),
            CityModelType::CityJSONFeature => Err(Error::ExpectedCityJSON(icm.type_cm)),
        }
    }
}

pub struct CityFeatureIterator<'cityobjects> {
    // We borrow the CityObjects from the CityModel for this struct, because the CityObjects values
    // are cloned into the CityFeature-s.
    cityobjects_iter: hash_map::Iter<'cityobjects, String, CityObject>,
}

impl<'cityobjects> Iterator for CityFeatureIterator<'cityobjects> {
    type Item = CityFeature;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((coid, _)) = self.cityobjects_iter.next() {
            Some(CityFeature::new(coid.to_string()))
        } else {
            None
        }
    }
}

pub struct CityFeature {
    id: String,
}

impl CityFeature {
    pub fn new(id: String) -> Self {
        Self { id }
    }

    /// Convert a CityFeature to a CityJSONFeature object string.
    pub fn to_string(&self) -> Result<String> {
        Ok(to_string(&ICityModel::from(self))?)
    }
}

impl Default for CityFeature {
    fn default() -> Self {
        Self::new(String::from(""))
    }
}

impl fmt::Debug for CityFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CityFeature").field("id", &self.id).finish()
    }
}

impl fmt::Display for CityFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "City(\n\tid: {}\n)", &self.id)
    }
}

impl FromStr for CityFeature {
    type Err = Error;

    /// Deserialize a string of CityJSON text.
    fn from_str(cityjson: &str) -> Result<Self> {
        let icm: ICityModel = from_str(cityjson)?;
        match icm.type_cm {
            CityModelType::CityJSON => Err(Error::ExpectedCityJSONFeature(icm.type_cm)),
            CityModelType::CityJSONFeature => Ok(Self {
                id: icm.id.unwrap(),
            }),
        }
    }
}

/// A register of what file extensions are supported.
/// It allows comparison for equality with an [`std::ffi::OsStr`](std::ffi::OsStr), which we get when working with
/// [`std::path::Path`](std::path::Path)s.
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

#[derive(Debug, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
#[serde(tag = "version", try_from = "String", into = "String")]
pub enum CityJSONVersion {
    V1_1,
}

impl fmt::Display for CityJSONVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CityJSONVersion::V1_1 => {
                write!(f, "1.1")
            }
        }
    }
}

impl TryFrom<&str> for CityJSONVersion {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        CityJSONVersion::_from_str(value)
    }
}

impl TryFrom<String> for CityJSONVersion {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        CityJSONVersion::_from_str(value.as_ref())
    }
}

impl CityJSONVersion {
    fn _from_str(value: &str) -> Result<CityJSONVersion> {
        match value {
            "1.1" | "1.1.1" | "1.1.2" | "1.1.3" => Ok(CityJSONVersion::V1_1),
            _ => Err(Error::UnsupportedVersion(
                value.to_string(),
                "1.1, 1.1.1, 1.1.2, 1.1.3".to_string(),
            )),
        }
    }
}

/// This implementation is only used for serializing the CityJSON version, because serde cannot
/// serialize from 'try_into' (which is provided by the 'try_from' implementations).
/// So we need this Into, even though [std says that one should avoid implementing Into](https://doc.rust-lang.org/std/convert/trait.Into.html).
impl Into<String> for CityJSONVersion {
    fn into(self) -> String {
        match self {
            CityJSONVersion::V1_1 => String::from("1.1"),
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum CityModelType {
    CityJSON,
    CityJSONFeature,
}

impl fmt::Display for CityModelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

// todo: "Transform members need to be private, so that we can change the internals if needed, eg f32 -> f64 or so"
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Transform {
    pub scale: [f32; 3],
    pub translate: [f32; 3],
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(scale: [{}, {}, {}], translate:[{}, {}, {}])",
            self.scale[0],
            self.scale[1],
            self.scale[2],
            self.translate[0],
            self.translate[1],
            self.translate[2]
        )
    }
}

impl Transform {
    pub fn new() -> Self {
        // NOTE: consider ::new(scale: &[f32;3], translate: &[f32;3])
        // NOTE: maybe use scale: [0.001, 0.001, 0.001] as default
        Self {
            scale: [1.0, 1.0, 1.0],
            translate: [0.0, 0.0, 0.0],
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

// NOTE: Not sure HashMap is the best choice in terms of performance.
// I've seen BTreeMap used in many places.
// Would there be any advantage of a custom type for the CityObject Id, instead of String?
type CityObjects = HashMap<String, CityObject>;

// NOTE: I think a CityObject should know its own Id. That would make it much simpler to send
// around CityObjects, eg. when converting to CityFeatures.
#[derive(Default, Debug)]
struct CityObject;

/// Indexed-structures.
/// Parsing a CityJSON document is (internally) a two-step process with cjlib.
/// In the first step, the document is deserialized into structures that are much alike the
/// CityJSON schema, and use indexed-vertices. These structures are private and they are
/// prefixed with an `I`, eg. `ICityModel`.
/// In the second step, the indexed-structures are transformed to a `CityModel`, through
/// de-referencing the vertices and other operations.

/// Indexed-CityModel, which is an intermediary struct that is directly deserialized from CityJSON
/// document and converted to a CityModel.
#[derive(Deserialize, Serialize)]
struct ICityModel {
    // NOTE: consider https://docs.rs/serde_with/latest/serde_with/index.html#skip_serializing_none
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(rename = "type")]
    type_cm: CityModelType,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<CityJSONVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transform: Option<Transform>,
    #[serde(skip_deserializing, rename = "CityObjects")]
    cityobjects: ICityObjects,
    vertices: IVertices,
}

impl From<&CityModel> for ICityModel {
    fn from(cm: &CityModel) -> Self {
        Self {
            id: None,
            type_cm: CityModelType::CityJSON,
            version: Some(cm.version),
            transform: Some(cm.transform.unwrap_or_default()),
            cityobjects: ICityObjects::new(),
            vertices: IVertices::new(),
        }
    }
}

impl From<&CityFeature> for ICityModel {
    fn from(cf: &CityFeature) -> Self {
        Self {
            id: Some(cf.id.clone()),
            type_cm: CityModelType::CityJSONFeature,
            version: None,
            transform: None,
            cityobjects: ICityObjects::new(),
            vertices: IVertices::new(),
        }
    }
}

type ICityObjects = HashMap<String, ICityObject>;

#[derive(Serialize)]
struct ICityObject;

/// Vertex coordinates, deserialized from a CityJSON document.
/// Uses i32, which is way too much, but i16 not enough, because a CityModel can easily have
/// transformed coordinates beyond +/-32767.
type IVertices = Vec<[i32; 3]>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::path::PathBuf;

    fn test_data_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("data")
    }

    fn test_output_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("output")
    }

    #[test]
    fn instantiate_citymodel() {
        let _cm = CityModel::new();
        let _cm2 = CityModel::default();
    }

    #[test]
    fn citymodel_from_str_minimal() {
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
        let cm = CityModel::from_str(cityjson_str).unwrap();
        println!("{:?}", cm);
    }

    #[test]
    fn citymodel_from_file() {
        let pb: PathBuf = test_data_dir().join("minimal_valid.city.json");
        let _ = CityModel::from_file(&pb);
        let ps: &str = pb.to_str().unwrap();
        let _ = CityModel::from_file(ps);
    }

    #[test]
    fn citymodel_to_string() {
        let tr = Transform {
            scale: [0.001, 0.001, 0.001],
            translate: [0.0, 0.0, 0.0],
        };
        let mut cm = CityModel::new();
        cm.set_transform(&tr);
        let cj = cm.to_string().unwrap();
        println!("CityJSON: {}", cj);
        println!("CityModel: {}", cm);
    }

    #[test]
    fn citymodel_to_file() {
        let pb: PathBuf = test_output_dir().join(".test_out.city.json");
        let _ = CityModel::new().to_file(pb);
    }

    #[test]
    fn debug_citymodel() {
        let cm = CityModel::new();
        println!("{:?}", cm);
    }

    #[test]
    fn display_citymodel() {
        let cm = CityModel::new();
        println!("{}", cm);
    }

    #[test]
    fn set_get_transform() {
        let mut cm = CityModel::default();
        println!("{:?}", cm);
        let t = Transform {
            scale: [1.0, 1.0, 1.0],
            translate: [0.0, 0.0, 0.0],
        };
        cm.set_transform(&t);
        println!("{:?}", cm);
    }

    #[test]
    fn cityjsonversion() {
        let vr = CityJSONVersion::try_from("1.2");
        assert_eq!(vr.unwrap(), CityJSONVersion::V1_1);
        let s: String = CityJSONVersion::V1_1.into();
        println!("CityJSONVersion.into(): {}", s);
        println!(
            "CityJSONVersion.to_string(): {}",
            CityJSONVersion::V1_1.to_string()
        );
        let v2 = CityJSONVersion::try_from("1.0");
        v2.expect_err("Unsupported CityJSON version.");
    }

    /// Can we deserialize a CityJSONFeature into an ICityModel?
    #[test]
    fn cityjsonfeature() {
        let cityjsonfeature_str = r#"{
            "type": "CityJSONFeature",
            "id": "id-1",
            "CityObjects": {},
            "vertices": []
        }"#;
        let _: ICityModel = from_str(cityjsonfeature_str).unwrap();
    }

    #[test]
    fn features_from_stream() {
        let feature_sequence = r#"{"type":"CityJSON","version":"1.1","transform":{"scale":[0.1,0.1,0.1],"translate":[0.0,0.0,0.0]},"CityObjects":{},"vertices":[]}
            {"type":"CityJSONFeature","id":"id-1","CityObjects":{},"vertices":[]}
            {"type":"CityJSONFeature","id":"id-2","CityObjects":{},"vertices":[]}"#;
        let stream = Cursor::new(feature_sequence);
        let cm = CityModel::from_stream(stream);
        println!("From stream: {:?}", cm);
    }

    #[test]
    fn features_from_file() {
        let pb: PathBuf = test_data_dir().join("minimal_valid.city.jsonl");
        let cm = CityModel::from_file(&pb);
        println!("From jsonl: {:?}", cm);
    }

    #[test]
    fn citymodel_to_features_iter() {
        let mut cityobjects = CityObjects::new();
        cityobjects.insert("id-1".to_string(), CityObject);
        cityobjects.insert("id-2".to_string(), CityObject);
        cityobjects.insert("id-3".to_string(), CityObject);
        let cm = CityModel {
            version: CityJSONVersion::V1_1,
            transform: None,
            cityobjects,
        };
        let cityfeature_iter: CityFeatureIterator = cm.to_features();
        for cf in cityfeature_iter {
            println!("{:?}", cf)
        }
        // The CityModel should still own its CityObject-s
        assert_eq!(cm.cityobjects.len(), 3);
        println!("{:?}", cm.cityobjects["id-1"]);

        let cityfeature_iter: CityFeatureIterator = cm.to_features();
        let cityjsonfeature_iter = cityfeature_iter.map(|cityfeature| cityfeature.to_string());
        for cityjsonfeature in cityjsonfeature_iter.flatten() {
            println!("{}", cityjsonfeature);
        }
    }

    #[test]
    fn features_to_file() {
        let mut cityobjects = CityObjects::new();
        cityobjects.insert("id-1".to_string(), CityObject);
        cityobjects.insert("id-2".to_string(), CityObject);
        cityobjects.insert("id-3".to_string(), CityObject);
        let cm = CityModel {
            version: CityJSONVersion::V1_1,
            transform: None,
            cityobjects,
        };

        let pb: PathBuf = test_output_dir().join(".test_out.city.jsonl");
        let _ = cm.to_file(pb);
    }
}
