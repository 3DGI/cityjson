mod attributes;
pub mod errors;
mod extensions;
mod metadata;
mod transform;

use errors::Result;
use serde_cityjson::v1_1;
use serde_cityjson::{from_str, CityJSON};
use std::fmt;
use std::io::BufRead;
use std::path::Path;

pub use attributes::Attributes;
pub use extensions::{Extension, ExtensionName, Extensions};
pub use metadata::{Contact, ContactRole, ContactType, Metadata};
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
    extra: Option<Attributes>,
    id: Option<String>,
    metadata: Option<Metadata>,
    transform: Option<Transform>,
    type_model: CityModelType,
    version: Option<CityJSONVersion>,
}

impl CityModel {
    pub fn new(type_model: CityModelType) -> Self {
        Self {
            extensions: None,
            extra: None,
            id: None,
            metadata: None,
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
            extra: cm.extra.map(|e| Attributes::try_from(e)).transpose()?,
            id: cm.id.map(|cow| cow.into_owned()),
            metadata: cm.metadata.map(|m| Metadata::try_from(m)).transpose()?,
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

    pub fn version_mut(&mut self) -> &mut CityJSONVersion {
        self.version.get_or_insert_with(CityJSONVersion::default)
    }

    pub fn transform(&self) -> Option<&Transform> {
        self.transform.as_ref()
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        self.transform.get_or_insert_with(Transform::default)
    }

    pub fn extensions(&self) -> Option<&Extensions> {
        self.extensions.as_ref()
    }

    pub fn extensions_mut(&mut self) -> &mut Extensions {
        self.extensions.get_or_insert_with(Extensions::new)
    }

    pub fn extra_root_properties(&self) -> Option<&Attributes> {
        self.extra.as_ref()
    }

    pub fn extra_root_properties_mut(&mut self) -> &mut Attributes {
        self.extra.get_or_insert_with(Attributes::default)
    }
}

impl Default for CityModel {
    fn default() -> Self {
        Self {
            extensions: None,
            id: None,
            metadata: None,
            transform: None,
            type_model: CityModelType::default(),
            version: Some(CityJSONVersion::default()),
            extra: None,
        }
    }
}

impl fmt::Debug for CityModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CityModel")
            .field("extensions", &self.extensions)
            .field("extra", &self.extra)
            .field("id", &self.id)
            .field("metadata", &self.metadata)
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
            concat!(
                "(\n",
                "\tversion: {}\n",
                "\tnr. cityobjects: \n",
                "\ttransform: {}\n",
                "\tmetadata: {}\n",
                "\textra_root_properties: {}\n",
                ")"
            ),
            format_option(&self.version),
            format_option(&self.transform),
            format_option(&self.metadata),
            format_option(&self.extra)
        )
    }
}

fn format_option<T: std::fmt::Display>(option: &Option<T>) -> String {
    option
        .as_ref()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "None".to_string())
}
