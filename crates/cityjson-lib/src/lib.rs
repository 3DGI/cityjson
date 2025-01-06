pub mod errors;

use errors::{Error, Result};
use serde_cityjson::v1_1;
use serde_cityjson::{from_str, CityJSON};
use std::fmt;
use std::io::BufRead;
use std::path::Path;

pub struct CityModel {
    id: Option<String>,
    type_model: CityModelType,
    version: Option<CityJSONVersion>,
    transform: Option<Transform>,
    extensions: Option<Extensions>,
}

impl CityModel {
    pub fn new() -> Self {
        Self {
            id: None,
            type_model: CityModelType(serde_cityjson::CityModelType::default()),
            version: None,
            transform: None,
            extensions: None,
        }
    }

    /// Deserialize a CityJSON object from a `&[u8]`.
    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
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
            CityJSON::V2_0(cm) => {
                todo!()
            }
        };
        Ok(Self {
            id: cm.id.map(|cow| cow.into_owned()),
            type_model: CityModelType(cm.type_cm),
            version: cm.version.map(|v| CityJSONVersion(v)),
            transform: cm.transform.map(|t| Transform(t)),
            extensions: cm.extensions.map(|e| Extensions(e)),
        })
    }

    /// Deserialize a CityJSON object or CityJSONFeatures from a file.
    /// If the file contains CityJSONFeatures, the first JSON object is expected to be a
    /// CityJSON object.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        todo!();
    }

    /// Create a CityModel from a stream of CityJSONFeatures, aggregating them into the CityModel's
    /// CityObjects. Assumes that the first item in the stream is a CityJSON object.
    pub fn from_stream<R>(cursor: R) -> Result<Self>
    where
        R: BufRead,
    {
        todo!()
    }

    pub fn version(&self) -> &Option<CityJSONVersion> {
        &self.version
    }

    fn set_version(&mut self, version: CityJSONVersion) {
        self.version = Some(version);
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
        Self {
            id: None,
            type_model: CityModelType(serde_cityjson::CityModelType::default()),
            version: Some(CityJSONVersion(serde_cityjson::CityJSONVersion::default())),
            transform: None,
            extensions: None,
        }
    }
}

impl fmt::Debug for CityModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CityModel")
            .field("version", &self.version)
            .field("transform", &self.transform)
            .finish()
    }
}

impl fmt::Display for CityModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(\n\tversion: {}\n\tnr. cityobjects: )", &self.version)
    }
}

pub struct CityModelType(serde_cityjson::CityModelType);

pub struct CityJSONVersion(serde_cityjson::CityJSONVersion);

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct Extensions(v1_1::Extensions);

#[derive(Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Extension(v1_1::Extension);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform(v1_1::Transform);
