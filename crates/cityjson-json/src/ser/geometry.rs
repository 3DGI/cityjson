use cityjson::resources::handles::GeometryHandle;
use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{CityModel, Geometry, GeometryType, VertexRef};
use serde_json::{Map, Value};

use crate::errors::{Error, Result};

pub(crate) fn geometries_to_json_value<VR, SS>(
    model: &CityModel<VR, SS>,
    geometry_handles: &[GeometryHandle],
) -> Result<Value>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    let mut geometries = Vec::with_capacity(geometry_handles.len());
    for handle in geometry_handles {
        let geometry = model
            .get_geometry(*handle)
            .ok_or_else(|| Error::InvalidValue(format!("missing geometry for handle {handle}")))?;
        geometries.push(geometry_to_json_value(geometry)?);
    }
    Ok(Value::Array(geometries))
}

fn geometry_to_json_value<VR, SS>(geometry: &Geometry<VR, SS>) -> Result<Value>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    if geometry.instance().is_some() {
        return Err(Error::UnsupportedFeature(
            "geometry template serialization is not implemented yet",
        ));
    }
    if geometry.semantics().is_some() {
        return Err(Error::UnsupportedFeature(
            "geometry semantics serialization is not implemented yet",
        ));
    }
    if geometry.materials().is_some() {
        return Err(Error::UnsupportedFeature(
            "geometry material serialization is not implemented yet",
        ));
    }
    if geometry.textures().is_some() {
        return Err(Error::UnsupportedFeature(
            "geometry texture serialization is not implemented yet",
        ));
    }

    let boundaries = geometry.boundaries().ok_or_else(|| {
        Error::InvalidValue(format!(
            "geometry '{}' is missing boundaries",
            geometry.type_geometry()
        ))
    })?;

    let mut value = Map::new();
    value.insert(
        "type".to_owned(),
        Value::String(geometry.type_geometry().to_string()),
    );
    if let Some(lod) = geometry.lod() {
        value.insert("lod".to_owned(), Value::String(lod.to_string()));
    }
    value.insert(
        "boundaries".to_owned(),
        match geometry.type_geometry() {
            GeometryType::MultiPoint => serde_json::to_value(boundaries.to_nested_multi_point()?)?,
            GeometryType::MultiLineString => {
                serde_json::to_value(boundaries.to_nested_multi_linestring()?)?
            }
            GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                serde_json::to_value(boundaries.to_nested_multi_or_composite_surface()?)?
            }
            GeometryType::Solid => serde_json::to_value(boundaries.to_nested_solid()?)?,
            GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                serde_json::to_value(boundaries.to_nested_multi_or_composite_solid()?)?
            }
            GeometryType::GeometryInstance => {
                return Err(Error::UnsupportedFeature(
                    "geometry template serialization is not implemented yet",
                ));
            }
            _ => {
                return Err(Error::InvalidValue(format!(
                    "unsupported geometry type '{}'",
                    geometry.type_geometry()
                )));
            }
        },
    );

    Ok(Value::Object(value))
}
