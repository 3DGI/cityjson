use std::fmt::{Display, Formatter};
use std::io::{BufRead, Write};
use std::path::Path;

pub use cityjson_json::CityJsonSeqWriteReport;
use serde::Deserialize;
use serde_json::Value;

use crate::{CityJSONVersion, CityModel, Error, Result};

pub use cityjson_json::v2_0::{
    CityJsonSeqReader, CityJsonSeqWriteOptions, FeatureStreamTransform,
    ReadOptions as JsonReadOptions, WriteOptions as JsonWriteOptions, read_feature,
    read_feature_stream as read_feature_stream_raw,
    read_feature_with_base as read_feature_with_base_raw, read_model, to_vec as to_vec_raw,
    write_feature_stream as write_feature_stream_raw, write_model,
};

pub mod staged {
    use std::io::Write;
    use std::path::Path;

    use serde_json::value::RawValue;
    use serde_json::{Map, Value};

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
        let base =
            cityjson_json::read_model(base_document_bytes, &cityjson_json::ReadOptions::default())?;
        cityjson_json::read_feature_with_base(
            feature_bytes,
            &base,
            &cityjson_json::ReadOptions::default(),
        )
        .map(CityModel::from)
        .map_err(Error::from)
    }

    pub fn from_feature_slice_with_base_assume_cityjson_feature_v2_0(
        feature_bytes: &[u8],
        base_document_bytes: &[u8],
    ) -> Result<CityModel> {
        from_feature_slice_with_base(feature_bytes, base_document_bytes)
    }

    pub fn from_feature_assembly_with_base(
        assembly: FeatureAssembly<'_>,
        base_document_bytes: &[u8],
    ) -> Result<CityModel> {
        let mut cityobjects = Map::with_capacity(assembly.cityobjects.len());
        for cityobject in assembly.cityobjects {
            cityobjects.insert(
                cityobject.id.to_owned(),
                serde_json::from_str::<Value>(cityobject.object.get())?,
            );
        }

        let feature = serde_json::json!({
            "type": "CityJSONFeature",
            "id": assembly.id,
            "CityObjects": cityobjects,
            "vertices": assembly.vertices,
        });
        let bytes = serde_json::to_vec(&feature)?;
        from_feature_slice_with_base(&bytes, base_document_bytes)
    }

    pub fn from_feature_file_with_base<P: AsRef<Path>>(
        path: P,
        base_document_bytes: &[u8],
    ) -> Result<CityModel> {
        from_feature_slice_with_base(&std::fs::read(path)?, base_document_bytes)
    }

    pub fn to_feature_writer(writer: &mut impl Write, model: &CityModel) -> Result<()> {
        match model.type_citymodel() {
            cityjson::CityModelType::CityJSONFeature => {
                cityjson_json::write_model(writer, model, &cityjson_json::WriteOptions::default())
                    .map_err(Error::from)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WriteOptions {
    pub pretty: bool,
    pub validate_default_themes: bool,
}

fn write_options(options: WriteOptions) -> cityjson_json::WriteOptions {
    cityjson_json::WriteOptions {
        pretty: options.pretty,
        validate_default_themes: options.validate_default_themes,
        trailing_newline: false,
    }
}

pub fn from_slice_assume_cityjson_v2_0(bytes: &[u8]) -> Result<CityModel> {
    cityjson_json::read_model(bytes, &cityjson_json::ReadOptions::default())
        .map(CityModel::from)
        .map_err(Error::from)
}

pub fn from_slice(bytes: &[u8]) -> Result<CityModel> {
    let probe = probe(bytes)?;
    match probe.kind {
        RootKind::CityJSON => match probe.version {
            Some(CityJSONVersion::V2_0) => from_slice_assume_cityjson_v2_0(bytes),
            None => Err(Error::MissingVersion),
            Some(other) => Err(Error::UnsupportedVersion {
                found: other.to_string(),
                supported: CityJSONVersion::V2_0.to_string(),
            }),
        },
        RootKind::CityJSONFeature => Err(Error::ExpectedCityJSON(probe.kind.to_string())),
    }
}

pub fn from_file<P: AsRef<Path>>(path: P) -> Result<CityModel> {
    let path = path.as_ref();
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("jsonl") => Err(Error::UnsupportedFeature(
            "CityJSONFeature streams must be read with json::read_feature_stream".into(),
        )),
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
    cityjson_json::read_feature(bytes, &cityjson_json::ReadOptions::default())
        .map(CityModel::from)
        .map_err(Error::from)
}

pub fn read_feature_stream<R>(reader: R) -> Result<impl Iterator<Item = Result<CityModel>>>
where
    R: BufRead,
{
    let iter = cityjson_json::read_feature_stream(reader, &cityjson_json::ReadOptions::default())?;
    Ok(iter.map(|item| item.map(CityModel::from).map_err(Error::from)))
}

pub fn read_cityjsonseq<R>(reader: R) -> Result<impl Iterator<Item = Result<CityModel>>>
where
    R: BufRead,
{
    read_feature_stream(reader)
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

pub fn write_cityjsonseq<I, W>(
    writer: W,
    base_root: &CityModel,
    features: I,
    transform: &cityjson::v2_0::Transform,
) -> Result<CityJsonSeqWriteReport>
where
    I: IntoIterator<Item = CityModel>,
    W: Write,
{
    let features = features.into_iter().collect::<Vec<_>>();
    write_cityjsonseq_refs(writer, base_root, features.iter(), transform)
}

pub fn write_cityjsonseq_refs<'a, I, W>(
    writer: W,
    _base_root: &CityModel,
    features: I,
    transform: &cityjson::v2_0::Transform,
) -> Result<CityJsonSeqWriteReport>
where
    I: IntoIterator<Item = &'a CityModel>,
    W: Write,
{
    let options = cityjson_json::CityJsonSeqWriteOptions {
        transform: cityjson_json::FeatureStreamTransform::Explicit(transform.clone()),
        ..cityjson_json::CityJsonSeqWriteOptions::default()
    };
    cityjson_json::write_feature_stream(writer, features.into_iter().cloned(), &options)
        .map_err(Error::from)
}

pub fn write_cityjsonseq_auto_transform<I, W>(
    writer: W,
    base_root: &CityModel,
    features: I,
    scale: [f64; 3],
) -> Result<CityJsonSeqWriteReport>
where
    I: IntoIterator<Item = CityModel>,
    W: Write,
{
    let features = features.into_iter().collect::<Vec<_>>();
    write_cityjsonseq_auto_transform_refs(writer, base_root, features.iter(), scale)
}

pub fn write_cityjsonseq_auto_transform_refs<'a, I, W>(
    writer: W,
    _base_root: &CityModel,
    features: I,
    scale: [f64; 3],
) -> Result<CityJsonSeqWriteReport>
where
    I: IntoIterator<Item = &'a CityModel>,
    W: Write,
{
    let options = cityjson_json::CityJsonSeqWriteOptions {
        transform: cityjson_json::FeatureStreamTransform::Auto { scale },
        ..cityjson_json::CityJsonSeqWriteOptions::default()
    };
    cityjson_json::write_feature_stream(writer, features.into_iter().cloned(), &options)
        .map_err(Error::from)
}

pub fn to_vec(model: &CityModel) -> Result<Vec<u8>> {
    cityjson_json::to_vec(model, &write_options(WriteOptions::default())).map_err(Error::from)
}

pub fn to_vec_with_options(model: &CityModel, options: WriteOptions) -> Result<Vec<u8>> {
    cityjson_json::to_vec(model, &write_options(options)).map_err(Error::from)
}

pub fn to_string(model: &CityModel) -> Result<String> {
    String::from_utf8(to_vec(model)?).map_err(|error| Error::Import(error.to_string()))
}

pub fn to_string_with_options(model: &CityModel, options: WriteOptions) -> Result<String> {
    String::from_utf8(to_vec_with_options(model, options)?)
        .map_err(|error| Error::Import(error.to_string()))
}

pub fn to_writer(writer: &mut impl Write, model: &CityModel) -> Result<()> {
    cityjson_json::write_model(writer, model, &write_options(WriteOptions::default()))
        .map_err(Error::from)
}

pub fn to_writer_with_options(
    writer: &mut impl Write,
    model: &CityModel,
    options: WriteOptions,
) -> Result<()> {
    cityjson_json::write_model(writer, model, &write_options(options)).map_err(Error::from)
}

pub fn to_feature_string(model: &CityModel) -> Result<String> {
    to_feature_string_with_options(model, WriteOptions::default())
}

pub fn to_feature_vec_with_options(model: &CityModel, options: WriteOptions) -> Result<Vec<u8>> {
    Ok(to_feature_string_with_options(model, options)?.into_bytes())
}

pub fn to_feature_string_with_options(model: &CityModel, options: WriteOptions) -> Result<String> {
    match model.type_citymodel() {
        cityjson::CityModelType::CityJSONFeature => to_string_with_options(model, options),
        other => Err(Error::UnsupportedType(other.to_string())),
    }
}

pub fn to_feature_writer(writer: &mut impl Write, model: &CityModel) -> Result<()> {
    staged::to_feature_writer(writer, model)
}

pub fn merge_feature_stream_slice(bytes: &[u8]) -> Result<CityModel> {
    let mut stream = serde_json::Deserializer::from_slice(bytes).into_iter::<Value>();
    let Some(first) = stream.next().transpose()? else {
        return Err(Error::Import("empty feature stream".into()));
    };
    let first = match first {
        Value::Object(map) => map,
        _ => return Err(Error::Import("stream items must be JSON objects".into())),
    };
    let first_bytes = serde_json::to_vec(&Value::Object(first.clone()))?;

    if matches!(probe(&first_bytes)?.kind(), RootKind::CityJSON) {
        let reader = std::io::Cursor::new(bytes);
        let mut merged = from_slice(&first_bytes)?;
        for feature in read_cityjsonseq(reader)? {
            crate::ops::append(&mut merged, &feature?)?;
        }
        return Ok(merged);
    }

    let mut models = vec![from_feature_slice(&first_bytes)?];
    for item in stream {
        let item = match item? {
            Value::Object(map) => map,
            _ => return Err(Error::Import("stream items must be JSON objects".into())),
        };
        let item_bytes = serde_json::to_vec(&Value::Object(item))?;
        models.push(from_feature_slice(&item_bytes)?);
    }
    crate::ops::merge(models)
}

pub fn merge_cityjsonseq_slice(bytes: &[u8]) -> Result<CityModel> {
    merge_feature_stream_slice(bytes)
}
