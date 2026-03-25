use std::collections::{HashMap, HashSet, VecDeque};

use cityjson::resources::handles::{SemanticHandle, TextureHandle};
use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{
    boundary::Boundary,
    geometry::{SemanticMapView, TextureMapView},
    CityModel, Geometry, GeometryType, Semantic, SemanticType, VertexRef,
};
use serde_json::{Map, Number, Value};

use crate::errors::{Error, Result};
use crate::ser::attributes::attributes_to_json_map;

pub(crate) fn semantics_to_json_value<VR, SS>(
    model: &CityModel<VR, SS>,
    geometry: &Geometry<VR, SS>,
) -> Result<Option<Value>>
where
    VR: VertexRef,
    SS: StringStorage,
{
    let Some(semantics) = geometry.semantics() else {
        return Ok(None);
    };

    let handles = collect_geometry_semantic_handles(model, geometry, semantics);
    if handles.is_empty() {
        return Ok(None);
    }

    let handle_to_local = handles
        .iter()
        .copied()
        .enumerate()
        .map(|(index, handle)| (handle, index))
        .collect::<HashMap<_, _>>();

    let surfaces = handles
        .iter()
        .map(|handle| {
            let semantic = model.get_semantic(*handle).ok_or_else(|| {
                Error::InvalidValue(format!("missing semantic for handle {handle}"))
            })?;
            semantic_to_json_value(semantic, &handle_to_local)
        })
        .collect::<Result<Vec<_>>>()?;

    let values = match geometry.type_geometry() {
        GeometryType::MultiPoint => {
            serialize_flat_semantics(semantics.points().iter(), &handle_to_local)
        }
        GeometryType::MultiLineString => {
            serialize_flat_semantics(semantics.linestrings().iter(), &handle_to_local)
        }
        GeometryType::MultiSurface | GeometryType::CompositeSurface => {
            serialize_flat_semantics(semantics.surfaces().iter(), &handle_to_local)
        }
        GeometryType::Solid | GeometryType::MultiSolid | GeometryType::CompositeSolid => {
            let boundary = geometry.boundaries().ok_or_else(|| {
                Error::InvalidValue(format!(
                    "geometry '{}' is missing boundaries",
                    geometry.type_geometry()
                ))
            })?;
            let assignments = semantics
                .surfaces()
                .iter()
                .map(|handle| {
                    handle
                        .as_ref()
                        .and_then(|handle| handle_to_local.get(handle).copied())
                })
                .collect::<Vec<_>>();
            serialize_surface_usize_options(boundary, geometry.type_geometry(), &assignments)
        }
        _ => {
            return Err(Error::InvalidValue(format!(
                "geometry semantics export is not supported for geometry type '{}'",
                geometry.type_geometry()
            )))
        }
    };

    Ok(Some(serde_json::json!({
        "surfaces": surfaces,
        "values": values,
    })))
}

pub(crate) fn materials_to_json_value<VR, SS>(
    model: &CityModel<VR, SS>,
    geometry: &Geometry<VR, SS>,
) -> Result<Option<Value>>
where
    VR: VertexRef,
    SS: StringStorage,
{
    let Some(materials) = geometry.materials() else {
        return Ok(None);
    };
    let boundary = geometry.boundaries().ok_or_else(|| {
        Error::InvalidValue(format!(
            "geometry '{}' is missing boundaries",
            geometry.type_geometry()
        ))
    })?;
    let dense_indices = model
        .iter_materials()
        .enumerate()
        .map(|(index, (handle, _))| (handle, index))
        .collect::<HashMap<_, _>>();

    let mut value = Map::new();
    for (theme, assignments) in materials.iter() {
        let surfaces = assignments
            .surfaces()
            .iter()
            .map(|material| (*material).and_then(|handle| dense_indices.get(&handle).copied()))
            .collect::<Vec<_>>();

        let theme_value = if is_uniform_non_null(&surfaces) {
            Value::Object(Map::from_iter([(
                "value".to_owned(),
                optional_index_to_json(surfaces.first().copied().flatten()),
            )]))
        } else {
            Value::Object(Map::from_iter([(
                "values".to_owned(),
                serialize_surface_usize_options(boundary, geometry.type_geometry(), &surfaces),
            )]))
        };
        value.insert(theme.as_ref().to_owned(), theme_value);
    }

    Ok(Some(Value::Object(value)))
}

pub(crate) fn textures_to_json_value<VR, SS>(
    model: &CityModel<VR, SS>,
    geometry: &Geometry<VR, SS>,
) -> Result<Option<Value>>
where
    VR: VertexRef,
    SS: StringStorage,
{
    let Some(textures) = geometry.textures() else {
        return Ok(None);
    };
    let boundary = geometry.boundaries().ok_or_else(|| {
        Error::InvalidValue(format!(
            "geometry '{}' is missing boundaries",
            geometry.type_geometry()
        ))
    })?;
    let dense_indices = model
        .iter_textures()
        .enumerate()
        .map(|(index, (handle, _))| (handle, index))
        .collect::<HashMap<_, _>>();

    let mut value = Map::new();
    for (theme, texture_map) in textures.iter() {
        value.insert(
            theme.as_ref().to_owned(),
            Value::Object(Map::from_iter([(
                "values".to_owned(),
                serialize_texture_values(
                    boundary,
                    geometry.type_geometry(),
                    texture_map,
                    &dense_indices,
                )?,
            )])),
        );
    }

    Ok(Some(Value::Object(value)))
}

fn collect_geometry_semantic_handles<VR, SS>(
    model: &CityModel<VR, SS>,
    geometry: &Geometry<VR, SS>,
    semantics: SemanticMapView<'_, VR>,
) -> Vec<SemanticHandle>
where
    VR: VertexRef,
    SS: StringStorage,
{
    let mut ordered = Vec::new();
    let mut seen = HashSet::new();
    let mut queue = VecDeque::new();

    let push_handle = |handle: SemanticHandle,
                       ordered: &mut Vec<SemanticHandle>,
                       seen: &mut HashSet<SemanticHandle>,
                       queue: &mut VecDeque<SemanticHandle>| {
        if seen.insert(handle) {
            ordered.push(handle);
            queue.push_back(handle);
        }
    };

    match geometry.type_geometry() {
        GeometryType::MultiPoint => {
            for handle in semantics.points().iter().flatten() {
                push_handle(*handle, &mut ordered, &mut seen, &mut queue);
            }
        }
        GeometryType::MultiLineString => {
            for handle in semantics.linestrings().iter().flatten() {
                push_handle(*handle, &mut ordered, &mut seen, &mut queue);
            }
        }
        _ => {
            for handle in semantics.surfaces().iter().flatten() {
                push_handle(*handle, &mut ordered, &mut seen, &mut queue);
            }
        }
    }

    while let Some(handle) = queue.pop_front() {
        let Some(semantic) = model.get_semantic(handle) else {
            continue;
        };
        if let Some(parent) = semantic.parent() {
            push_handle(parent, &mut ordered, &mut seen, &mut queue);
        }
        if let Some(children) = semantic.children() {
            for &child in children {
                push_handle(child, &mut ordered, &mut seen, &mut queue);
            }
        }
    }

    ordered
}

fn semantic_to_json_value<SS>(
    semantic: &Semantic<SS>,
    handle_to_local: &HashMap<SemanticHandle, usize>,
) -> Result<Value>
where
    SS: StringStorage,
{
    let mut value = Map::new();
    value.insert(
        "type".to_owned(),
        Value::String(semantic_type_to_str(semantic.type_semantic()).to_owned()),
    );

    if let Some(children) = semantic.children() {
        let local_children = children
            .iter()
            .filter_map(|handle| handle_to_local.get(handle).copied())
            .map(|index| Value::Number(Number::from(index)))
            .collect::<Vec<_>>();
        if !local_children.is_empty() {
            value.insert("children".to_owned(), Value::Array(local_children));
        }
    }

    if let Some(parent) = semantic.parent() {
        if let Some(index) = handle_to_local.get(&parent).copied() {
            value.insert("parent".to_owned(), Value::Number(Number::from(index)));
        }
    }

    if let Some(attributes) = semantic.attributes() {
        value.extend(attributes_to_json_map(attributes)?);
    }

    Ok(Value::Object(value))
}

fn serialize_flat_semantics<'a>(
    handles: impl IntoIterator<Item = &'a Option<SemanticHandle>>,
    handle_to_local: &HashMap<SemanticHandle, usize>,
) -> Value {
    Value::Array(
        handles
            .into_iter()
            .map(|handle| {
                optional_index_to_json(
                    handle
                        .as_ref()
                        .and_then(|handle| handle_to_local.get(handle).copied()),
                )
            })
            .collect(),
    )
}

fn serialize_surface_usize_options<VR>(
    boundary: &Boundary<VR>,
    geometry_type: &GeometryType,
    assignments: &[Option<usize>],
) -> Value
where
    VR: VertexRef,
{
    match geometry_type {
        GeometryType::MultiSurface | GeometryType::CompositeSurface => Value::Array(
            assignments
                .iter()
                .map(|value| optional_index_to_json(*value))
                .collect(),
        ),
        GeometryType::Solid => Value::Array(
            (0..boundary.shells().len())
                .map(|shell_index| {
                    let (start, end) = surface_range_for_shell(boundary, shell_index);
                    Value::Array(
                        assignments[start..end]
                            .iter()
                            .map(|value| optional_index_to_json(*value))
                            .collect(),
                    )
                })
                .collect(),
        ),
        GeometryType::MultiSolid | GeometryType::CompositeSolid => Value::Array(
            (0..boundary.solids().len())
                .map(|solid_index| {
                    let (shell_start, shell_end) = shell_range_for_solid(boundary, solid_index);
                    Value::Array(
                        (shell_start..shell_end)
                            .map(|shell_index| {
                                let (surface_start, surface_end) =
                                    surface_range_for_shell(boundary, shell_index);
                                Value::Array(
                                    assignments[surface_start..surface_end]
                                        .iter()
                                        .map(|value| optional_index_to_json(*value))
                                        .collect(),
                                )
                            })
                            .collect(),
                    )
                })
                .collect(),
        ),
        _ => Value::Array(
            assignments
                .iter()
                .map(|value| optional_index_to_json(*value))
                .collect(),
        ),
    }
}

fn serialize_texture_values<VR>(
    boundary: &Boundary<VR>,
    geometry_type: &GeometryType,
    texture_map: TextureMapView<'_, VR>,
    dense_indices: &HashMap<TextureHandle, usize>,
) -> Result<Value>
where
    VR: VertexRef,
{
    Ok(match geometry_type {
        GeometryType::MultiSurface | GeometryType::CompositeSurface => Value::Array(
            (0..boundary.surfaces().len())
                .map(|surface_index| {
                    let (ring_start, ring_end) = ring_range_for_surface(boundary, surface_index);
                    Value::Array(
                        (ring_start..ring_end)
                            .map(|ring_index| {
                                serialize_ring_texture_value(texture_map, ring_index, dense_indices)
                            })
                            .collect::<Result<Vec<_>>>()
                            .unwrap_or_default(),
                    )
                })
                .collect(),
        ),
        GeometryType::Solid => Value::Array(
            (0..boundary.shells().len())
                .map(|shell_index| {
                    let (surface_start, surface_end) =
                        surface_range_for_shell(boundary, shell_index);
                    Value::Array(
                        (surface_start..surface_end)
                            .map(|surface_index| {
                                let (ring_start, ring_end) =
                                    ring_range_for_surface(boundary, surface_index);
                                Ok(Value::Array(
                                    (ring_start..ring_end)
                                        .map(|ring_index| {
                                            serialize_ring_texture_value(
                                                texture_map,
                                                ring_index,
                                                dense_indices,
                                            )
                                        })
                                        .collect::<Result<Vec<_>>>()?,
                                ))
                            })
                            .collect::<Result<Vec<_>>>()
                            .unwrap_or_default(),
                    )
                })
                .collect(),
        ),
        GeometryType::MultiSolid | GeometryType::CompositeSolid => Value::Array(
            (0..boundary.solids().len())
                .map(|solid_index| {
                    let (shell_start, shell_end) = shell_range_for_solid(boundary, solid_index);
                    Value::Array(
                        (shell_start..shell_end)
                            .map(|shell_index| {
                                let (surface_start, surface_end) =
                                    surface_range_for_shell(boundary, shell_index);
                                Ok(Value::Array(
                                    (surface_start..surface_end)
                                        .map(|surface_index| {
                                            let (ring_start, ring_end) =
                                                ring_range_for_surface(boundary, surface_index);
                                            Ok(Value::Array(
                                                (ring_start..ring_end)
                                                    .map(|ring_index| {
                                                        serialize_ring_texture_value(
                                                            texture_map,
                                                            ring_index,
                                                            dense_indices,
                                                        )
                                                    })
                                                    .collect::<Result<Vec<_>>>()?,
                                            ))
                                        })
                                        .collect::<Result<Vec<_>>>()?,
                                ))
                            })
                            .collect::<Result<Vec<_>>>()
                            .unwrap_or_default(),
                    )
                })
                .collect(),
        ),
        _ => {
            return Err(Error::InvalidValue(format!(
                "geometry texture export is not supported for geometry type '{}'",
                geometry_type
            )))
        }
    })
}

fn serialize_ring_texture_value<VR>(
    texture_map: TextureMapView<'_, VR>,
    ring_index: usize,
    dense_indices: &HashMap<TextureHandle, usize>,
) -> Result<Value>
where
    VR: VertexRef,
{
    let texture = texture_map
        .ring_textures()
        .get(ring_index)
        .copied()
        .flatten()
        .and_then(|handle| dense_indices.get(&handle).copied());
    let Some(texture_index) = texture else {
        return Ok(Value::Array(vec![Value::Null]));
    };

    let vertex_start = texture_map
        .rings()
        .get(ring_index)
        .map(|value| value.to_usize())
        .unwrap_or(0);
    let vertex_end = texture_map
        .rings()
        .get(ring_index + 1)
        .map(|value| value.to_usize())
        .unwrap_or(texture_map.vertices().len());

    let mut values = Vec::with_capacity(vertex_end.saturating_sub(vertex_start) + 1);
    values.push(Value::Number(Number::from(texture_index)));
    for uv_index in &texture_map.vertices()[vertex_start..vertex_end] {
        values.push(optional_index_to_json(uv_index.map(|uv| uv.to_usize())));
    }
    Ok(Value::Array(values))
}

fn ring_range_for_surface<VR>(boundary: &Boundary<VR>, surface_index: usize) -> (usize, usize)
where
    VR: VertexRef,
{
    let start = boundary.surfaces()[surface_index].to_usize();
    let end = boundary
        .surfaces()
        .get(surface_index + 1)
        .map(|value| value.to_usize())
        .unwrap_or(boundary.rings().len());
    (start, end)
}

fn surface_range_for_shell<VR>(boundary: &Boundary<VR>, shell_index: usize) -> (usize, usize)
where
    VR: VertexRef,
{
    let start = boundary.shells()[shell_index].to_usize();
    let end = boundary
        .shells()
        .get(shell_index + 1)
        .map(|value| value.to_usize())
        .unwrap_or(boundary.surfaces().len());
    (start, end)
}

fn shell_range_for_solid<VR>(boundary: &Boundary<VR>, solid_index: usize) -> (usize, usize)
where
    VR: VertexRef,
{
    let start = boundary.solids()[solid_index].to_usize();
    let end = boundary
        .solids()
        .get(solid_index + 1)
        .map(|value| value.to_usize())
        .unwrap_or(boundary.shells().len());
    (start, end)
}

fn semantic_type_to_str<SS>(semantic_type: &SemanticType<SS>) -> &str
where
    SS: StringStorage,
{
    match semantic_type {
        SemanticType::RoofSurface => "RoofSurface",
        SemanticType::GroundSurface => "GroundSurface",
        SemanticType::WallSurface => "WallSurface",
        SemanticType::ClosureSurface => "ClosureSurface",
        SemanticType::OuterCeilingSurface => "OuterCeilingSurface",
        SemanticType::OuterFloorSurface => "OuterFloorSurface",
        SemanticType::Window => "Window",
        SemanticType::Door => "Door",
        SemanticType::InteriorWallSurface => "InteriorWallSurface",
        SemanticType::CeilingSurface => "CeilingSurface",
        SemanticType::FloorSurface => "FloorSurface",
        SemanticType::WaterSurface => "WaterSurface",
        SemanticType::WaterGroundSurface => "WaterGroundSurface",
        SemanticType::WaterClosureSurface => "WaterClosureSurface",
        SemanticType::TrafficArea => "TrafficArea",
        SemanticType::AuxiliaryTrafficArea => "AuxiliaryTrafficArea",
        SemanticType::TransportationMarking => "TransportationMarking",
        SemanticType::TransportationHole => "TransportationHole",
        SemanticType::Extension(value) => value.as_ref(),
        SemanticType::Default => "Default",
        _ => "Default",
    }
}

fn optional_index_to_json(value: Option<usize>) -> Value {
    value
        .map(|value| Value::Number(Number::from(value)))
        .unwrap_or(Value::Null)
}

fn is_uniform_non_null(values: &[Option<usize>]) -> bool {
    let Some(first) = values.first().copied().flatten() else {
        return false;
    };
    values.iter().all(|value| *value == Some(first))
}
