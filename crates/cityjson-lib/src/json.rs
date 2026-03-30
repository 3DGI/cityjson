use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, Write};
use std::path::Path;

use serde::Deserialize;

use crate::{CityJSONVersion, CityModel, Error, Result};

pub mod staged {
    use std::io::Write;
    use std::path::Path;

    use serde_json::value::RawValue;

    use crate::{CityModel, Error, Result};

    #[derive(Debug, Clone, Copy)]
    pub struct FeatureObjectFragment<'a> {
        pub id: &'a str,
        pub object: &'a RawValue,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct FeatureAssembly<'a> {
        pub id: &'a str,
        pub cityobjects: &'a [FeatureObjectFragment<'a>],
        pub vertices: &'a [[i64; 3]],
    }

    pub fn from_feature_slice_with_base(
        feature_bytes: &[u8],
        base_document_bytes: &[u8],
    ) -> Result<CityModel> {
        let probe = super::probe(feature_bytes)?;
        match probe.kind() {
            super::RootKind::CityJSON => {
                Err(Error::ExpectedCityJSONFeature(probe.kind().to_string()))
            }
            super::RootKind::CityJSONFeature => {
                let feature_input = std::str::from_utf8(feature_bytes).map_err(|error| {
                    Error::Json(serde_json::Error::io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        error,
                    )))
                })?;
                let base_input = std::str::from_utf8(base_document_bytes).map_err(|error| {
                    Error::Json(serde_json::Error::io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        error,
                    )))
                })?;
                Ok(CityModel(
                    serde_cityjson::v2_0::from_feature_str_owned_with_base(
                        feature_input,
                        base_input,
                    )?,
                ))
            }
        }
    }

    pub fn from_feature_slice_with_base_assume_cityjson_feature_v2_0(
        feature_bytes: &[u8],
        base_document_bytes: &[u8],
    ) -> Result<CityModel> {
        let feature_input = super::decode_utf8_input(feature_bytes, "CityJSONFeature document")?;
        let base_input = super::decode_utf8_input(base_document_bytes, "CityJSON document")?;
        Ok(CityModel(
            serde_cityjson::v2_0::from_feature_str_owned_with_base(feature_input, base_input)?,
        ))
    }

    pub fn from_feature_assembly_with_base(
        assembly: FeatureAssembly<'_>,
        base_document_bytes: &[u8],
    ) -> Result<CityModel> {
        let base_input = std::str::from_utf8(base_document_bytes).map_err(|error| {
            Error::Json(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                error,
            )))
        })?;
        let cityobjects = assembly
            .cityobjects
            .iter()
            .map(|cityobject| serde_cityjson::FeatureObject {
                id: cityobject.id,
                object: cityobject.object,
            })
            .collect::<Vec<_>>();
        let parts = serde_cityjson::FeatureParts {
            id: assembly.id,
            cityobjects: &cityobjects,
            vertices: assembly.vertices,
        };
        Ok(CityModel(
            serde_cityjson::from_feature_parts_owned_with_base(parts, base_input)?,
        ))
    }

    pub fn from_feature_file_with_base<P: AsRef<Path>>(
        path: P,
        base_document_bytes: &[u8],
    ) -> Result<CityModel> {
        from_feature_slice_with_base(&std::fs::read(path)?, base_document_bytes)
    }

    pub fn to_feature_writer(writer: &mut impl Write, model: &CityModel) -> Result<()> {
        match model.as_inner().type_citymodel() {
            cityjson::CityModelType::CityJSONFeature => {
                serde_cityjson::to_writer_validated(writer, model.as_inner())?;
                Ok(())
            }
            other => Err(Error::UnsupportedType(other.to_string())),
        }
    }
}

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

fn decode_utf8_input<'a>(bytes: &'a [u8], label: &str) -> Result<&'a str> {
    std::str::from_utf8(bytes).map_err(|error| {
        Error::Json(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("{label} is not valid UTF-8: {error}"),
        )))
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WriteOptions {
    pub pretty: bool,
    pub validate_default_themes: bool,
}

pub fn from_slice_assume_cityjson_v2_0(bytes: &[u8]) -> Result<CityModel> {
    let input = decode_utf8_input(bytes, "CityJSON document")?;
    Ok(CityModel(serde_cityjson::v2_0::from_str_owned(input)?))
}

pub fn from_slice(bytes: &[u8]) -> Result<CityModel> {
    let probe = probe(bytes)?;
    match probe.kind {
        RootKind::CityJSON => match probe.version {
            None => Err(Error::MissingVersion),
            Some(CityJSONVersion::V2_0) => from_slice_assume_cityjson_v2_0(bytes),
            Some(CityJSONVersion::V1_0) => todo!(),
            Some(CityJSONVersion::V1_1) => todo!(),
        },
        RootKind::CityJSONFeature => Err(Error::ExpectedCityJSON(probe.kind.to_string())),
    }
}

pub fn from_file<P: AsRef<Path>>(path: P) -> Result<CityModel> {
    let path = path.as_ref();
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("jsonl") => {
            let _ = File::open(path)?;
            Err(Error::UnsupportedFeature(
                "CityJSONFeature streams must be read with json::read_feature_stream".into(),
            ))
        }
        _ => from_slice(&std::fs::read(path)?),
    }
}

pub fn from_feature_slice(bytes: &[u8]) -> Result<CityModel> {
    let probe = probe(bytes)?;
    match probe.kind {
        RootKind::CityJSON => Err(Error::ExpectedCityJSONFeature(probe.kind.to_string())),
        RootKind::CityJSONFeature => from_feature_slice_assume_cityjson_feature_v2_0(bytes),
    }
}

pub fn from_feature_file<P: AsRef<Path>>(path: P) -> Result<CityModel> {
    from_feature_slice(&std::fs::read(path)?)
}

pub fn from_feature_slice_assume_cityjson_feature_v2_0(bytes: &[u8]) -> Result<CityModel> {
    let input = decode_utf8_input(bytes, "CityJSONFeature document")?;
    Ok(CityModel(serde_cityjson::v2_0::from_feature_str_owned(
        input,
    )?))
}

pub fn read_feature_stream<R>(reader: R) -> Result<impl Iterator<Item = Result<CityModel>>>
where
    R: BufRead,
{
    let iter = serde_cityjson::v2_0::read_feature_stream(reader)?;
    Ok(iter.map(|item| item.map(CityModel::from).map_err(Error::from)))
}

pub fn write_feature_stream<I, W>(mut writer: W, models: I) -> Result<()>
where
    I: IntoIterator<Item = CityModel>,
    W: Write,
{
    for model in models {
        staged::to_feature_writer(&mut writer, &model)?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

pub fn write_feature_stream_refs<'a, I, W>(mut writer: W, models: I) -> Result<()>
where
    I: IntoIterator<Item = &'a CityModel>,
    W: Write,
{
    for model in models {
        staged::to_feature_writer(&mut writer, model)?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

pub fn to_vec(model: &CityModel) -> Result<Vec<u8>> {
    Ok(to_string(model)?.into_bytes())
}

pub fn to_vec_with_options(model: &CityModel, options: WriteOptions) -> Result<Vec<u8>> {
    Ok(to_string_with_options(model, options)?.into_bytes())
}

pub fn to_string(model: &CityModel) -> Result<String> {
    Ok(serde_cityjson::to_string_validated(model.as_inner())?)
}

pub fn to_string_with_options(model: &CityModel, options: WriteOptions) -> Result<String> {
    if options.validate_default_themes {
        model.as_inner().validate_default_themes()?;
    }

    let value = serde_cityjson::as_json(model.as_inner());
    if options.pretty {
        Ok(serde_json::to_string_pretty(&value)?)
    } else {
        Ok(serde_json::to_string(&value)?)
    }
}

pub fn to_writer(writer: &mut impl Write, model: &CityModel) -> Result<()> {
    writer.write_all(to_string(model)?.as_bytes())?;
    Ok(())
}

pub fn to_writer_with_options(
    writer: &mut impl Write,
    model: &CityModel,
    options: WriteOptions,
) -> Result<()> {
    writer.write_all(to_string_with_options(model, options)?.as_bytes())?;
    Ok(())
}

pub fn to_feature_string(model: &CityModel) -> Result<String> {
    Ok(serde_cityjson::v2_0::to_string_feature(model.as_inner())?)
}

pub fn to_feature_vec_with_options(model: &CityModel, options: WriteOptions) -> Result<Vec<u8>> {
    Ok(to_feature_string_with_options(model, options)?.into_bytes())
}

pub fn to_feature_string_with_options(model: &CityModel, options: WriteOptions) -> Result<String> {
    match model.as_inner().type_citymodel() {
        cityjson::CityModelType::CityJSONFeature => {
            if options.validate_default_themes {
                model.as_inner().validate_default_themes()?;
            }

            let value = serde_cityjson::as_json(model.as_inner());
            if options.pretty {
                Ok(serde_json::to_string_pretty(&value)?)
            } else {
                Ok(serde_json::to_string(&value)?)
            }
        }
        other => Err(Error::UnsupportedType(other.to_string())),
    }
}

pub fn to_feature_writer(writer: &mut impl Write, model: &CityModel) -> Result<()> {
    staged::to_feature_writer(writer, model)
}

pub fn merge_feature_stream_slice(bytes: &[u8]) -> Result<CityModel> {
    let mut stream = serde_json::Deserializer::from_slice(bytes).into_iter::<serde_json::Value>();
    let first = stream
        .next()
        .transpose()?
        .ok_or_else(|| Error::Import("empty feature stream".into()))?;
    let first = match first {
        serde_json::Value::Object(map) => map,
        _ => return Err(Error::Import("stream items must be JSON objects".into())),
    };

    let first_bytes = serde_json::to_vec(&serde_json::Value::Object(first.clone()))?;
    let root_probe = probe(&first_bytes)?;
    let mut models = Vec::new();
    match root_probe.kind() {
        RootKind::CityJSON => models.push(from_slice(&first_bytes)?),
        RootKind::CityJSONFeature => models.push(from_feature_slice(&first_bytes)?),
    }

    for item in stream {
        let item = item?;
        let item = match item {
            serde_json::Value::Object(map) => map,
            _ => return Err(Error::Import("stream items must be JSON objects".into())),
        };
        let item_bytes = serde_json::to_vec(&serde_json::Value::Object(item))?;
        let item_probe = probe(&item_bytes)?;
        match item_probe.kind() {
            RootKind::CityJSON => models.push(from_slice(&item_bytes)?),
            RootKind::CityJSONFeature => models.push(from_feature_slice(&item_bytes)?),
        }
    }

    crate::ops::merge(models)
}

#[cfg(test)]
mod tests {
    use super::{
        RootKind, from_feature_slice, from_feature_slice_assume_cityjson_feature_v2_0, from_slice,
        from_slice_assume_cityjson_v2_0, probe, staged,
    };

    fn v2_document() -> &'static [u8] {
        include_bytes!("../tests/data/v2_0/minimal.city.json")
    }

    fn feature_document() -> &'static [u8] {
        br#"{"type":"CityJSONFeature","version":"2.0","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}"#
    }

    #[test]
    fn trusted_document_parse_matches_regular_parse() {
        let regular = from_slice(v2_document()).expect("regular parse should succeed");
        let trusted =
            from_slice_assume_cityjson_v2_0(v2_document()).expect("trusted parse should succeed");

        assert_eq!(
            regular.as_inner().cityobjects().len(),
            trusted.as_inner().cityobjects().len()
        );
        assert_eq!(
            regular.as_inner().geometry_count(),
            trusted.as_inner().geometry_count()
        );
        assert_eq!(
            regular.as_inner().vertices().len(),
            trusted.as_inner().vertices().len()
        );
    }

    #[test]
    fn trusted_feature_parse_matches_regular_parse() {
        let regular = from_feature_slice(feature_document()).expect("regular parse should succeed");
        let trusted = from_feature_slice_assume_cityjson_feature_v2_0(feature_document())
            .expect("trusted feature parse should succeed");

        assert_eq!(
            regular.as_inner().cityobjects().len(),
            trusted.as_inner().cityobjects().len()
        );
        assert_eq!(
            regular.as_inner().geometry_count(),
            trusted.as_inner().geometry_count()
        );
        assert_eq!(
            regular.as_inner().vertices().len(),
            trusted.as_inner().vertices().len()
        );
    }

    #[test]
    fn trusted_feature_with_base_parse_matches_regular_parse() {
        let regular = staged::from_feature_slice_with_base(feature_document(), v2_document())
            .expect("regular feature-with-base parse should succeed");
        let trusted = staged::from_feature_slice_with_base_assume_cityjson_feature_v2_0(
            feature_document(),
            v2_document(),
        )
        .expect("trusted feature-with-base parse should succeed");

        assert_eq!(
            regular.as_inner().cityobjects().len(),
            trusted.as_inner().cityobjects().len()
        );
        assert_eq!(
            regular.as_inner().geometry_count(),
            trusted.as_inner().geometry_count()
        );
        assert_eq!(
            regular.as_inner().vertices().len(),
            trusted.as_inner().vertices().len()
        );
    }

    #[test]
    fn trusted_document_parse_skips_root_validation() {
        let parsed = from_slice_assume_cityjson_v2_0(feature_document())
            .expect("trusted parse should delegate root validation to caller");

        let header = probe(feature_document()).expect("probe should succeed");
        assert_eq!(header.kind(), RootKind::CityJSONFeature);
        assert_eq!(
            parsed.as_inner().type_citymodel().to_string(),
            "CityJSONFeature"
        );
    }
}
