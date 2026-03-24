use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::Deserialize;

use crate::format::{ActiveCityJsonBoundary, CityJsonBoundary};
use crate::{CityJSONVersion, CityModel, Error, Result};

#[derive(Debug, Deserialize)]
struct Header {
    #[serde(rename = "type")]
    kind: String,
    version: Option<String>,
}

pub(crate) fn from_slice(bytes: &[u8]) -> Result<CityModel> {
    let header: Header = serde_json::from_slice(bytes)?;
    match header.kind.as_str() {
        "CityJSON" => match CityJSONVersion::try_from(
            header.version.as_deref().ok_or(Error::MissingVersion)?,
        )? {
            CityJSONVersion::V2_0 => ActiveCityJsonBoundary::import_document_v2(bytes),
            CityJSONVersion::V1_0 => todo!(),
            CityJSONVersion::V1_1 => todo!(),
        },
        "CityJSONFeature" => Err(Error::ExpectedCityJSON(header.kind)),
        _ => Err(Error::UnsupportedType(header.kind)),
    }
}

pub(crate) fn from_file<P: AsRef<Path>>(path: P) -> Result<CityModel> {
    let path = path.as_ref();
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("jsonl") => from_stream(BufReader::new(File::open(path)?)),
        _ => from_slice(&std::fs::read(path)?),
    }
}

pub(crate) fn from_stream<R>(reader: R) -> Result<CityModel>
where
    R: BufRead,
{
    let mut first_line = None;
    let mut remaining = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        if first_line.is_none() {
            first_line = Some(line);
        } else {
            remaining.push(line);
        }
    }

    let first_line = first_line
        .ok_or_else(|| Error::Streaming("stream does not contain any JSON values".into()))?;

    let header: Header = serde_json::from_str(&first_line)?;
    if header.kind != "CityJSON" {
        return Err(Error::ExpectedCityJSON(header.kind));
    }

    let version =
        CityJSONVersion::try_from(header.version.as_deref().ok_or(Error::MissingVersion)?)?;
    let mut model = match version {
        CityJSONVersion::V2_0 => ActiveCityJsonBoundary::import_document_v2(first_line.as_bytes())?,
        CityJSONVersion::V1_0 => todo!(),
        CityJSONVersion::V1_1 => todo!(),
    };

    for line in remaining {
        let header: Header = serde_json::from_str(&line)?;
        if header.kind != "CityJSONFeature" {
            return Err(Error::ExpectedCityJSONFeature(header.kind));
        }

        if let Some(feature_version) = header.version {
            let feature_version = CityJSONVersion::try_from(feature_version)?;
            if feature_version != version {
                return Err(Error::Streaming(format!(
                    "mixed CityJSON versions in stream: root is {version}, feature is {feature_version}"
                )));
            }
        }

        match version {
            CityJSONVersion::V2_0 => {
                ActiveCityJsonBoundary::merge_feature_v2(&mut model, line.as_bytes())?
            }
            CityJSONVersion::V1_0 => todo!(),
            CityJSONVersion::V1_1 => todo!(),
        }
    }

    Ok(model)
}
