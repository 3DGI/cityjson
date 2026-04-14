use std::collections::HashSet;
use std::io::BufRead;
use std::io::Write;

use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{BBox, BorrowedCityModel, CityModel, OwnedCityModel, Transform, VertexRef};
use cityjson::{CityJSONVersion, CityModelType};
use serde::Serialize;
use serde::ser::SerializeMap;
use serde_json::value::RawValue;
use serde_json::{Map, Value};

pub use crate::de::ParseStringStorage;
use crate::errors::{Error, Result};

#[derive(Debug, Clone, Copy)]
pub struct FeatureObject<'a> {
    pub id: &'a str,
    pub object: &'a RawValue,
}

#[derive(Debug, Clone, Copy)]
pub struct FeatureParts<'a> {
    pub id: &'a str,
    pub cityobjects: &'a [FeatureObject<'a>],
    pub vertices: &'a [[i64; 3]],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CityJSONSeqWriteOptions {
    pub validate_default_themes: bool,
    pub trailing_newline: bool,
    pub update_metadata_geographical_extent: bool,
}

impl Default for CityJSONSeqWriteOptions {
    fn default() -> Self {
        Self {
            validate_default_themes: true,
            trailing_newline: true,
            update_metadata_geographical_extent: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoTransformOptions {
    pub scale: [f64; 3],
    pub validate_default_themes: bool,
    pub trailing_newline: bool,
    pub update_metadata_geographical_extent: bool,
}

impl Default for AutoTransformOptions {
    fn default() -> Self {
        Self {
            scale: [0.001, 0.001, 0.001],
            validate_default_themes: true,
            trailing_newline: true,
            update_metadata_geographical_extent: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CityJSONSeqWriteReport {
    pub transform: Transform,
    pub geographical_extent: Option<BBox>,
    pub feature_count: usize,
    pub cityobject_count: usize,
}

/// Parse a `CityJSON` document into a [`CityModel`].
///
/// # Errors
///
/// Returns an error if the input is not valid `CityJSON`.
pub fn from_str<'de, SS>(input: &'de str) -> Result<CityModel<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    crate::de::from_str_generic::<SS>(input)
}

/// Parse a `CityJSON` document into an [`OwnedCityModel`].
///
/// # Errors
///
/// Returns an error if the input is not valid `CityJSON`.
pub fn from_str_owned(input: &str) -> Result<OwnedCityModel> {
    crate::de::from_str_owned(input)
}

/// Parse a `CityJSONFeature` object into an [`OwnedCityModel`].
///
/// # Errors
///
/// Returns an error if the input is not valid `CityJSONFeature`.
pub fn from_feature_str_owned(input: &str) -> Result<OwnedCityModel> {
    let model = from_str_owned(input)?;
    match model.type_citymodel() {
        CityModelType::CityJSONFeature => Ok(model),
        other => Err(Error::UnsupportedType(other.to_string())),
    }
}

/// Parse a standalone `CityJSONFeature` object using the non-feature root state
/// from a companion `CityJSON` document.
///
/// This mirrors [`read_feature_stream`] for deployments where the metadata
/// document and feature files live separately on disk.
///
/// # Errors
///
/// Returns an error if the base document is not valid `CityJSON`, the feature is
/// not valid `CityJSONFeature`, or the combined document cannot be parsed.
pub fn from_feature_str_owned_with_base(
    feature_input: &str,
    base_document_input: &str,
) -> Result<OwnedCityModel> {
    let aggregate_root = into_object(serde_json::from_str(base_document_input)?)?;
    let version = ensure_document_root(&aggregate_root)?;
    let base_root = build_feature_base_root(&aggregate_root);
    let feature = into_object(serde_json::from_str(feature_input)?)?;
    ensure_feature_root(&feature, version)?;
    let input = serde_json::to_string(&Value::Object(materialize_feature_document(
        &base_root, feature,
    )))?;
    from_feature_str_owned(&input)
}

/// Parse a standalone `CityJSONFeature` assembled from typed feature parts and
/// the non-feature root state of a companion `CityJSON` document.
///
/// # Errors
///
/// Returns an error if the base document is not valid `CityJSON`, the feature
/// parts are inconsistent, or the combined document cannot be parsed.
pub fn from_feature_parts_owned_with_base(
    parts: FeatureParts<'_>,
    base_document_input: &str,
) -> Result<OwnedCityModel> {
    validate_feature_parts(parts)?;
    let aggregate_root = into_object(serde_json::from_str(base_document_input)?)?;
    ensure_document_root(&aggregate_root)?;
    let base_root = build_feature_base_root(&aggregate_root);
    let input = serde_json::to_string(&MaterializedFeaturePartsDocument {
        base_root: &base_root,
        parts,
    })?;
    from_feature_str_owned(&input)
}

/// Parse a `CityJSON` document into a [`BorrowedCityModel`].
///
/// # Errors
///
/// Returns an error if the input is not valid `CityJSON`.
pub fn from_str_borrowed(input: &str) -> Result<BorrowedCityModel<'_>> {
    crate::de::from_str_borrowed(input)
}

/// Read a strict `CityJSON` + `CityJSONFeature` stream into self-contained feature models.
///
/// # Errors
///
/// Returns an error if the stream is empty, the first non-empty item is not
/// `CityJSON`, a later item is not `CityJSONFeature`, versions conflict, or
/// feature IDs are duplicated across the stream.
pub fn read_feature_stream<R>(reader: R) -> Result<impl Iterator<Item = Result<OwnedCityModel>>>
where
    R: BufRead,
{
    let parsed = parse_feature_stream(reader)?;
    let mut models = Vec::with_capacity(parsed.features.len());
    for feature in parsed.features {
        let feature = materialize_feature_document(&parsed.base_root, feature);
        let input = serde_json::to_string(&Value::Object(feature))?;
        models.push(from_feature_str_owned(&input));
    }
    Ok(models.into_iter())
}

/// Merge a strict `CityJSON` + `CityJSONFeature` stream into one [`OwnedCityModel`].
///
/// # Errors
///
/// Returns an error if the stream shape is invalid or if feature items carry
/// incompatible root state that cannot be merged without loss.
pub fn merge_feature_stream<R>(reader: R) -> Result<OwnedCityModel>
where
    R: BufRead,
{
    let mut parsed = parse_feature_stream(reader)?;
    for feature in parsed.features {
        merge_feature_into_root(&mut parsed.aggregate_root, feature)?;
    }

    let input = serde_json::to_string(&Value::Object(parsed.aggregate_root))?;
    let model = from_str_owned(&input)?;
    match model.type_citymodel() {
        CityModelType::CityJSON => Ok(model),
        other => Err(Error::UnsupportedType(other.to_string())),
    }
}

/// Write a strict `CityJSONSeq` stream from a canonical base root and feature packages.
///
/// The output starts with one `CityJSON` header item followed by `CityJSONFeature`
/// items quantized against the provided stream-level transform.
///
/// # Errors
///
/// Returns an error if the base root is not a valid canonical `CityJSON` root,
/// if feature packages are invalid or incompatible, or if serialization fails.
pub fn write_cityjsonseq_with_transform_refs<'a, W, I, VR, SS>(
    writer: W,
    base_root: &CityModel<VR, SS>,
    features: I,
    transform: &Transform,
    options: CityJSONSeqWriteOptions,
) -> Result<CityJSONSeqWriteReport>
where
    W: Write,
    I: IntoIterator<Item = &'a CityModel<VR, SS>>,
    VR: VertexRef + Serialize + 'a,
    SS: StringStorage + 'a,
{
    let features: Vec<_> = features.into_iter().collect();
    write_cityjsonseq_with_transform_slice(writer, base_root, &features, transform, options)
}

/// Write a strict `CityJSONSeq` stream, deriving translation from the overall
/// feature extent and taking the quantization scale from the provided options.
///
/// # Errors
///
/// Returns an error if the base root is invalid, feature packages are invalid,
/// or serialization fails.
pub fn write_cityjsonseq_auto_transform_refs<'a, W, I, VR, SS>(
    writer: W,
    base_root: &CityModel<VR, SS>,
    features: I,
    options: AutoTransformOptions,
) -> Result<CityJSONSeqWriteReport>
where
    W: Write,
    I: IntoIterator<Item = &'a CityModel<VR, SS>>,
    VR: VertexRef + Serialize + 'a,
    SS: StringStorage + 'a,
{
    let features: Vec<_> = features.into_iter().collect();
    let extent = collect_features_extent(&features);
    let mut transform = Transform::new();
    transform.set_scale(options.scale);
    transform.set_translate(extent.as_ref().map_or([0.0, 0.0, 0.0], |bbox| {
        [bbox.min_x(), bbox.min_y(), bbox.min_z()]
    }));

    write_cityjsonseq_with_transform_slice(
        writer,
        base_root,
        &features,
        &transform,
        CityJSONSeqWriteOptions {
            validate_default_themes: options.validate_default_themes,
            trailing_newline: options.trailing_newline,
            update_metadata_geographical_extent: options.update_metadata_geographical_extent,
        },
    )
}

/// Serialize a [`CityModel`] to a `CityJSON` string.
///
/// # Errors
///
/// Returns an error if the model cannot be serialized.
pub fn to_string<VR, SS>(model: &CityModel<VR, SS>) -> Result<String>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    Ok(serde_json::to_string(&as_json(model))?)
}

/// Serialize a [`CityModel`] to a `CityJSON` string, validating default themes.
///
/// # Errors
///
/// Returns an error if the model fails validation or cannot be serialized.
pub fn to_string_validated<VR, SS>(model: &CityModel<VR, SS>) -> Result<String>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    model.validate_default_themes()?;
    Ok(serde_json::to_string(&as_json(model))?)
}

/// Serialize a [`CityModel`] to a `CityJSON` byte vector.
///
/// # Errors
///
/// Returns an error if the model cannot be serialized.
pub fn to_vec<VR, SS>(model: &CityModel<VR, SS>) -> Result<Vec<u8>>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    Ok(serde_json::to_vec(&as_json(model))?)
}

/// Serialize a [`CityModel`] to a `CityJSON` byte vector, validating default themes.
///
/// # Errors
///
/// Returns an error if the model fails validation or cannot be serialized.
pub fn to_vec_validated<VR, SS>(model: &CityModel<VR, SS>) -> Result<Vec<u8>>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    model.validate_default_themes()?;
    Ok(serde_json::to_vec(&as_json(model))?)
}

/// Serialize a [`CityModel`] to a `CityJSON` writer.
///
/// # Errors
///
/// Returns an error if the model cannot be serialized.
pub fn to_writer<W, VR, SS>(writer: W, model: &CityModel<VR, SS>) -> Result<()>
where
    W: Write,
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    Ok(serde_json::to_writer(writer, &as_json(model))?)
}

/// Serialize a [`CityModel`] to a `CityJSON` writer, validating default themes.
///
/// # Errors
///
/// Returns an error if the model fails validation or cannot be serialized.
pub fn to_writer_validated<W, VR, SS>(writer: W, model: &CityModel<VR, SS>) -> Result<()>
where
    W: Write,
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    model.validate_default_themes()?;
    Ok(serde_json::to_writer(writer, &as_json(model))?)
}

/// Serialize a [`CityModel`] as a `CityJSONFeature` string.
///
/// # Errors
///
/// Returns an error if the model is not a `CityJSONFeature` or cannot be serialized.
pub fn to_string_feature<VR, SS>(model: &CityModel<VR, SS>) -> Result<String>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    match model.type_citymodel() {
        CityModelType::CityJSONFeature => to_string_validated(model),
        other => Err(Error::UnsupportedType(other.to_string())),
    }
}

pub fn as_json<VR, SS>(model: &CityModel<VR, SS>) -> SerializableCityModel<'_, VR, SS>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    SerializableCityModel { model }
}

pub struct SerializableCityModel<'a, VR, SS>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    pub(crate) model: &'a CityModel<VR, SS>,
}

impl<VR, SS> Serialize for SerializableCityModel<'_, VR, SS>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::ser::serialize_citymodel(serializer, self.model)
    }
}

struct SerializableCityModelWithOptions<'a, VR, SS>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    model: &'a CityModel<VR, SS>,
    options: crate::ser::CityModelSerializeOptions<'a>,
}

impl<VR, SS> Serialize for SerializableCityModelWithOptions<'_, VR, SS>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::ser::serialize_citymodel_with_options(serializer, self.model, &self.options)
    }
}

struct MaterializedFeaturePartsDocument<'a> {
    base_root: &'a Map<String, Value>,
    parts: FeatureParts<'a>,
}

impl Serialize for MaterializedFeaturePartsDocument<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.base_root.len() + 4))?;
        map.serialize_entry("type", "CityJSONFeature")?;
        for (key, value) in self.base_root {
            map.serialize_entry(key, value)?;
        }
        map.serialize_entry("id", self.parts.id)?;
        map.serialize_entry(
            "CityObjects",
            &SerializableFeatureObjects(self.parts.cityobjects),
        )?;
        map.serialize_entry("vertices", self.parts.vertices)?;
        map.end()
    }
}

struct SerializableFeatureObjects<'a>(&'a [FeatureObject<'a>]);

impl Serialize for SerializableFeatureObjects<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for object in self.0 {
            map.serialize_entry(object.id, object.object)?;
        }
        map.end()
    }
}

struct ParsedFeatureStream {
    base_root: Map<String, Value>,
    aggregate_root: Map<String, Value>,
    features: Vec<Map<String, Value>>,
}

fn parse_feature_stream<R>(reader: R) -> Result<ParsedFeatureStream>
where
    R: BufRead,
{
    let mut stream = serde_json::Deserializer::from_reader(reader).into_iter::<Value>();
    let first = stream
        .next()
        .transpose()?
        .ok_or(Error::MalformedRootObject("empty feature stream"))?;
    let aggregate_root = into_object(first)?;
    let version = ensure_document_root(&aggregate_root)?;
    let mut seen_ids = collect_cityobject_ids(&aggregate_root)?;
    let base_root = build_feature_base_root(&aggregate_root);

    let mut features = Vec::new();
    for item in stream {
        let feature = into_object(item?)?;
        ensure_feature_root(&feature, version)?;
        extend_seen_ids(&mut seen_ids, &feature)?;
        features.push(feature);
    }

    Ok(ParsedFeatureStream {
        base_root,
        aggregate_root,
        features,
    })
}

fn validate_feature_parts(parts: FeatureParts<'_>) -> Result<()> {
    let mut seen_ids = HashSet::with_capacity(parts.cityobjects.len());
    let mut root_id_present = false;
    for object in parts.cityobjects {
        if !seen_ids.insert(object.id) {
            return Err(Error::InvalidValue(format!(
                "duplicate CityObject id in feature parts: {}",
                object.id
            )));
        }
        if object.id == parts.id {
            root_id_present = true;
        }
    }
    if !root_id_present {
        return Err(Error::InvalidValue(format!(
            "feature root id does not resolve to a CityObject: {}",
            parts.id
        )));
    }

    Ok(())
}

fn into_object(value: Value) -> Result<Map<String, Value>> {
    match value {
        Value::Object(map) => Ok(map),
        _ => Err(Error::MalformedRootObject(
            "stream items must be JSON objects",
        )),
    }
}

fn ensure_document_root(root: &Map<String, Value>) -> Result<CityJSONVersion> {
    let kind = root_kind(root)?;
    if kind != CityModelType::CityJSON {
        return Err(Error::MalformedRootObject(
            "first non-empty stream item must be CityJSON",
        ));
    }

    let version = root
        .get("version")
        .and_then(Value::as_str)
        .ok_or(Error::MalformedRootObject("missing root version"))?;
    let version = CityJSONVersion::try_from(version)
        .map_err(|_| Error::UnsupportedVersion(version.to_owned()))?;
    if version != CityJSONVersion::V2_0 {
        return Err(Error::UnsupportedVersion(version.to_string()));
    }
    Ok(version)
}

fn ensure_feature_root(root: &Map<String, Value>, version: CityJSONVersion) -> Result<()> {
    let kind = root_kind(root)?;
    if kind != CityModelType::CityJSONFeature {
        return Err(Error::MalformedRootObject(
            "stream items after the first must be CityJSONFeature",
        ));
    }

    if let Some(found) = root.get("version").and_then(Value::as_str) {
        let found = CityJSONVersion::try_from(found)
            .map_err(|_| Error::UnsupportedVersion(found.to_owned()))?;
        if found != version {
            return Err(Error::InvalidValue(format!(
                "feature stream version mismatch: expected {version}, found {found}"
            )));
        }
    }

    Ok(())
}

fn build_feature_base_root(root: &Map<String, Value>) -> Map<String, Value> {
    root.iter()
        .filter(|(key, _)| {
            !matches!(
                key.as_str(),
                "type" | "version" | "CityObjects" | "vertices"
            )
        })
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn root_kind(root: &Map<String, Value>) -> Result<CityModelType> {
    let type_name = root
        .get("type")
        .and_then(Value::as_str)
        .ok_or(Error::MalformedRootObject("missing root type"))?;
    CityModelType::try_from(type_name).map_err(|_| Error::UnsupportedType(type_name.to_owned()))
}

fn collect_cityobject_ids(root: &Map<String, Value>) -> Result<HashSet<String>> {
    let Some(cityobjects) = root.get("CityObjects") else {
        return Ok(HashSet::new());
    };
    let cityobjects = cityobjects
        .as_object()
        .ok_or(Error::MalformedRootObject("CityObjects must be an object"))?;
    Ok(cityobjects.keys().cloned().collect())
}

fn extend_seen_ids(seen: &mut HashSet<String>, root: &Map<String, Value>) -> Result<()> {
    for id in collect_cityobject_ids(root)? {
        if !seen.insert(id.clone()) {
            return Err(Error::InvalidValue(format!(
                "duplicate CityObject id in feature stream: {id}"
            )));
        }
    }
    Ok(())
}

fn materialize_feature_document(
    base_root: &Map<String, Value>,
    feature: Map<String, Value>,
) -> Map<String, Value> {
    let mut document = base_root.clone();
    document.insert(
        "type".to_owned(),
        Value::String(CityModelType::CityJSONFeature.to_string()),
    );
    for (key, value) in feature {
        if key != "version" {
            document.insert(key, value);
        }
    }
    document
}

fn merge_feature_into_root(
    aggregate_root: &mut Map<String, Value>,
    mut feature: Map<String, Value>,
) -> Result<()> {
    ensure_compatible_feature_root(aggregate_root, &feature)?;

    let vertex_offset = aggregate_root
        .get("vertices")
        .and_then(Value::as_array)
        .ok_or(Error::MalformedRootObject("vertices must be an array"))?
        .len();

    let mut cityobjects = feature
        .remove("CityObjects")
        .ok_or(Error::MalformedRootObject("missing CityObjects"))?;
    offset_feature_cityobject_vertices(&mut cityobjects, vertex_offset)?;

    let feature_vertices = feature
        .remove("vertices")
        .ok_or(Error::MalformedRootObject("missing vertices"))?;
    let feature_vertices = feature_vertices
        .as_array()
        .ok_or(Error::MalformedRootObject("vertices must be an array"))?;

    let aggregate_vertices = aggregate_root
        .get_mut("vertices")
        .and_then(Value::as_array_mut)
        .ok_or(Error::MalformedRootObject("vertices must be an array"))?;
    aggregate_vertices.extend(feature_vertices.iter().cloned());

    let cityobjects = cityobjects
        .as_object()
        .ok_or(Error::MalformedRootObject("CityObjects must be an object"))?;
    let aggregate_cityobjects = aggregate_root
        .get_mut("CityObjects")
        .and_then(Value::as_object_mut)
        .ok_or(Error::MalformedRootObject("CityObjects must be an object"))?;
    for (id, cityobject) in cityobjects {
        aggregate_cityobjects.insert(id.clone(), cityobject.clone());
    }

    Ok(())
}

fn ensure_compatible_feature_root(
    aggregate_root: &Map<String, Value>,
    feature: &Map<String, Value>,
) -> Result<()> {
    for (key, value) in feature {
        if matches!(
            key.as_str(),
            "type" | "version" | "id" | "CityObjects" | "vertices"
        ) {
            continue;
        }

        match aggregate_root.get(key) {
            Some(existing) if existing == value => {}
            Some(_) => {
                return Err(Error::InvalidValue(format!(
                    "feature stream carries incompatible root state for '{key}'"
                )));
            }
            None => {
                return Err(Error::UnsupportedFeature(
                    "feature-specific root sections are not yet mergeable",
                ));
            }
        }
    }

    Ok(())
}

fn offset_feature_cityobject_vertices(value: &mut Value, offset: usize) -> Result<()> {
    match value {
        Value::Array(items) => {
            for item in items {
                offset_feature_cityobject_vertices(item, offset)?;
            }
            Ok(())
        }
        Value::Object(map) => {
            for (key, value) in map {
                if key == "boundaries" {
                    offset_boundary_indices(value, offset)?;
                } else {
                    offset_feature_cityobject_vertices(value, offset)?;
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn offset_boundary_indices(value: &mut Value, offset: usize) -> Result<()> {
    match value {
        Value::Array(items) => {
            for item in items {
                offset_boundary_indices(item, offset)?;
            }
            Ok(())
        }
        Value::Number(number) => {
            let index = number.as_u64().ok_or(Error::MalformedRootObject(
                "boundary indices must be integers",
            ))?;
            let offset = u64::try_from(offset)
                .map_err(|_| Error::InvalidValue("vertex offset overflow".to_owned()))?;
            *value = Value::Number(serde_json::Number::from(index + offset));
            Ok(())
        }
        Value::Null => Ok(()),
        _ => Err(Error::MalformedRootObject(
            "geometry boundaries must be arrays of integer indices",
        )),
    }
}

fn write_cityjsonseq_with_transform_slice<W, VR, SS>(
    mut writer: W,
    base_root: &CityModel<VR, SS>,
    features: &[&CityModel<VR, SS>],
    transform: &Transform,
    options: CityJSONSeqWriteOptions,
) -> Result<CityJSONSeqWriteReport>
where
    W: Write,
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    validate_strict_stream_assembly(base_root, features, options.validate_default_themes)?;

    let geographical_extent = collect_features_extent(features);
    let header_extent = if options.update_metadata_geographical_extent {
        geographical_extent.as_ref()
    } else {
        None
    };
    let header = SerializableCityModelWithOptions {
        model: base_root,
        options: crate::ser::CityModelSerializeOptions {
            type_name: CityModelType::CityJSON,
            include_id: false,
            include_version: true,
            transform: Some(transform),
            include_transform: true,
            include_metadata: true,
            metadata_geographical_extent: header_extent,
            include_extensions: true,
            include_vertices: true,
            include_appearance: true,
            include_geometry_templates: true,
            include_cityobjects: true,
            include_extra: true,
        },
    };
    serde_json::to_writer(&mut writer, &header)?;
    if !features.is_empty() || options.trailing_newline {
        write_newline(&mut writer)?;
    }

    let mut cityobject_count = 0;
    for (index, feature) in features.iter().enumerate() {
        cityobject_count += feature.cityobjects().len();
        let feature_item = SerializableCityModelWithOptions {
            model: feature,
            options: crate::ser::CityModelSerializeOptions {
                type_name: CityModelType::CityJSONFeature,
                include_id: true,
                include_version: false,
                transform: Some(transform),
                include_transform: false,
                include_metadata: false,
                metadata_geographical_extent: None,
                include_extensions: false,
                include_vertices: true,
                include_appearance: false,
                include_geometry_templates: false,
                include_cityobjects: true,
                include_extra: false,
            },
        };
        serde_json::to_writer(&mut writer, &feature_item)?;
        if index + 1 < features.len() || options.trailing_newline {
            write_newline(&mut writer)?;
        }
    }

    Ok(CityJSONSeqWriteReport {
        transform: transform.clone(),
        geographical_extent,
        feature_count: features.len(),
        cityobject_count,
    })
}

fn validate_strict_stream_assembly<VR, SS>(
    base_root: &CityModel<VR, SS>,
    features: &[&CityModel<VR, SS>],
    validate_default_themes: bool,
) -> Result<()>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    ensure_stream_base_root(base_root)?;
    let base_signature = shared_root_signature(base_root)?;
    let mut seen_ids = HashSet::new();

    for feature in features {
        ensure_stream_feature_root(feature)?;
        if validate_default_themes {
            feature.validate_default_themes()?;
        }

        if shared_root_signature(feature)? != base_signature {
            return Err(Error::InvalidValue(
                "feature stream carries incompatible root state".to_owned(),
            ));
        }

        for (_, cityobject) in feature.cityobjects().iter() {
            let id = cityobject.id().to_owned();
            if !seen_ids.insert(id.clone()) {
                return Err(Error::InvalidValue(format!(
                    "duplicate CityObject id in feature stream: {id}"
                )));
            }
        }
    }

    Ok(())
}

fn ensure_stream_base_root<VR, SS>(base_root: &CityModel<VR, SS>) -> Result<()>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    if base_root.type_citymodel() != CityModelType::CityJSON {
        return Err(Error::UnsupportedType(
            base_root.type_citymodel().to_string(),
        ));
    }
    if !base_root.cityobjects().is_empty() {
        return Err(Error::InvalidValue(
            "base root must have empty CityObjects".to_owned(),
        ));
    }
    if !base_root.vertices().is_empty() {
        return Err(Error::InvalidValue(
            "base root must have empty vertices".to_owned(),
        ));
    }
    Ok(())
}

fn ensure_stream_feature_root<VR, SS>(feature: &CityModel<VR, SS>) -> Result<()>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    if feature.type_citymodel() != CityModelType::CityJSONFeature {
        return Err(Error::UnsupportedType(feature.type_citymodel().to_string()));
    }
    if feature.id().is_none() {
        return Err(Error::InvalidValue(
            "CityJSONFeature root id is required".to_owned(),
        ));
    }
    Ok(())
}

fn shared_root_signature<VR, SS>(model: &CityModel<VR, SS>) -> Result<Map<String, Value>>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    let value = serde_json::to_value(&SerializableCityModelWithOptions {
        model,
        options: crate::ser::CityModelSerializeOptions {
            type_name: model.type_citymodel(),
            include_id: false,
            include_version: true,
            transform: model.transform(),
            include_transform: model.transform().is_some(),
            include_metadata: true,
            metadata_geographical_extent: None,
            include_extensions: true,
            include_vertices: false,
            include_appearance: true,
            include_geometry_templates: true,
            include_cityobjects: false,
            include_extra: true,
        },
    })?;
    let mut root = into_object(value)?;
    root.remove("type");
    root.remove("version");
    root.remove("transform");
    if let Some(metadata) = root.get_mut("metadata").and_then(Value::as_object_mut) {
        metadata.remove("geographicalExtent");
        if metadata.is_empty() {
            root.remove("metadata");
        }
    }
    Ok(root)
}

fn collect_features_extent<VR, SS>(features: &[&CityModel<VR, SS>]) -> Option<BBox>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    let mut extent = ExtentAccumulator::default();
    for feature in features {
        for vertex in feature.vertices().as_slice() {
            extent.include([vertex.x(), vertex.y(), vertex.z()]);
        }
        for (_, cityobject) in feature.cityobjects().iter() {
            if let Some(bbox) = cityobject.geographical_extent() {
                extent.include_bbox(*bbox);
            }
        }
    }
    extent.finish()
}

fn write_newline<W>(writer: &mut W) -> Result<()>
where
    W: Write,
{
    writer
        .write_all(b"\n")
        .map_err(|err| Error::Json(serde_json::Error::io(err)))
}

#[derive(Default)]
struct ExtentAccumulator {
    min: Option<[f64; 3]>,
    max: Option<[f64; 3]>,
}

impl ExtentAccumulator {
    fn include(&mut self, coordinate: [f64; 3]) {
        match (&mut self.min, &mut self.max) {
            (Some(min), Some(max)) => {
                for axis in 0..3 {
                    min[axis] = min[axis].min(coordinate[axis]);
                    max[axis] = max[axis].max(coordinate[axis]);
                }
            }
            (None, None) => {
                self.min = Some(coordinate);
                self.max = Some(coordinate);
            }
            _ => unreachable!("extent accumulator stores min and max together"),
        }
    }

    fn include_bbox(&mut self, bbox: BBox) {
        self.include([bbox.min_x(), bbox.min_y(), bbox.min_z()]);
        self.include([bbox.max_x(), bbox.max_y(), bbox.max_z()]);
    }

    fn finish(self) -> Option<BBox> {
        match (self.min, self.max) {
            (Some(min), Some(max)) => {
                Some(BBox::new(min[0], min[1], min[2], max[0], max[1], max[2]))
            }
            (None, None) => None,
            _ => unreachable!("extent accumulator stores min and max together"),
        }
    }
}
