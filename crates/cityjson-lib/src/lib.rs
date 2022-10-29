use serde::{Deserialize, Serialize};
use serde_json::{error, from_reader, from_str, to_string};
use std::collections::{hash_map, HashMap};
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

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

    /// Parse a CityJSON document from a string.
    pub fn from_str(cityjson: &str) -> Self {
        let icm: ICityModel = from_str(cityjson).expect("Could not deserialize into ICityModel.");
        match icm.type_cm {
            CityModelType::CityJSON => Self {
                version: icm.version.unwrap(),
                transform: icm.transform,
                cityobjects: Default::default(),
            },
            CityModelType::CityJSONFeature => {
                todo!() // add error Not a CityJSON
            }
        }
    }

    /// Read from a file, either a regular JSON file or [JSON Lines text](https://jsonlines.org/).
    /// The parsing strategy is based on the file extension.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let file = File::open(path.as_ref()).expect("Couldn't open CityJSON file");
        let reader = BufReader::new(&file);
        if let Some(extension) = path.as_ref().extension() {
            match extension.to_str().unwrap() {
                "json" | "cityjson" => Self::from_reader(reader),
                "jsonl" => Self::from_stream(reader),
                _ => {
                    // let's try parsing as a regular CityJSON file
                    Self::from_reader(reader)
                }
            }
        } else {
            todo!() // error here
        }
    }

    /// Create a CityModel from a stream of CityJSONFeatures, aggregating them into the CityModel's
    /// CityObjects. Assumes that the first item in the stream is a CityJSON.
    pub fn from_stream<R>(cursor: R) -> Self
    where
        R: BufRead,
    {
        let mut stream_iter = cursor.lines();

        let mut cm: CityModel;
        if let Some(res) = stream_iter.next() {
            let cityjson_str = res.expect("Failed to read item from the stream.");
            cm = CityModel::from_str(&cityjson_str);
        } else {
            todo!() // return an error from here
        }

        for res in stream_iter {
            let cityjsonfeature_str = res.expect("Failed to read item from the stream.");
            let cf = CityFeature::from_str(&cityjsonfeature_str);
            cm.cityobjects.insert(cf.id, CityObject);
        }
        cm
    }

    pub fn from_reader<R>(reader: R) -> Self
    where
        R: Read,
    {
        let icm: ICityModel = from_reader(reader).expect("Could not deserialize into ICityModel.");
        Self {
            version: icm.version.unwrap(),
            transform: icm.transform,
            cityobjects: Default::default(),
        }
    }

    /// Convert a CityModel to a CityJSON document string.
    pub fn to_string(&self) -> error::Result<String> {
        to_string(&ICityModel::from(self))
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> error::Result<()> {
        let ps = path.as_ref().to_str().expect("Invalid path.");
        let file_out = File::create(path.as_ref())
            .unwrap_or_else(|_| panic!("Could not open the file {} for writing.", ps));
        if let Some(extension) = path.as_ref().extension() {
            match extension.to_str().unwrap() {
                "json" | "cityjson" => serde_json::to_writer(&file_out, &ICityModel::from(self)),
                "jsonl" => todo!(),
                _ => {
                    todo!() // error with unknown extension
                }
            }
        } else {
            todo!() // error here
        }
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

    /// Parse a string of CityJSON text.
    pub fn from_str(cityjson: &str) -> Self {
        let icm: ICityModel = from_str(cityjson).expect("Could not deserialize into ICityModel.");
        match icm.type_cm {
            CityModelType::CityJSON => {
                todo!() // need error Not CityJSONFeature
            }
            CityModelType::CityJSONFeature => Self {
                id: icm.id.unwrap(),
            },
        }
    }

    /// Convert a CityFeature to a CityJSONFeature object string.
    pub fn to_string(&self) -> error::Result<String> {
        to_string(&ICityModel::from(self))
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
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "1.1" | "1.1.1" | "1.1.2" | "1.1.3" => Ok(CityJSONVersion::V1_1),
            _ => Err("Unsupported CityJSON version. Versions supported: 1.1, 1.1.1, 1.1.2, 1.1.3"),
        }
    }
}

impl TryFrom<String> for CityJSONVersion {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "1.1" | "1.1.1" | "1.1.2" | "1.1.3" => Ok(CityJSONVersion::V1_1),
            _ => Err("Unsupported CityJSON version. Versions supported: 1.1, 1.1.1, 1.1.2, 1.1.3"),
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
    #[serde(skip_deserializing)]
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
        let cm = CityModel::from_str(cityjson_str);
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
        let vr = CityJSONVersion::try_from("1.1");
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
}
