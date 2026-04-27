use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::io::Cursor;

use cityjson::CityModelType;
use cityjson::v2_0::OwnedCityModel;
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::errors::{Error, Result};
use crate::v2_0::{
    ReadOptions, WriteOptions, read_feature, read_feature_stream, read_model, to_vec,
};

pub mod staged {
    use std::io::Write;
    use std::path::Path;

    use cityjson::CityModelType;
    use cityjson::v2_0::OwnedCityModel;
    use serde_json::value::RawValue;
    use serde_json::{Map, Value};

    use crate::errors::{Error, Result};
    use crate::v2_0::{ReadOptions, WriteOptions, read_feature_with_base, read_model, write_model};

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

    /// # Errors
    ///
    /// Returns an error if JSON parsing fails or the input is not a valid `CityJSONFeature`.
    pub fn from_feature_slice_with_base(
        feature_bytes: &[u8],
        base_document_bytes: &[u8],
    ) -> Result<OwnedCityModel> {
        let base = read_model(base_document_bytes, &ReadOptions::default())?;
        read_feature_with_base(feature_bytes, &base, &ReadOptions::default())
    }

    /// # Errors
    ///
    /// Returns an error if JSON parsing fails or the input is not a valid `CityJSONFeature`.
    pub fn from_feature_slice_with_base_assume_cityjson_feature_v2_0(
        feature_bytes: &[u8],
        base_document_bytes: &[u8],
    ) -> Result<OwnedCityModel> {
        from_feature_slice_with_base(feature_bytes, base_document_bytes)
    }

    /// # Errors
    ///
    /// Returns an error if JSON serialization or parsing fails, or the assembly is not valid.
    pub fn from_feature_assembly_with_base(
        assembly: FeatureAssembly<'_>,
        base_document_bytes: &[u8],
    ) -> Result<OwnedCityModel> {
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

    /// # Errors
    ///
    /// Returns an error if the file cannot be read or the content is not a valid `CityJSONFeature`.
    pub fn from_feature_file_with_base<P: AsRef<Path>>(
        path: P,
        base_document_bytes: &[u8],
    ) -> Result<OwnedCityModel> {
        let bytes =
            std::fs::read(path).map_err(|error| Error::Json(serde_json::Error::io(error)))?;
        from_feature_slice_with_base(&bytes, base_document_bytes)
    }

    /// # Errors
    ///
    /// Returns an error if the model is not a `CityJSONFeature` or if serialization fails.
    pub fn to_feature_writer(writer: &mut impl Write, model: &OwnedCityModel) -> Result<()> {
        match model.type_citymodel() {
            CityModelType::CityJSONFeature => write_model(writer, model, &WriteOptions::default()),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Probe {
    kind: RootKind,
    version: Option<String>,
}

impl Probe {
    #[must_use]
    pub fn kind(&self) -> RootKind {
        self.kind
    }

    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
}

#[derive(Debug, Deserialize)]
struct Header {
    #[serde(rename = "type")]
    kind: String,
    version: Option<String>,
}

/// # Errors
///
/// Returns an error if JSON parsing fails or the root type is not recognized.
pub fn probe(bytes: &[u8]) -> Result<Probe> {
    let header: Header = serde_json::from_slice(bytes)?;
    let kind = match header.kind.as_str() {
        "CityJSON" => RootKind::CityJSON,
        "CityJSONFeature" => RootKind::CityJSONFeature,
        other => return Err(Error::UnsupportedType(other.to_owned())),
    };

    Ok(Probe {
        kind,
        version: header.version,
    })
}

fn import_error(message: impl Into<String>) -> Error {
    Error::InvalidValue(message.into())
}

fn serialize_root(model: &OwnedCityModel) -> Result<Map<String, Value>> {
    match serde_json::from_slice(&to_vec(model, &WriteOptions::default())?)? {
        Value::Object(root) => Ok(root),
        _ => Err(import_error("serialized CityJSON root is not an object")),
    }
}

fn parse_root(root: Map<String, Value>) -> Result<OwnedCityModel> {
    let bytes = serde_json::to_vec(&Value::Object(root))?;
    match probe(&bytes)?.kind() {
        RootKind::CityJSON => read_model(&bytes, &ReadOptions::default()),
        RootKind::CityJSONFeature => read_feature(&bytes, &ReadOptions::default()),
    }
}

fn root_kind(root: &Map<String, Value>) -> Result<&str> {
    root.get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| import_error("CityJSON root is missing its type"))
}

fn get_object<'a>(root: &'a Map<String, Value>, key: &str) -> Option<&'a Map<String, Value>> {
    root.get(key).and_then(Value::as_object)
}

fn get_object_mut<'a>(
    root: &'a mut Map<String, Value>,
    key: &str,
) -> Option<&'a mut Map<String, Value>> {
    root.get_mut(key).and_then(Value::as_object_mut)
}

fn get_array<'a>(root: &'a Map<String, Value>, key: &str) -> Option<&'a Vec<Value>> {
    root.get(key).and_then(Value::as_array)
}

fn get_array_mut<'a>(root: &'a mut Map<String, Value>, key: &str) -> Option<&'a mut Vec<Value>> {
    root.get_mut(key).and_then(Value::as_array_mut)
}

#[derive(Debug, Clone, PartialEq)]
enum TransformMergeState {
    Empty,
    Present(Value),
    Cleared,
}

impl TransformMergeState {
    fn from_root(root: &Map<String, Value>) -> Self {
        match root.get("transform") {
            Some(transform) => Self::Present(transform.clone()),
            None => Self::Empty,
        }
    }
}

fn reconcile_transform_state(
    current: TransformMergeState,
    source: Option<&Value>,
) -> TransformMergeState {
    match (current, source) {
        (TransformMergeState::Empty, None) => TransformMergeState::Empty,
        (TransformMergeState::Empty, Some(transform)) => {
            TransformMergeState::Present(transform.clone())
        }
        (TransformMergeState::Present(transform), None) => TransformMergeState::Present(transform),
        (TransformMergeState::Present(transform), Some(source_transform))
            if transform == *source_transform =>
        {
            TransformMergeState::Present(transform)
        }
        (TransformMergeState::Cleared, _) | (TransformMergeState::Present(_), Some(_)) => {
            TransformMergeState::Cleared
        }
    }
}

fn apply_transform_state(root: &mut Map<String, Value>, state: &TransformMergeState) {
    match state {
        TransformMergeState::Empty | TransformMergeState::Cleared => {
            root.remove("transform");
        }
        TransformMergeState::Present(transform) => {
            root.insert("transform".to_string(), transform.clone());
        }
    }
}

fn append_kind_compatible(target_kind: &str, source_kind: &str) -> bool {
    target_kind == source_kind || (target_kind == "CityJSON" && source_kind == "CityJSONFeature")
}

fn merge_root_object_field(
    target: &mut Map<String, Value>,
    source: &Map<String, Value>,
    key: &str,
) -> Result<()> {
    let Some(source_map) = get_object(source, key) else {
        return Ok(());
    };

    let target_value = target
        .entry(key.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let target_map = target_value
        .as_object_mut()
        .ok_or_else(|| import_error(format!("target '{key}' field is not an object")))?;

    for (entry_key, entry_value) in source_map {
        match target_map.get(entry_key) {
            Some(existing) if existing != entry_value => {
                return Err(import_error(format!(
                    "conflicting '{key}' entry for '{entry_key}' during append"
                )));
            }
            Some(_) => {}
            None => {
                target_map.insert(entry_key.clone(), entry_value.clone());
            }
        }
    }

    Ok(())
}

fn remap_index_value(value: &mut Value, offset: u64) -> Result<()> {
    match value {
        Value::Number(number) => {
            let index = number
                .as_u64()
                .ok_or_else(|| import_error("expected non-negative integer index"))?;
            *value = Value::from(index + offset);
            Ok(())
        }
        Value::Array(items) => {
            for item in items {
                remap_index_value(item, offset)?;
            }
            Ok(())
        }
        Value::Null => Ok(()),
        _ => Err(import_error("expected an index array")),
    }
}

fn remap_geometry_boundaries(geometry: &mut Map<String, Value>, vertex_offset: u64) -> Result<()> {
    if let Some(boundaries) = geometry.get_mut("boundaries") {
        remap_index_value(boundaries, vertex_offset)?;
    }

    Ok(())
}

fn prune_relations(cityobject: &mut Map<String, Value>, selected: &BTreeSet<String>, key: &str) {
    let Some(values) = cityobject.get_mut(key).and_then(Value::as_array_mut) else {
        return;
    };

    values.retain(|value| value.as_str().is_some_and(|id| selected.contains(id)));
    if values.is_empty() {
        cityobject.remove(key);
    }
}

/// # Errors
///
/// Returns an error if serialization or re-parsing of the model fails, or the type is unsupported.
pub fn cleanup(model: &OwnedCityModel) -> Result<OwnedCityModel> {
    let options = WriteOptions {
        validate_default_themes: matches!(model.type_citymodel(), CityModelType::CityJSONFeature),
        ..WriteOptions::default()
    };
    let bytes = to_vec(model, &options)?;

    match model.type_citymodel() {
        CityModelType::CityJSON => read_model(&bytes, &ReadOptions::default()),
        CityModelType::CityJSONFeature => read_feature(&bytes, &ReadOptions::default()),
        other => Err(Error::UnsupportedType(other.to_string())),
    }
}

/// # Errors
///
/// Returns an error if serialization fails, the id set is empty, or no `CityObjects` match.
pub fn extract<'a, I>(model: &OwnedCityModel, cityobject_ids: I) -> Result<OwnedCityModel>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut root = serialize_root(model)?;
    let selected = cityobject_ids
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();

    if selected.is_empty() {
        return Err(import_error(
            "extract requires at least one CityObject identifier",
        ));
    }

    let cityobjects = get_object_mut(&mut root, "CityObjects")
        .ok_or_else(|| import_error("CityJSON root is missing its CityObjects map"))?;
    cityobjects.retain(|id, _| selected.contains(id));

    if cityobjects.is_empty() {
        return Err(import_error("extract selection matched no CityObjects"));
    }

    for cityobject in cityobjects.values_mut() {
        let Some(cityobject) = cityobject.as_object_mut() else {
            return Err(import_error("CityObject entry is not an object"));
        };

        prune_relations(cityobject, &selected, "children");
        prune_relations(cityobject, &selected, "parents");
    }

    parse_root(root)
}

fn merge_one(
    target_root: &mut Map<String, Value>,
    source_root: &Map<String, Value>,
    transform_state: &mut TransformMergeState,
) -> Result<()> {
    if !append_kind_compatible(root_kind(target_root)?, root_kind(source_root)?) {
        return Err(import_error(
            "model append currently requires both inputs to have the same root type",
        ));
    }

    *transform_state =
        reconcile_transform_state(transform_state.clone(), source_root.get("transform"));

    let vertex_offset = get_array(target_root, "vertices").map_or(0_u64, |vertices| {
        u64::try_from(vertices.len()).unwrap_or(u64::MAX)
    });

    let source_vertices = get_array(source_root, "vertices")
        .cloned()
        .ok_or_else(|| import_error("source model is missing its vertices array"))?;
    let target_vertices = get_array_mut(target_root, "vertices")
        .ok_or_else(|| import_error("target model is missing its vertices array"))?;
    target_vertices.extend(source_vertices);

    merge_root_object_field(target_root, source_root, "extensions")?;

    let source_cityobjects = get_object(source_root, "CityObjects")
        .ok_or_else(|| import_error("source model is missing its CityObjects map"))?;
    let target_cityobjects = get_object_mut(target_root, "CityObjects")
        .ok_or_else(|| import_error("target model is missing its CityObjects map"))?;

    for (id, cityobject_value) in source_cityobjects {
        if target_cityobjects.contains_key(id) {
            return Err(import_error(format!(
                "duplicate CityObject id during append: {id}"
            )));
        }

        let mut cityobject = cityobject_value
            .as_object()
            .ok_or_else(|| import_error("source CityObject entry is not an object"))?
            .clone();

        if let Some(geometries) = cityobject.get_mut("geometry").and_then(Value::as_array_mut) {
            for geometry in geometries {
                let geometry = geometry
                    .as_object_mut()
                    .ok_or_else(|| import_error("geometry entry is not an object"))?;
                remap_geometry_boundaries(geometry, vertex_offset)?;
            }
        }

        target_cityobjects.insert(id.clone(), Value::Object(cityobject));
    }

    Ok(())
}

/// # Errors
///
/// Returns an error if JSON serialization or parsing fails, the root types are incompatible,
/// ids conflict, or the append cannot be applied.
pub fn append(target: &mut OwnedCityModel, source: &OwnedCityModel) -> Result<()> {
    let mut target_root = serialize_root(target)?;
    let source_root = serialize_root(source)?;
    let mut transform_state = TransformMergeState::from_root(&target_root);

    merge_one(&mut target_root, &source_root, &mut transform_state)?;
    apply_transform_state(&mut target_root, &transform_state);

    *target = parse_root(target_root)?;
    Ok(())
}

/// # Errors
///
/// Returns an error if the iterator is empty or if any merge step fails.
pub fn merge<I>(models: I) -> Result<OwnedCityModel>
where
    I: IntoIterator<Item = OwnedCityModel>,
{
    let mut models = models.into_iter();
    let Some(first) = models.next() else {
        return Err(import_error("merge requires at least one model"));
    };

    let mut merged_root = serialize_root(&first)?;
    let mut transform_state = TransformMergeState::from_root(&merged_root);

    for model in models {
        let source_root = serialize_root(&model)?;
        merge_one(&mut merged_root, &source_root, &mut transform_state)?;
    }

    apply_transform_state(&mut merged_root, &transform_state);
    parse_root(merged_root)
}

/// # Errors
///
/// Returns an error if the stream is empty, items are not JSON objects, or merging fails.
pub fn merge_feature_stream_slice(bytes: &[u8]) -> Result<OwnedCityModel> {
    let mut stream = serde_json::Deserializer::from_slice(bytes).into_iter::<Value>();
    let Some(first) = stream.next().transpose()? else {
        return Err(import_error("empty feature stream"));
    };
    let Value::Object(first) = first else {
        return Err(import_error("stream items must be JSON objects"));
    };
    let first_bytes = serde_json::to_vec(&Value::Object(first.clone()))?;

    if matches!(probe(&first_bytes)?.kind(), RootKind::CityJSON) {
        let reader = Cursor::new(bytes);
        let mut merged = read_model(&first_bytes, &ReadOptions::default())?;
        for feature in read_feature_stream(reader, &ReadOptions::default())? {
            append(&mut merged, &feature?)?;
        }
        return Ok(merged);
    }

    let mut models = vec![read_feature(&first_bytes, &ReadOptions::default())?];
    for item in stream {
        let Value::Object(item) = item? else {
            return Err(import_error("stream items must be JSON objects"));
        };
        let item_bytes = serde_json::to_vec(&Value::Object(item))?;
        models.push(read_feature(&item_bytes, &ReadOptions::default())?);
    }
    merge(models)
}

/// # Errors
///
/// Returns an error if the input is not a valid `CityJSONSeq` stream or merging fails.
pub fn merge_cityjsonseq_slice(bytes: &[u8]) -> Result<OwnedCityModel> {
    merge_feature_stream_slice(bytes)
}
