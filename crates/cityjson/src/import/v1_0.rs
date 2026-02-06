//! CityJSON v1.0 to v2.0 conversion.
//!
//! This module handles conversion of legacy v1.0 CityJSON documents to the current v2.0 format.
//!
//! ## Schema Differences (v1.0 → v2.0)
//!
//! - `GenericCityObject` doesn't exist in v2.0, mapped to Extension
//! - Metadata structure may differ
//! - Some semantic types have been renamed or extended

use crate::error::{Error, Result};
use crate::prelude::*;
use crate::v2_0::{CityModel, CityObject, CityObjectType};
use crate::CityModelType;

/// Converts a CityJSON v1.0 document to v2.0.
///
/// # Arguments
///
/// * `json_str` - A JSON string containing a v1.0 CityJSON document
///
/// # Returns
///
/// A v2.0 CityModel with appropriate type conversions applied
pub fn convert_to_v2<SS: StringStorage>(json_str: &str) -> Result<CityModel<u32, SS>>
where
    SS::String: From<String>,
{
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| Error::InvalidJson(e.to_string()))?;

    convert_from_value::<SS>(&value)
}

/// Converts from a parsed JSON value.
pub(crate) fn convert_from_value<SS: StringStorage>(
    value: &serde_json::Value,
) -> Result<CityModel<u32, SS>>
where
    SS::String: From<String>,
{
    let mut model = CityModel::new(CityModelType::CityJSON);

    // Convert vertices
    if let Some(vertices_value) = value.get("vertices") {
        convert_vertices(&mut model, vertices_value)?;
    }

    // Convert CityObjects
    if let Some(objects_value) = value.get("CityObjects") {
        convert_city_objects(&mut model, objects_value)?;
    }

    // Convert metadata if present
    if let Some(metadata_value) = value.get("metadata") {
        convert_metadata(&mut model, metadata_value)?;
    }

    // Convert transform if present
    if let Some(transform_value) = value.get("transform") {
        convert_transform(&mut model, transform_value)?;
    }

    // Convert extensions if present
    if let Some(extensions_value) = value.get("extensions") {
        convert_extensions(&mut model, extensions_value)?;
    }

    Ok(model)
}

fn convert_vertices<VR: VertexRef, SS: StringStorage>(
    model: &mut CityModel<VR, SS>,
    value: &serde_json::Value,
) -> Result<()>
where
    SS::String: From<String>,
{
    let vertices = value.as_array().ok_or_else(|| {
        Error::Import("'vertices' must be an array".to_string())
    })?;

    for vertex in vertices {
        let coords = vertex.as_array().ok_or_else(|| {
            Error::Import("Each vertex must be an array".to_string())
        })?;

        if coords.len() >= 3 {
            let x = coords[0].as_i64().unwrap_or(0);
            let y = coords[1].as_i64().unwrap_or(0);
            let z = coords[2].as_i64().unwrap_or(0);
            let coord = QuantizedCoordinate::new(x, y, z);
            model.add_vertex(coord)?;
        }
    }

    Ok(())
}

fn convert_city_objects<VR: VertexRef, SS: StringStorage>(
    model: &mut CityModel<VR, SS>,
    value: &serde_json::Value,
) -> Result<()>
where
    SS::String: From<String>,
{
    let objects = value.as_object().ok_or_else(|| {
        Error::Import("'CityObjects' must be an object".to_string())
    })?;

    for (id, obj_value) in objects {
        let city_object = convert_city_object::<SS>(id.clone().into(), obj_value)?;
        model.cityobjects_mut().add(city_object);
    }

    Ok(())
}

fn convert_city_object<SS: StringStorage>(
    id: SS::String,
    value: &serde_json::Value,
) -> Result<CityObject<SS>>
where
    SS::String: From<String>,
{
    let type_str = value
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::Import("CityObject missing 'type'".to_string()))?;

    let co_type = convert_city_object_type::<SS>(type_str);
    let mut city_object = CityObject::new(crate::v2_0::CityObjectIdentifier::new(id), co_type);

    // Convert attributes if present
    if let Some(attrs_value) = value.get("attributes") {
        convert_attributes(&mut city_object, attrs_value)?;
    }

    // Note: Geometry conversion is not yet fully implemented
    // TODO: Convert geometry references

    Ok(city_object)
}

/// Maps v1.0 CityObject types to v2.0.
///
/// Notable changes:
/// - `GenericCityObject` doesn't exist in v2.0, map to Extension
fn convert_city_object_type<SS: StringStorage>(type_str: &str) -> CityObjectType<SS>
where
    SS::String: From<String>,
{
    match type_str {
        // Standard types that exist in both versions
        "Building" => CityObjectType::Building,
        "BuildingPart" => CityObjectType::BuildingPart,
        "BuildingInstallation" => CityObjectType::BuildingInstallation,
        "Bridge" => CityObjectType::Bridge,
        "BridgePart" => CityObjectType::BridgePart,
        "BridgeInstallation" => CityObjectType::BridgeInstallation,
        "BridgeConstructionElement" => CityObjectType::BridgeConstructiveElement,
        "CityObjectGroup" => CityObjectType::CityObjectGroup,
        "CityFurniture" => CityObjectType::CityFurniture,
        "LandUse" => CityObjectType::LandUse,
        "PlantCover" => CityObjectType::PlantCover,
        "Railway" => CityObjectType::Railway,
        "Road" => CityObjectType::Road,
        "SolitaryVegetationObject" => CityObjectType::SolitaryVegetationObject,
        "TINRelief" => CityObjectType::TINRelief,
        "TransportSquare" => CityObjectType::TransportSquare,
        "Tunnel" => CityObjectType::Tunnel,
        "TunnelPart" => CityObjectType::TunnelPart,
        "TunnelInstallation" => CityObjectType::TunnelInstallation,
        "WaterBody" => CityObjectType::WaterBody,

        // v1.0 types that don't exist in v2.0
        "GenericCityObject" => CityObjectType::Extension("+GenericCityObject".to_string().into()),

        // Extension types (start with +)
        other if other.starts_with('+') => CityObjectType::Extension(other.to_string().into()),

        // Unknown types become extensions
        other => CityObjectType::Extension(format!("+{}", other).into()),
    }
}

fn convert_attributes<SS: StringStorage>(
    city_object: &mut CityObject<SS>,
    value: &serde_json::Value,
) -> Result<()>
where
    SS::String: From<String>,
{
    let attrs_obj = value.as_object().ok_or_else(|| {
        Error::Import("'attributes' must be an object".to_string())
    })?;

    let attributes = city_object.attributes_mut();

    for (key, attr_value) in attrs_obj {
        let converted = convert_attribute_value::<SS>(attr_value);
        attributes.insert(key.clone().into(), converted);
    }

    Ok(())
}

fn convert_attribute_value<SS: StringStorage>(value: &serde_json::Value) -> AttributeValue<SS>
where
    SS::String: From<String>,
{
    match value {
        serde_json::Value::Null => AttributeValue::Null,
        serde_json::Value::Bool(b) => AttributeValue::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                AttributeValue::Integer(i)
            } else if let Some(u) = n.as_u64() {
                AttributeValue::Unsigned(u)
            } else if let Some(f) = n.as_f64() {
                AttributeValue::Float(f)
            } else {
                AttributeValue::Null
            }
        }
        serde_json::Value::String(s) => AttributeValue::String(s.clone().into()),
        serde_json::Value::Array(arr) => {
            let converted: Vec<AttributeValue<SS>> = arr
                .iter()
                .map(|v| convert_attribute_value::<SS>(v))
                .collect();
            AttributeValue::Vec(converted.iter().map(|v| Box::new(v.clone())).collect())
        }
        serde_json::Value::Object(obj) => {
            let mut map = std::collections::HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone().into(), Box::new(convert_attribute_value::<SS>(v)));
            }
            AttributeValue::Map(map)
        }
    }
}

fn convert_metadata<VR: VertexRef, SS: StringStorage>(
    model: &mut CityModel<VR, SS>,
    value: &serde_json::Value,
) -> Result<()>
where
    SS::String: From<String>,
{
    let metadata = model.metadata_mut();

    // Map v1.0 metadata fields to v2.0
    if let Some(identifier) = value.get("identifier").and_then(|v| v.as_str()) {
        metadata.set_identifier(CityModelIdentifier::new(identifier.to_string().into()));
    }

    if let Some(crs) = value.get("referenceSystem").and_then(|v| v.as_str()) {
        metadata.set_reference_system(CRS::new(crs.to_string().into()));
    }

    // Geographical extent
    if let Some(extent) = value.get("geographicalExtent").and_then(|v| v.as_array())
        && extent.len() >= 6
    {
        let bbox = BBox::new(
            extent[0].as_f64().unwrap_or(0.0),
            extent[1].as_f64().unwrap_or(0.0),
            extent[2].as_f64().unwrap_or(0.0),
            extent[3].as_f64().unwrap_or(0.0),
            extent[4].as_f64().unwrap_or(0.0),
            extent[5].as_f64().unwrap_or(0.0),
        );
        metadata.set_geographical_extent(bbox);
    }

    Ok(())
}

fn convert_transform<VR: VertexRef, SS: StringStorage>(
    model: &mut CityModel<VR, SS>,
    value: &serde_json::Value,
) -> Result<()>
where
    SS::String: From<String>,
{
    let transform = model.transform_mut();

    if let Some(scale) = value.get("scale").and_then(|v| v.as_array())
        && scale.len() >= 3
    {
        transform.set_scale([
            scale[0].as_f64().unwrap_or(1.0),
            scale[1].as_f64().unwrap_or(1.0),
            scale[2].as_f64().unwrap_or(1.0),
        ]);
    }

    if let Some(translate) = value.get("translate").and_then(|v| v.as_array())
        && translate.len() >= 3
    {
        transform.set_translate([
            translate[0].as_f64().unwrap_or(0.0),
            translate[1].as_f64().unwrap_or(0.0),
            translate[2].as_f64().unwrap_or(0.0),
        ]);
    }

    Ok(())
}

fn convert_extensions<VR: VertexRef, SS: StringStorage>(
    model: &mut CityModel<VR, SS>,
    value: &serde_json::Value,
) -> Result<()>
where
    SS::String: From<String>,
{
    let extensions = value.as_object().ok_or_else(|| {
        Error::Import("'extensions' must be an object".to_string())
    })?;

    for (name, ext_value) in extensions {
        let url = ext_value
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let version = ext_value
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0");

        let extension = crate::v2_0::Extension::new(
            name.clone().into(),
            url.to_string().into(),
            version.to_string().into(),
        );
        model.extensions_mut().add(extension);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_city_object_type_building() {
        let result: CityObjectType<OwnedStringStorage> = convert_city_object_type("Building");
        assert_eq!(result, CityObjectType::Building);
    }

    #[test]
    fn test_convert_city_object_type_generic() {
        let result: CityObjectType<OwnedStringStorage> = convert_city_object_type("GenericCityObject");
        assert!(matches!(result, CityObjectType::Extension(_)));
    }

    #[test]
    fn test_convert_city_object_type_extension() {
        let result: CityObjectType<OwnedStringStorage> = convert_city_object_type("+Custom");
        assert!(matches!(result, CityObjectType::Extension(_)));
    }
}
