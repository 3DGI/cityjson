use serde::Deserialize;
use serde_json::{from_reader, from_str};
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

///```rust
///let cm = cjlib::CityModel::new();
///let cm2 = cjlib::CityModel::default();
/// ```
pub struct CityModel {
    type_cm: CityModelType,
    version: CityJSONVersion,
    transform: Option<Transform>,
}

impl CityModel {
    pub fn new() -> Self {
        Self {
            type_cm: CityModelType::CityJSON,
            version: CityJSONVersion::V1_1,
            transform: None,
        }
    }

    /// Parse a string of CityJSON text.
    pub fn from_str(cityjson: &str) -> Self {
        let icm: ICityModel = from_str(cityjson).expect("Could not deserialize into ICityModel.");
        Self {
            type_cm: icm.type_cm,
            version: icm.version,
            transform: Some(icm.transform),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let file = File::open(path.as_ref()).expect("Couldn't open CityJSON file");
        let reader = BufReader::new(&file);
        Self::from_reader(reader)
    }

    pub fn from_reader<R>(reader: R) -> Self
    where
        R: Read,
    {
        let icm: ICityModel = from_reader(reader).expect("Could not deserialize into ICityModel.");
        Self {
            type_cm: icm.type_cm,
            version: icm.version,
            transform: Some(icm.transform),
        }
    }

    pub fn type_cm(&self) -> &CityModelType {
        &self.type_cm
    }

    fn set_type(&mut self, type_cm: CityModelType) {
        self.type_cm = type_cm;
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
        self.transform = Some(transform.clone());
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
            .field("type_cm", &self.type_cm)
            .field("transform", &self.transform)
            .finish()
    }
}

impl fmt::Display for CityModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(\n\tversion: {}\n\ttype: {}\n)",
            &self.version, &self.type_cm,
        )
    }
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "version", try_from = "String")]
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

#[derive(Debug, Deserialize)]
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
#[derive(Debug, Clone, Copy, Deserialize)]
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

/// Indexed-structures.
/// Parsing a CityJSON document is (internally) a two-step process with cjlib.
/// In the first step, the document is deserialized into structures that are much alike the
/// CityJSON schema, and use indexed-vertices. These structures are private and they are
/// prefixed with an `I`, eg. `ICityModel`.
/// In the second step, the indexed-structures are transformed to a `CityModel`, through
/// dereferencing the vertices and other operations.

/// Indexed-CityModel, which is an intermediary struct that is directly deserialized from CityJSON
/// document and converted to a CityModel.
#[derive(Deserialize)]
struct ICityModel {
    #[serde(rename = "type")]
    type_cm: CityModelType,
    version: CityJSONVersion,
    transform: Transform,
    #[serde(skip)]
    cityobjects: ICityObjectsPlaceholder,
    vertices: IVertices,
}

type ICityObjectsPlaceholder = String;

/// Vertex coordinates, deserialized from a CityJSON document.
/// Uses i32, which is way too much, but i16 not enough, because a CityModel can easily have
/// transformed coordinates beyond +/-32767.
type IVertices = Vec<[i32; 3]>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_data_dir() -> PathBuf {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.join("resources").join("data")
    }

    #[test]
    fn instantiate_citymodel() {
        let _cm = CityModel::new();
        let _cm2 = CityModel::default();
    }

    #[test]
    fn citymodel_from_str() {
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
        let cm = CityModel::from_str(&cityjson_str);
        println!("{:?}", cm);
    }

    #[test]
    fn citymodel_from_file() {
        let pb: PathBuf = test_data_dir().join("minimal_valid.city.json");
        let _ = CityModel::from_file(&pb);
        let ps: String = pb.to_str().unwrap().to_string();
        let _ = CityModel::from_file(ps);
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
        let v = CityJSONVersion::try_from("1.1");
        assert_eq!(v.unwrap(), CityJSONVersion::V1_1);
        let v2 = CityJSONVersion::try_from("1.2");
        v2.expect_err("Unsupported CityJSON version.");
    }
}
