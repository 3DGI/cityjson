use std::collections::BTreeSet;

use serde_json::{Map, Value};

use crate::{CityModel, Error, Result, json};

fn unsupported(message: impl Into<String>) -> Error {
    Error::UnsupportedFeature(message.into())
}

fn import_error(message: impl Into<String>) -> Error {
    Error::Import(message.into())
}

fn serialize_root(model: &CityModel) -> Result<Map<String, Value>> {
    match serde_json::from_slice(&json::to_vec(model)?)? {
        Value::Object(root) => Ok(root),
        _ => Err(import_error("serialized CityJSON root is not an object")),
    }
}

fn parse_root(root: Map<String, Value>) -> Result<CityModel> {
    let bytes = serde_json::to_vec(&Value::Object(root))?;
    match json::probe(&bytes)?.kind() {
        json::RootKind::CityJSON => json::from_slice(&bytes),
        json::RootKind::CityJSONFeature => json::from_feature_slice(&bytes),
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

fn same_transform(target: &Map<String, Value>, source: &Map<String, Value>) -> bool {
    match source.get("transform") {
        None => true,
        Some(source_transform) => target.get("transform") == Some(source_transform),
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

/// Roundtrip a model through JSON serialization and parsing.
///
/// This keeps the semantic model authoritative in Rust and is the cleanup path
/// exposed to the FFI layer for normalization-sensitive workflows.
pub fn cleanup(model: &CityModel) -> Result<CityModel> {
    let bytes = match model.type_citymodel() {
        cityjson::CityModelType::CityJSON => json::to_vec(model)?,
        cityjson::CityModelType::CityJSONFeature => json::to_feature_vec_with_options(
            model,
            json::WriteOptions {
                pretty: false,
                validate_default_themes: true,
            },
        )?,
        other => return Err(Error::UnsupportedType(other.to_string())),
    };

    match model.type_citymodel() {
        cityjson::CityModelType::CityJSON => json::from_slice(&bytes),
        cityjson::CityModelType::CityJSONFeature => json::from_feature_slice(&bytes),
        other => Err(Error::UnsupportedType(other.to_string())),
    }
}

/// Extract a submodel by selected CityObject identifiers.
///
/// The extracted model keeps the source document's shared root state so it
/// remains self-contained, then prunes parent/child links that point outside
/// the selected object set.
pub fn extract<'a, I>(model: &CityModel, cityobject_ids: I) -> Result<CityModel>
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

/// Append one model into another.
///
/// This path intentionally uses serialized CityJSON as the interchange form so
/// the append behavior stays aligned with the Rust serializer and parser.
/// The current implementation is conservative: it supports appending models
/// that do not carry appearance resources or geometry templates, and it
/// requires matching root transforms.
pub fn append(target: &mut CityModel, source: &CityModel) -> Result<()> {
    let mut target_root = serialize_root(target)?;
    let source_root = serialize_root(source)?;

    if !same_transform(&target_root, &source_root) {
        return Err(unsupported(
            "model append currently requires identical transform objects",
        ));
    }

    if !append_kind_compatible(root_kind(&target_root)?, root_kind(&source_root)?) {
        return Err(import_error(
            "model append currently requires both inputs to have the same root type",
        ));
    }

    let vertex_offset = get_array(&target_root, "vertices").map_or(0_u64, |vertices| {
        u64::try_from(vertices.len()).unwrap_or(u64::MAX)
    });

    let source_vertices = get_array(&source_root, "vertices")
        .cloned()
        .ok_or_else(|| import_error("source model is missing its vertices array"))?;
    let target_vertices = get_array_mut(&mut target_root, "vertices")
        .ok_or_else(|| import_error("target model is missing its vertices array"))?;
    target_vertices.extend(source_vertices);

    merge_root_object_field(&mut target_root, &source_root, "extensions")?;

    let source_cityobjects = get_object(&source_root, "CityObjects")
        .ok_or_else(|| import_error("source model is missing its CityObjects map"))?;
    let target_cityobjects = get_object_mut(&mut target_root, "CityObjects")
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

    *target = parse_root(target_root)?;
    Ok(())
}

/// Merge a sequence of models into one accumulator.
pub fn merge<I>(models: I) -> Result<CityModel>
where
    I: IntoIterator<Item = CityModel>,
{
    let mut models = models.into_iter();
    let Some(mut merged) = models.next() else {
        return Err(import_error("merge requires at least one model"));
    };

    for model in models {
        append(&mut merged, &model)?;
    }

    Ok(merged)
}
