pub mod errors;
pub mod transform;

use errors::Result;
use serde_cityjson::v1_1;
use serde_cityjson::{from_str, CityJSON};
use std::collections::HashMap;
use std::fmt;
use std::io::BufRead;
use std::path::Path;

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

pub type ExtensionName = String;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct Extensions(HashMap<ExtensionName, Extension>);

// Implement conversion from serde_cityjson types
impl From<v1_1::Extensions> for Extensions {
    fn from(ext: v1_1::Extensions) -> Self {
        Self(
            ext.into_iter()
                .map(|(k, v)| (k, Extension::from(v)))
                .collect(),
        )
    }
}

impl Extensions {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert<N: Into<ExtensionName>>(&mut self, name: N, extension: Extension) {
        self.0.insert(name.into(), extension);
    }

    pub fn remove(&mut self, name: &str) -> Option<Extension> {
        self.0.remove(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&Extension> {
        self.0.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Extension> {
        self.0.get_mut(name)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ExtensionName, &Extension)> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&ExtensionName, &mut Extension)> {
        self.0.iter_mut()
    }
}

impl<'a> IntoIterator for &'a Extensions {
    type Item = (&'a ExtensionName, &'a Extension);
    type IntoIter = std::collections::hash_map::Iter<'a, ExtensionName, Extension>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Extensions {
    type Item = (&'a ExtensionName, &'a mut Extension);
    type IntoIter = std::collections::hash_map::IterMut<'a, ExtensionName, Extension>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

#[derive(Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Extension {
    url: String,
    version: String,
}

impl From<v1_1::Extension> for Extension {
    fn from(ext: v1_1::Extension) -> Self {
        Self {
            url: ext.url,
            version: ext.version,
        }
    }
}

impl Extension {
    pub fn new(url: String, version: String) -> Self {
        Self { url, version }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }

    pub fn set_version(&mut self, version: String) {
        self.version = version;
    }
}
