use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use serde::Deserialize;

use crate::{CityJSONVersion, CityModel, Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootKind {
    CityJSON,
    CityJSONFeature,
}

impl RootKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::CityJSON => "CityJSON",
            Self::CityJSONFeature => "CityJSONFeature",
        }
    }
}

impl Display for RootKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Probe {
    kind: RootKind,
    version: Option<CityJSONVersion>,
}

impl Probe {
    pub fn kind(&self) -> RootKind {
        self.kind
    }

    pub fn version(&self) -> Option<CityJSONVersion> {
        self.version
    }
}

#[derive(Debug, Deserialize)]
struct Header {
    #[serde(rename = "type")]
    kind: String,
    version: Option<String>,
}

pub fn probe(bytes: &[u8]) -> Result<Probe> {
    let header: Header = serde_json::from_slice(bytes)?;
    let kind = match header.kind.as_str() {
        "CityJSON" => RootKind::CityJSON,
        "CityJSONFeature" => RootKind::CityJSONFeature,
        other => return Err(Error::UnsupportedType(other.to_owned())),
    };

    let version = match header.version {
        Some(version) => Some(CityJSONVersion::try_from(version)?),
        None => None,
    };

    Ok(Probe { kind, version })
}

pub fn from_slice(bytes: &[u8]) -> Result<CityModel> {
    let probe = probe(bytes)?;
    match probe.kind {
        RootKind::CityJSON => match probe.version {
            None => Err(Error::MissingVersion),
            Some(CityJSONVersion::V2_0) => {
                let input = std::str::from_utf8(bytes).map_err(|error| {
                    Error::Json(serde_json::Error::io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        error,
                    )))
                })?;
                Ok(CityModel(serde_cityjson::v2_0::from_str_owned(input)?))
            }
            Some(CityJSONVersion::V1_0) => todo!(),
            Some(CityJSONVersion::V1_1) => todo!(),
        },
        RootKind::CityJSONFeature => Err(Error::ExpectedCityJSON(probe.kind.to_string())),
    }
}

pub fn from_file<P: AsRef<Path>>(path: P) -> Result<CityModel> {
    let path = path.as_ref();
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("jsonl") => from_stream(BufReader::new(File::open(path)?)),
        _ => from_slice(&std::fs::read(path)?),
    }
}

pub fn from_feature_slice(bytes: &[u8]) -> Result<CityModel> {
    let _ = bytes;
    Err(Error::Streaming(
        "CityJSONFeature parsing is not yet implemented in serde_cityjson".into(),
    ))
}

pub fn merge_feature_stream<R>(reader: R) -> Result<CityModel>
where
    R: BufRead,
{
    let _ = reader;
    Err(Error::Streaming(
        "CityJSONFeature stream aggregation is not yet implemented in serde_cityjson".into(),
    ))
}

pub fn read_feature_stream<R>(reader: R) -> Result<impl Iterator<Item = Result<CityModel>>>
where
    R: BufRead,
{
    let _ = reader;
    let iter = std::iter::empty::<Result<CityModel>>();
    Err(Error::Streaming(
        "CityJSONFeature stream iteration is not yet implemented in serde_cityjson".into(),
    ))?;
    Ok(iter)
}

pub fn from_stream<R>(reader: R) -> Result<CityModel>
where
    R: BufRead,
{
    merge_feature_stream(reader)
}

pub fn to_vec(model: &CityModel) -> Result<Vec<u8>> {
    Ok(to_string(model)?.into_bytes())
}

pub fn to_string(model: &CityModel) -> Result<String> {
    Ok(serde_cityjson::to_string_validated(model.as_inner())?)
}

pub fn to_writer(writer: &mut impl Write, model: &CityModel) -> Result<()> {
    writer.write_all(to_string(model)?.as_bytes())?;
    Ok(())
}

pub fn to_feature_string(model: &CityModel) -> Result<String> {
    to_string(model)
}
