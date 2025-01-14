mod attributes;
pub mod errors;
mod extensions;
pub mod transform;

use errors::Result;
use serde_cityjson::v1_1;
use serde_cityjson::{from_str, CityJSON};
use std::fmt;
use std::io::BufRead;
use std::path::Path;

pub use extensions::{Extension, ExtensionName, Extensions};
pub use serde_cityjson::{CityJSONVersion, CityModelType};
pub use transform::Transform;

#[test]
fn citymodel_from_str_minimal() {
    let cityjson_str = r#"{
      "type": "CityJSON",
      "version": "1.1",
      "extensions": {},
      "transform": {
        "scale": [ 1.0, 1.0, 1.0 ],
        "translate": [ 0.0, 0.0, 0.0 ]
      },
      "metadata": {},
      "CityObjects": {},
      "vertices": [],
      "appearance": {},
      "geometry-templates": {
        "templates": [],
        "vertices-templates": []
      }
    }"#;
    let cm = CityModel::from_str(cityjson_str).unwrap();
    println!("{cm}");
}

pub struct CityModel {
    extensions: Option<Extensions>,
    id: Option<String>,
    transform: Option<Transform>,
    type_model: CityModelType,
    version: Option<CityJSONVersion>,
}

impl CityModel {
    pub fn new(type_model: CityModelType) -> Self {
        Self {
            extensions: None,
            id: None,
            transform: None,
            type_model,
            version: None,
        }
    }

    /// Deserialize a CityJSON object from a `&[u8]`.
    pub fn from_slice(_bytes: &[u8]) -> Result<Self> {
        todo!()
    }

    /// Deserialize a CityJSON object from a `&str`.
    pub fn from_str(s: &str) -> Result<Self> {
        let cityjson = from_str(s)?;
        let cm: v1_1::CityModel = match cityjson {
            CityJSON::V1_1(cm) => {
                // todo: v2_0::CityModel::from(cm)
                cm
            }
            CityJSON::V2_0(_cm) => {
                todo!()
            }
        };
        Ok(Self {
            extensions: cm.extensions.map(|e| Extensions::from(e)),
            id: cm.id.map(|cow| cow.into_owned()),
            transform: cm.transform.map(|t| Transform::from(t)),
            type_model: cm.type_cm,
            version: cm.version,
        })
    }

    /// Deserialize a CityJSON object or CityJSONFeatures from a file.
    /// If the file contains CityJSONFeatures, the first JSON object is expected to be a
    /// CityJSON object.
    pub fn from_file<P: AsRef<Path>>(_path: P) -> Result<Self> {
        todo!();
    }

    /// Create a CityModel from a stream of CityJSONFeatures, aggregating them into the CityModel's
    /// CityObjects. Assumes that the first item in the stream is a CityJSON object.
    pub fn from_stream<R>(_cursor: R) -> Result<Self>
    where
        R: BufRead,
    {
        todo!()
    }

    pub fn version(&self) -> &Option<CityJSONVersion> {
        &self.version
    }

    pub fn set_version(&mut self, version: CityJSONVersion) {
        self.version = Some(version);
    }

    pub fn transform(&self) -> Option<&Transform> {
        self.transform.as_ref()
    }

    pub fn set_transform(&mut self, transform: &Transform) {
        self.transform = Some(transform.clone());
    }

    pub fn extensions(&self) -> Option<&Extensions> {
        self.extensions.as_ref()
    }

    pub fn extensions_mut(&mut self) -> &mut Extensions {
        self.extensions.get_or_insert_with(Extensions::new)
    }
}

impl Default for CityModel {
    fn default() -> Self {
        Self {
            extensions: None,
            id: None,
            transform: None,
            type_model: CityModelType::default(),
            version: Some(CityJSONVersion::default()),
        }
    }
}

impl fmt::Debug for CityModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CityModel")
            .field("extensions", &self.extensions)
            .field("id", &self.id)
            .field("transform", &self.transform)
            .field("type_model", &self.type_model)
            .field("version", &self.version)
            .finish()
    }
}

impl fmt::Display for CityModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(\n\tversion: {}\n\tnr. cityobjects: \n\ttransform: {}\n)",
            format_version_option(&self.version),
            format_transform_option(&self.transform)
        )
    }
}

fn format_version_option(version: &Option<CityJSONVersion>) -> String {
    version
        .as_ref()
        .map(|v| v.to_string())
        .unwrap_or("None".to_string())
}

fn format_transform_option(transform: &Option<Transform>) -> String {
    transform
        .as_ref()
        .map(|t| t.to_string())
        .unwrap_or("None".to_string())
}
