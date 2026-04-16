#![allow(clippy::wildcard_imports)]

use super::*;

pub(super) fn geometry_tables(
    relational: &ModelRelationalView<'_>,
) -> Result<ExportedGeometryTables> {
    let mut exported = ExportedGeometryTables::default();
    let context = GeometryExportContext { relational };

    for (cityobject_ix, (_, object)) in relational.cityobjects().iter().enumerate() {
        if let Some(geometries) = object.geometry() {
            for (ordinal, geometry_handle) in geometries.iter().enumerate() {
                append_geometry_tables(
                    &context,
                    u64::try_from(cityobject_ix).expect("cityobject index fits into u64"),
                    *geometry_handle,
                    ordinal,
                    &mut exported,
                )?;
            }
        }
    }

    Ok(exported)
}

pub(super) fn append_geometry_tables(
    context: &GeometryExportContext<'_>,
    cityobject_ix: u64,
    geometry_handle: cityjson::prelude::GeometryHandle,
    ordinal: usize,
    exported: &mut ExportedGeometryTables,
) -> Result<()> {
    let geometry_id = raw_id_from_handle(geometry_handle);
    let geometry = context
        .relational
        .raw()
        .geometries()
        .resources()
        .get(raw_index_from_handle(geometry_handle))
        .and_then(Option::as_ref)
        .ok_or_else(|| {
            Error::Conversion(format!("missing geometry for handle {geometry_handle:?}"))
        })?;
    if *geometry.type_geometry() == GeometryType::GeometryInstance {
        return append_geometry_instance(cityobject_ix, geometry_id, geometry, ordinal, exported);
    }
    append_boundary_geometry_tables(cityobject_ix, geometry_id, geometry, ordinal, exported)
}

pub(super) fn append_geometry_instance(
    cityobject_ix: u64,
    geometry_id: u64,
    geometry: &Geometry<u32, cityjson::prelude::OwnedStringStorage>,
    ordinal: usize,
    exported: &mut ExportedGeometryTables,
) -> Result<()> {
    let instance = geometry.instance().ok_or_else(|| {
        Error::Conversion("geometry instance missing instance payload".to_string())
    })?;
    let template_geometry_id = raw_id_from_handle(instance.template());
    exported.instances.push(
        geometry_id,
        cityobject_ix,
        usize_to_u32(ordinal, "geometry ordinal")?,
        geometry.lod().map(ToString::to_string),
        template_geometry_id,
        u64::from(instance.reference_point().value()),
        Some(instance.transformation().into()),
    );
    Ok(())
}

pub(super) fn append_boundary_geometry_tables(
    cityobject_ix: u64,
    geometry_id: u64,
    geometry: &Geometry<u32, cityjson::prelude::OwnedStringStorage>,
    ordinal: usize,
    exported: &mut ExportedGeometryTables,
) -> Result<()> {
    let boundary = geometry.boundaries().ok_or_else(|| {
        Error::Conversion("boundary-carrying geometry missing boundaries".to_string())
    })?;
    let payload = borrowed_boundary_payload(*geometry.type_geometry(), boundary);
    append_geometry_semantic_rows(geometry_id, geometry, &payload, exported)?;
    append_geometry_material_rows(
        geometry_id,
        *geometry.type_geometry(),
        &payload,
        geometry.materials(),
        exported,
    )?;
    append_geometry_ring_texture_rows(
        geometry_id,
        *geometry.type_geometry(),
        &payload,
        geometry.textures(),
        exported,
    )?;
    exported.geometries.push(
        geometry_id,
        cityobject_ix,
        usize_to_u32(ordinal, "geometry ordinal")?,
        &geometry.type_geometry().to_string(),
        geometry.lod().map(ToString::to_string),
    );
    exported.boundaries.push(
        geometry_id,
        payload.vertex_indices,
        payload.line_offsets,
        payload.ring_offsets,
        payload.surface_offsets,
        payload.shell_offsets,
        payload.solid_offsets,
    )?;
    Ok(())
}

pub(super) fn append_geometry_semantic_rows(
    geometry_id: u64,
    geometry: &Geometry<u32, cityjson::prelude::OwnedStringStorage>,
    boundary: &impl BoundaryPayloadView,
    exported: &mut ExportedGeometryTables,
) -> Result<()> {
    let Some(semantics) = geometry.semantics() else {
        return Ok(());
    };
    match geometry.type_geometry() {
        GeometryType::MultiPoint => {
            if semantics.points().len() != boundary.vertex_indices().len() {
                return Err(Error::Conversion(format!(
                    "point semantic row count {} does not match point count {}",
                    semantics.points().len(),
                    boundary.vertex_indices().len()
                )));
            }
            for (point_ordinal, semantic_id) in semantics.points().iter().enumerate() {
                exported.point_semantics.push(
                    geometry_id,
                    usize_to_u32(point_ordinal, "point ordinal")?,
                    semantic_id.map(raw_id_from_handle),
                );
            }
        }
        GeometryType::MultiLineString => {
            let linestring_count = required_offsets(boundary.line_offsets(), "line_offsets")?.len();
            if semantics.linestrings().len() != linestring_count {
                return Err(Error::Conversion(format!(
                    "linestring semantic row count {} does not match linestring count {}",
                    semantics.linestrings().len(),
                    linestring_count
                )));
            }
            for (linestring_ordinal, semantic_id) in semantics.linestrings().iter().enumerate() {
                exported.linestring_semantics.push(
                    geometry_id,
                    usize_to_u32(linestring_ordinal, "linestring ordinal")?,
                    semantic_id.map(raw_id_from_handle),
                );
            }
        }
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => {
            for (surface_ordinal, semantic_id) in semantics.surfaces().iter().enumerate() {
                exported.surface_semantics.push(
                    geometry_id,
                    usize_to_u32(surface_ordinal, "surface ordinal")?,
                    semantic_id.map(raw_id_from_handle),
                );
            }
        }
        GeometryType::GeometryInstance => {
            return Err(Error::Unsupported("geometry instances".to_string()));
        }
        _ => return Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RingLayout {
    pub(super) start: usize,
    pub(super) len: usize,
    pub(super) surface_ordinal: u32,
    pub(super) ring_ordinal: u32,
}

pub(super) fn append_geometry_material_rows(
    geometry_id: u64,
    geometry_type: GeometryType,
    boundary: &impl BoundaryPayloadView,
    materials: Option<MaterialThemesView<'_, u32, cityjson::prelude::OwnedStringStorage>>,
    exported: &mut ExportedGeometryTables,
) -> Result<()> {
    let Some(materials) = materials else {
        return Ok(());
    };

    for (theme, map) in materials.iter() {
        match geometry_type {
            GeometryType::MultiSurface
            | GeometryType::CompositeSurface
            | GeometryType::Solid
            | GeometryType::MultiSolid
            | GeometryType::CompositeSolid => {
                let surface_count = surface_count(boundary);
                if map.surfaces().len() != surface_count {
                    return Err(Error::Conversion(format!(
                        "material theme {} has {} surface assignments, expected {}",
                        theme,
                        map.surfaces().len(),
                        surface_count
                    )));
                }
                for (surface_ordinal, material_handle) in map.surfaces().iter().enumerate() {
                    let Some(material_handle) = material_handle else {
                        continue;
                    };
                    exported.surface_materials.push(
                        geometry_id,
                        usize_to_u32(surface_ordinal, "surface ordinal")?,
                        theme.as_ref(),
                        raw_id_from_handle(*material_handle),
                    );
                }
            }
            GeometryType::GeometryInstance
            | GeometryType::MultiPoint
            | GeometryType::MultiLineString => {
                return Err(Error::Unsupported("geometry materials".to_string()));
            }
            _ => return Err(Error::Unsupported("unsupported geometry type".to_string())),
        }
    }

    Ok(())
}

pub(super) fn append_geometry_ring_texture_rows(
    geometry_id: u64,
    geometry_type: GeometryType,
    boundary: &impl BoundaryPayloadView,
    textures: Option<TextureThemesView<'_, u32, cityjson::prelude::OwnedStringStorage>>,
    exported: &mut ExportedGeometryTables,
) -> Result<()> {
    let Some(textures) = textures else {
        return Ok(());
    };
    ensure_surface_backed_geometry(geometry_type, "geometry textures")?;
    let ring_layouts = ring_layouts(boundary)?;

    for (theme, map) in textures.iter() {
        if map.rings().len() != ring_layouts.len() {
            return Err(Error::Conversion(format!(
                "texture theme {} has {} rings, expected {}",
                theme,
                map.rings().len(),
                ring_layouts.len()
            )));
        }
        if map.vertices().len() != boundary.vertex_indices().len() {
            return Err(Error::Conversion(format!(
                "texture theme {} has {} uv assignments, expected {}",
                theme,
                map.vertices().len(),
                boundary.vertex_indices().len()
            )));
        }
        let ring_textures = map.ring_textures();
        for (ring_index, layout) in ring_layouts.iter().enumerate() {
            let Some(texture_handle) = ring_textures[ring_index] else {
                continue;
            };
            let uv_indices = map.vertices()[layout.start..layout.start + layout.len]
                .iter()
                .map(|value: &Option<cityjson::v2_0::VertexIndex<u32>>| {
                    value
                        .map(|uv: cityjson::v2_0::VertexIndex<u32>| u64::from(uv.value()))
                        .ok_or_else(|| {
                            Error::Conversion(format!(
                                "textured ring {ring_index} for theme {theme} contains missing uv indices"
                            ))
                        })
                })
                .collect::<Result<Vec<_>>>()?;
            exported.ring_textures.push(
                geometry_id,
                layout.surface_ordinal,
                layout.ring_ordinal,
                theme.as_ref(),
                raw_id_from_handle(texture_handle),
                &uv_indices,
            )?;
        }
    }

    Ok(())
}

pub(super) fn append_template_geometry_ring_texture_rows(
    template_geometry_id: u64,
    geometry_type: GeometryType,
    boundary: &impl BoundaryPayloadView,
    textures: &TextureThemesView<'_, u32, cityjson::prelude::OwnedStringStorage>,
    exported: &mut ExportedTemplateGeometryTables,
) -> Result<()> {
    ensure_surface_backed_geometry(geometry_type, "template geometry textures")?;
    let ring_layouts = template_ring_layouts(boundary)?;

    for (theme, map) in textures.iter() {
        if map.rings().len() != ring_layouts.len() {
            return Err(Error::Conversion(format!(
                "template geometry texture theme {} has {} rings, expected {}",
                theme,
                map.rings().len(),
                ring_layouts.len()
            )));
        }
        if map.vertices().len() != boundary.vertex_indices().len() {
            return Err(Error::Conversion(format!(
                "template geometry texture theme {} has {} uv assignments, expected {}",
                theme,
                map.vertices().len(),
                boundary.vertex_indices().len()
            )));
        }
        let ring_textures = map.ring_textures();
        for (ring_index, layout) in ring_layouts.iter().enumerate() {
            let Some(texture_handle) = ring_textures[ring_index] else {
                continue;
            };
            let uv_indices = map.vertices()[layout.start..layout.start + layout.len]
                .iter()
                .map(|value: &Option<cityjson::v2_0::VertexIndex<u32>>| {
                    value
                        .map(|uv: cityjson::v2_0::VertexIndex<u32>| u64::from(uv.value()))
                        .ok_or_else(|| {
                            Error::Conversion(format!(
                                "template textured ring {ring_index} for theme {theme} contains missing uv indices"
                            ))
                        })
                })
                .collect::<Result<Vec<_>>>()?;
            exported.ring_textures.push(
                template_geometry_id,
                layout.surface_ordinal,
                layout.ring_ordinal,
                theme.as_ref(),
                raw_id_from_handle(texture_handle),
                &uv_indices,
            )?;
        }
    }

    Ok(())
}

pub(super) fn ensure_surface_backed_geometry(
    geometry_type: GeometryType,
    feature: &str,
) -> Result<()> {
    match geometry_type {
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => Ok(()),
        GeometryType::MultiPoint | GeometryType::MultiLineString => {
            Err(Error::Unsupported(feature.to_string()))
        }
        GeometryType::GeometryInstance => Err(Error::Unsupported(feature.to_string())),
        _ => Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
}

pub(super) fn ring_layouts(boundary: &impl BoundaryPayloadView) -> Result<Vec<RingLayout>> {
    let ring_offsets = required_offsets(boundary.ring_offsets(), "ring_offsets")?;
    let surface_offsets = required_offsets(boundary.surface_offsets(), "surface_offsets")?;
    validate_offsets(
        ring_offsets,
        boundary.vertex_indices().len(),
        "ring_offsets",
    )?;
    validate_offsets(surface_offsets, ring_offsets.len(), "surface_offsets")?;

    let mut layouts = Vec::with_capacity(ring_offsets.len());
    for (surface_ordinal, surface_index) in surface_offsets.iter().enumerate() {
        let surface_start = offset_to_usize(*surface_index, ring_offsets.len(), "surface_offsets")?;
        let surface_end = offset_end(
            surface_offsets,
            surface_ordinal,
            ring_offsets.len(),
            "surface_offsets",
        )?;
        for (ring_ordinal, ring_index) in (surface_start..surface_end).enumerate() {
            let start = offset_to_usize(
                ring_offsets[ring_index],
                boundary.vertex_indices().len(),
                "ring_offsets",
            )?;
            let end = offset_end(
                ring_offsets,
                ring_index,
                boundary.vertex_indices().len(),
                "ring_offsets",
            )?;
            layouts.push(RingLayout {
                start,
                len: end - start,
                surface_ordinal: usize_to_u32(surface_ordinal, "surface ordinal")?,
                ring_ordinal: usize_to_u32(ring_ordinal, "ring ordinal")?,
            });
        }
    }
    Ok(layouts)
}

pub(super) fn template_ring_layouts(
    boundary: &impl BoundaryPayloadView,
) -> Result<Vec<RingLayout>> {
    let ring_offsets = required_offsets(boundary.ring_offsets(), "ring_offsets")?;
    let surface_offsets = required_offsets(boundary.surface_offsets(), "surface_offsets")?;
    validate_offsets(
        ring_offsets,
        boundary.vertex_indices().len(),
        "ring_offsets",
    )?;
    validate_offsets(surface_offsets, ring_offsets.len(), "surface_offsets")?;

    let mut layouts = Vec::with_capacity(ring_offsets.len());
    for (surface_ordinal, surface_index) in surface_offsets.iter().enumerate() {
        let surface_start = offset_to_usize(*surface_index, ring_offsets.len(), "surface_offsets")?;
        let surface_end = offset_end(
            surface_offsets,
            surface_ordinal,
            ring_offsets.len(),
            "surface_offsets",
        )?;
        for (ring_ordinal, ring_index) in (surface_start..surface_end).enumerate() {
            let start = offset_to_usize(
                ring_offsets[ring_index],
                boundary.vertex_indices().len(),
                "ring_offsets",
            )?;
            let end = offset_end(
                ring_offsets,
                ring_index,
                boundary.vertex_indices().len(),
                "ring_offsets",
            )?;
            layouts.push(RingLayout {
                start,
                len: end - start,
                surface_ordinal: usize_to_u32(surface_ordinal, "surface ordinal")?,
                ring_ordinal: usize_to_u32(ring_ordinal, "ring ordinal")?,
            });
        }
    }
    Ok(layouts)
}

pub(super) fn rgb_to_components(value: RGB) -> [f64; 3] {
    value.to_array().map(f64::from)
}

pub(super) fn rgba_to_components(value: RGBA) -> [f64; 4] {
    value.to_array().map(f64::from)
}

pub(super) fn rgb_from_components(value: [f64; 3]) -> RGB {
    RGB::from(value.map(decode_payload_f32))
}

pub(super) fn rgba_from_components(value: [f64; 4]) -> RGBA {
    RGBA::from(value.map(decode_payload_f32))
}

pub(super) trait BoundaryPayloadView {
    fn vertex_indices(&self) -> &[u32];
    fn line_offsets(&self) -> Option<&[u32]>;
    fn ring_offsets(&self) -> Option<&[u32]>;
    fn surface_offsets(&self) -> Option<&[u32]>;
    fn shell_offsets(&self) -> Option<&[u32]>;
    fn solid_offsets(&self) -> Option<&[u32]>;
}

pub(super) fn template_geometry_tables(
    relational: &ModelRelationalView<'_>,
) -> Result<ExportedTemplateGeometryTables> {
    let mut exported = ExportedTemplateGeometryTables::default();
    for (handle, geometry) in relational.model().iter_geometry_templates() {
        append_template_geometry_tables(handle, geometry, &mut exported)?;
    }
    Ok(exported)
}

pub(super) fn append_template_geometry_tables(
    handle: cityjson::prelude::GeometryTemplateHandle,
    geometry: &Geometry<u32, cityjson::prelude::OwnedStringStorage>,
    exported: &mut ExportedTemplateGeometryTables,
) -> Result<()> {
    let template_geometry_id = raw_id_from_handle(handle);
    let boundary = geometry
        .boundaries()
        .ok_or_else(|| Error::Conversion("template geometry missing boundaries".to_string()))?;
    let payload = borrowed_boundary_payload(*geometry.type_geometry(), boundary);
    exported.geometries.push(
        template_geometry_id,
        &geometry.type_geometry().to_string(),
        geometry.lod().map(ToString::to_string),
    );
    append_template_semantic_rows(template_geometry_id, geometry, &payload, exported)?;
    append_template_material_rows(template_geometry_id, geometry, &payload, exported)?;
    if let Some(textures) = geometry.textures() {
        append_template_geometry_ring_texture_rows(
            template_geometry_id,
            *geometry.type_geometry(),
            &payload,
            &textures,
            exported,
        )?;
    }
    exported.boundaries.push(
        template_geometry_id,
        payload.vertex_indices,
        payload.line_offsets,
        payload.ring_offsets,
        payload.surface_offsets,
        payload.shell_offsets,
        payload.solid_offsets,
    )?;
    Ok(())
}

pub(super) fn append_template_semantic_rows(
    template_geometry_id: u64,
    geometry: &Geometry<u32, cityjson::prelude::OwnedStringStorage>,
    boundary: &impl BoundaryPayloadView,
    exported: &mut ExportedTemplateGeometryTables,
) -> Result<()> {
    let Some(semantics) = geometry.semantics() else {
        return Ok(());
    };
    match geometry.type_geometry() {
        GeometryType::MultiPoint => {
            if semantics.points().len() != boundary.vertex_indices().len() {
                return Err(Error::Conversion(format!(
                    "template geometry {} has {} point semantics, expected {}",
                    template_geometry_id,
                    semantics.points().len(),
                    boundary.vertex_indices().len()
                )));
            }
            for (primitive_ordinal, semantic_id) in semantics.points().iter().enumerate() {
                exported.semantics.push(
                    template_geometry_id,
                    PRIMITIVE_TYPE_POINT,
                    usize_to_u32(primitive_ordinal, "primitive ordinal")?,
                    semantic_id.map(raw_id_from_handle),
                );
            }
        }
        GeometryType::MultiLineString => {
            let linestring_count = required_offsets(boundary.line_offsets(), "line_offsets")?.len();
            if semantics.linestrings().len() != linestring_count {
                return Err(Error::Conversion(format!(
                    "template geometry {} has {} linestring semantics, expected {}",
                    template_geometry_id,
                    semantics.linestrings().len(),
                    linestring_count
                )));
            }
            for (primitive_ordinal, semantic_id) in semantics.linestrings().iter().enumerate() {
                exported.semantics.push(
                    template_geometry_id,
                    PRIMITIVE_TYPE_LINESTRING,
                    usize_to_u32(primitive_ordinal, "primitive ordinal")?,
                    semantic_id.map(raw_id_from_handle),
                );
            }
        }
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => {
            for (primitive_ordinal, semantic_id) in semantics.surfaces().iter().enumerate() {
                exported.semantics.push(
                    template_geometry_id,
                    PRIMITIVE_TYPE_SURFACE,
                    usize_to_u32(primitive_ordinal, "primitive ordinal")?,
                    semantic_id.map(raw_id_from_handle),
                );
            }
        }
        GeometryType::GeometryInstance => {
            return Err(Error::Unsupported("geometry instances".to_string()));
        }
        _ => return Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
    Ok(())
}

pub(super) fn append_template_material_rows(
    template_geometry_id: u64,
    geometry: &Geometry<u32, cityjson::prelude::OwnedStringStorage>,
    boundary: &impl BoundaryPayloadView,
    exported: &mut ExportedTemplateGeometryTables,
) -> Result<()> {
    let Some(materials) = geometry.materials() else {
        return Ok(());
    };
    for (theme, map) in materials.iter() {
        match geometry.type_geometry() {
            GeometryType::MultiPoint => {
                if map.points().len() != boundary.vertex_indices().len() {
                    return Err(Error::Conversion(format!(
                        "template geometry {} material theme {} has {} point assignments, expected {}",
                        template_geometry_id,
                        theme,
                        map.points().len(),
                        boundary.vertex_indices().len()
                    )));
                }
                for (primitive_ordinal, material_handle) in map.points().iter().enumerate() {
                    let Some(material_handle) = material_handle else {
                        continue;
                    };
                    exported.materials.push(
                        template_geometry_id,
                        PRIMITIVE_TYPE_POINT,
                        usize_to_u32(primitive_ordinal, "primitive ordinal")?,
                        theme.as_ref(),
                        raw_id_from_handle(*material_handle),
                    );
                }
            }
            GeometryType::MultiLineString => {
                let linestring_count =
                    required_offsets(boundary.line_offsets(), "line_offsets")?.len();
                if map.linestrings().len() != linestring_count {
                    return Err(Error::Conversion(format!(
                        "template geometry {} material theme {} has {} linestring assignments, expected {}",
                        template_geometry_id,
                        theme,
                        map.linestrings().len(),
                        linestring_count
                    )));
                }
                for (primitive_ordinal, material_handle) in map.linestrings().iter().enumerate() {
                    let Some(material_handle) = material_handle else {
                        continue;
                    };
                    exported.materials.push(
                        template_geometry_id,
                        PRIMITIVE_TYPE_LINESTRING,
                        usize_to_u32(primitive_ordinal, "primitive ordinal")?,
                        theme.as_ref(),
                        raw_id_from_handle(*material_handle),
                    );
                }
            }
            GeometryType::MultiSurface
            | GeometryType::CompositeSurface
            | GeometryType::Solid
            | GeometryType::MultiSolid
            | GeometryType::CompositeSolid => {
                let surface_count = template_surface_count(boundary);
                if map.surfaces().len() != surface_count {
                    return Err(Error::Conversion(format!(
                        "template geometry {} material theme {} has {} surface assignments, expected {}",
                        template_geometry_id,
                        theme,
                        map.surfaces().len(),
                        surface_count
                    )));
                }
                for (primitive_ordinal, material_handle) in map.surfaces().iter().enumerate() {
                    let Some(material_handle) = material_handle else {
                        continue;
                    };
                    exported.materials.push(
                        template_geometry_id,
                        PRIMITIVE_TYPE_SURFACE,
                        usize_to_u32(primitive_ordinal, "primitive ordinal")?,
                        theme.as_ref(),
                        raw_id_from_handle(*material_handle),
                    );
                }
            }
            GeometryType::GeometryInstance => {
                return Err(Error::Unsupported("geometry materials".to_string()));
            }
            _ => return Err(Error::Unsupported("unsupported geometry type".to_string())),
        }
    }
    Ok(())
}

pub(super) fn borrowed_boundary_payload(
    geometry_type: GeometryType,
    boundary: &Boundary<u32>,
) -> BorrowedBoundary<'_> {
    let columnar = boundary.to_columnar();
    let vertex_indices = raw_vertex_index_slice(columnar.vertices);
    let ring_offsets = raw_vertex_index_slice(columnar.ring_offsets);
    let surface_offsets = raw_vertex_index_slice(columnar.surface_offsets);
    let shell_offsets = raw_vertex_index_slice(columnar.shell_offsets);
    let solid_offsets = raw_vertex_index_slice(columnar.solid_offsets);

    let (line_offsets, ring_offsets, surface_offsets, shell_offsets, solid_offsets) =
        match geometry_type {
            GeometryType::MultiPoint => (None, None, None, None, None),
            GeometryType::MultiLineString => (Some(ring_offsets), None, None, None, None),
            GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                (None, Some(ring_offsets), Some(surface_offsets), None, None)
            }
            GeometryType::Solid => (
                None,
                Some(ring_offsets),
                Some(surface_offsets),
                Some(shell_offsets),
                None,
            ),
            GeometryType::MultiSolid | GeometryType::CompositeSolid => (
                None,
                Some(ring_offsets),
                Some(surface_offsets),
                Some(shell_offsets),
                Some(solid_offsets),
            ),
            GeometryType::GeometryInstance => unreachable!("instances rejected earlier"),
            _ => unreachable!("unsupported geometry type rejected earlier"),
        };

    BorrowedBoundary {
        vertex_indices,
        line_offsets,
        ring_offsets,
        surface_offsets,
        shell_offsets,
        solid_offsets,
    }
}

pub(super) fn raw_vertex_index_slice(values: &[cityjson::v2_0::VertexIndex<u32>]) -> &[u32] {
    const {
        assert!(
            std::mem::size_of::<cityjson::v2_0::VertexIndex<u32>>() == std::mem::size_of::<u32>()
        );
        assert!(
            std::mem::align_of::<cityjson::v2_0::VertexIndex<u32>>() == std::mem::align_of::<u32>()
        );
    }

    // SAFETY: `VertexIndex<u32>` is `#[repr(transparent)]` over `u32`.
    unsafe { std::slice::from_raw_parts(values.as_ptr().cast::<u32>(), values.len()) }
}

impl BoundaryPayloadView for BorrowedBoundary<'_> {
    fn vertex_indices(&self) -> &[u32] {
        self.vertex_indices
    }

    fn line_offsets(&self) -> Option<&[u32]> {
        self.line_offsets
    }

    fn ring_offsets(&self) -> Option<&[u32]> {
        self.ring_offsets
    }

    fn surface_offsets(&self) -> Option<&[u32]> {
        self.surface_offsets
    }

    fn shell_offsets(&self) -> Option<&[u32]> {
        self.shell_offsets
    }

    fn solid_offsets(&self) -> Option<&[u32]> {
        self.solid_offsets
    }
}

pub(super) struct IndexedSemanticMapSpec<'a> {
    label: &'a str,
    expected_count: usize,
}

pub(super) fn build_indexed_semantic_map<F>(
    rows: &Range<usize>,
    view: &IndexedSemanticBatchView,
    spec: &IndexedSemanticMapSpec<'_>,
    handles: &HashMap<u64, cityjson::prelude::SemanticHandle>,
    append: F,
) -> Result<Option<SemanticMap<u32>>>
where
    F: Fn(&mut SemanticMap<u32>, Option<cityjson::prelude::SemanticHandle>),
{
    if rows.is_empty() {
        return Ok(None);
    }
    if rows.len() != spec.expected_count {
        return Err(Error::Conversion(format!(
            "{label} semantic row count {} does not match {label} count {}",
            rows.len(),
            spec.expected_count,
            label = spec.label,
        )));
    }

    let mut map = SemanticMap::new();
    for (expected_ordinal, row_index) in rows.clone().enumerate() {
        let actual_ordinal = usize::try_from(view.ordinal.value(row_index))
            .expect("u32 semantic ordinal fits into usize");
        if actual_ordinal != expected_ordinal {
            return Err(Error::Conversion(format!(
                "{label} semantic ordinal {} is out of order, expected {}",
                view.ordinal.value(row_index),
                expected_ordinal,
                label = spec.label,
            )));
        }
        let semantic_id =
            (!view.semantic_id.is_null(row_index)).then(|| view.semantic_id.value(row_index));
        append(
            &mut map,
            semantic_id.and_then(|id| handles.get(&id).copied()),
        );
    }
    Ok(Some(map))
}

pub(super) fn build_semantic_map(
    geometry_type: &str,
    boundary: &impl BoundaryPayloadView,
    surface_rows: Option<&GroupedBatchView<IndexedSemanticBatchView>>,
    point_rows: Option<&GroupedBatchView<IndexedSemanticBatchView>>,
    linestring_rows: Option<&GroupedBatchView<IndexedSemanticBatchView>>,
    geometry_id: u64,
    handles: &HashMap<u64, cityjson::prelude::SemanticHandle>,
) -> Result<Option<SemanticMap<u32>>> {
    match parse_geometry_type(geometry_type)? {
        GeometryType::MultiPoint => {
            let Some(grouped) = point_rows else {
                return Ok(None);
            };
            let Some(rows) = grouped_row_range(Some(grouped), geometry_id) else {
                return Ok(None);
            };
            build_indexed_semantic_map(
                rows,
                &grouped.view,
                &IndexedSemanticMapSpec {
                    label: "point",
                    expected_count: boundary.vertex_indices().len(),
                },
                handles,
                SemanticMap::add_point,
            )
        }
        GeometryType::MultiLineString => {
            let Some(grouped) = linestring_rows else {
                return Ok(None);
            };
            let Some(rows) = grouped_row_range(Some(grouped), geometry_id) else {
                return Ok(None);
            };
            build_indexed_semantic_map(
                rows,
                &grouped.view,
                &IndexedSemanticMapSpec {
                    label: "linestring",
                    expected_count: required_offsets(boundary.line_offsets(), "line_offsets")?
                        .len(),
                },
                handles,
                SemanticMap::add_linestring,
            )
        }
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => {
            let Some(grouped) = surface_rows else {
                return Ok(None);
            };
            let Some(rows) = grouped_row_range(Some(grouped), geometry_id) else {
                return Ok(None);
            };
            build_indexed_semantic_map(
                rows,
                &grouped.view,
                &IndexedSemanticMapSpec {
                    label: "surface",
                    expected_count: surface_count(boundary),
                },
                handles,
                SemanticMap::add_surface,
            )
        }
        GeometryType::GeometryInstance => Err(Error::Unsupported("geometry instances".to_string())),
        _ => Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
}

pub(super) fn build_material_maps(
    geometry_type: &str,
    boundary: &impl BoundaryPayloadView,
    surface_rows: Option<&GroupedBatchView<GeometrySurfaceMaterialBatchView>>,
    geometry_id: u64,
    handles: &HashMap<u64, cityjson::prelude::MaterialHandle>,
) -> Result<Option<MaterialThemeMaps>> {
    match parse_geometry_type(geometry_type)? {
        GeometryType::MultiPoint | GeometryType::MultiLineString => Ok(None),
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => {
            let Some(rows) = grouped_row_range(surface_rows, geometry_id) else {
                return Ok(None);
            };
            let view = &surface_rows.expect("checked above").view;
            grouped_material_maps(
                rows,
                surface_count(boundary),
                |row| {
                    Ok((
                        view.theme.value(row).to_string(),
                        view.surface_ordinal.value(row),
                        view.material_id.value(row),
                    ))
                },
                MaterialMap::add_surface,
                |row, count| format!("material assignment row {row} exceeds surface count {count}"),
                |row| format!("duplicate material assignment at row {row}"),
                handles,
            )
            .map(Some)
        }
        GeometryType::GeometryInstance => Err(Error::Unsupported("geometry materials".to_string())),
        _ => Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
}

pub(super) fn grouped_material_maps<FFields, FAppend, FExceeds, FDuplicate>(
    rows: &Range<usize>,
    primitive_count: usize,
    fields: FFields,
    append: FAppend,
    exceeds_message: FExceeds,
    duplicate_message: FDuplicate,
    handles: &HashMap<u64, cityjson::prelude::MaterialHandle>,
) -> Result<MaterialThemeMaps>
where
    FFields: Fn(usize) -> Result<(String, u32, u64)>,
    FAppend: Fn(&mut MaterialMap<u32>, Option<cityjson::prelude::MaterialHandle>),
    FExceeds: Fn(usize, usize) -> String,
    FDuplicate: Fn(usize) -> String,
{
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let mut grouped = BTreeMap::<String, Vec<Option<cityjson::prelude::MaterialHandle>>>::new();
    for row in rows.clone() {
        let (theme, ordinal, id) = fields(row)?;
        let ordinal = usize::try_from(ordinal).expect("u32 ordinal fits into usize");
        if ordinal >= primitive_count {
            return Err(Error::Conversion(exceeds_message(row, primitive_count)));
        }
        let material = *handles
            .get(&id)
            .ok_or_else(|| Error::Conversion(format!("missing material {id}")))?;
        let entries = grouped
            .entry(theme)
            .or_insert_with(|| vec![None; primitive_count]);
        if entries[ordinal].is_some() {
            return Err(Error::Conversion(duplicate_message(row)));
        }
        entries[ordinal] = Some(material);
    }
    Ok(grouped
        .into_iter()
        .map(|(theme, values)| {
            let mut map = MaterialMap::new();
            for value in values {
                append(&mut map, value);
            }
            (ThemeName::new(theme), map)
        })
        .collect())
}

pub(super) fn build_ordered_template_semantic_map(
    view: &TemplateSemanticBatchView,
    rows: &Range<usize>,
    expected_primitive_type: &str,
    expected_count: usize,
    count_label: &str,
    handles: &HashMap<u64, cityjson::prelude::SemanticHandle>,
) -> Result<Option<SemanticMap<u32>>> {
    if rows.is_empty() {
        return Ok(None);
    }
    if rows.len() != expected_count {
        return Err(Error::Conversion(format!(
            "template {count_label} semantic row count {} does not match {count_label} count {}",
            rows.len(),
            expected_count
        )));
    }

    let mut map = SemanticMap::new();
    for (expected_ordinal, row_index) in rows.clone().enumerate() {
        if view.primitive_type.value(row_index) != expected_primitive_type {
            return Err(Error::Conversion(format!(
                "template {count_label} semantic row has unexpected primitive type {}",
                view.primitive_type.value(row_index)
            )));
        }
        let actual_ordinal = usize::try_from(view.primitive_ordinal.value(row_index))
            .expect("u32 primitive ordinal fits into usize");
        if actual_ordinal != expected_ordinal {
            return Err(Error::Conversion(format!(
                "template {count_label} semantic ordinal {} is out of order, expected {}",
                view.primitive_ordinal.value(row_index),
                expected_ordinal
            )));
        }
        let semantic_id =
            (!view.semantic_id.is_null(row_index)).then(|| view.semantic_id.value(row_index));
        let handle = semantic_id.and_then(|id| handles.get(&id).copied());
        match expected_primitive_type {
            PRIMITIVE_TYPE_POINT => map.add_point(handle),
            PRIMITIVE_TYPE_LINESTRING => map.add_linestring(handle),
            PRIMITIVE_TYPE_SURFACE => map.add_surface(handle),
            other => {
                return Err(Error::Conversion(format!(
                    "unsupported template semantic primitive type {other}"
                )));
            }
        }
    }

    Ok(Some(map))
}

pub(super) fn build_template_semantic_map(
    geometry_type: &str,
    boundary: &impl BoundaryPayloadView,
    rows: Option<&GroupedBatchView<TemplateSemanticBatchView>>,
    template_geometry_id: u64,
    handles: &HashMap<u64, cityjson::prelude::SemanticHandle>,
) -> Result<Option<SemanticMap<u32>>> {
    let Some(grouped) = rows else {
        return Ok(None);
    };
    let Some(rows) = grouped_row_range(Some(grouped), template_geometry_id) else {
        return Ok(None);
    };
    match parse_geometry_type(geometry_type)? {
        GeometryType::MultiPoint => build_ordered_template_semantic_map(
            &grouped.view,
            rows,
            PRIMITIVE_TYPE_POINT,
            boundary.vertex_indices().len(),
            "point",
            handles,
        ),
        GeometryType::MultiLineString => build_ordered_template_semantic_map(
            &grouped.view,
            rows,
            PRIMITIVE_TYPE_LINESTRING,
            required_offsets(boundary.line_offsets(), "line_offsets")?.len(),
            "linestring",
            handles,
        ),
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => build_ordered_template_semantic_map(
            &grouped.view,
            rows,
            PRIMITIVE_TYPE_SURFACE,
            template_surface_count(boundary),
            "surface",
            handles,
        ),
        GeometryType::GeometryInstance => Err(Error::Unsupported("geometry instances".to_string())),
        _ => Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
}

pub(super) fn build_template_material_maps(
    geometry_type: &str,
    boundary: &impl BoundaryPayloadView,
    rows: Option<&GroupedBatchView<TemplateMaterialBatchView>>,
    template_geometry_id: u64,
    handles: &HashMap<u64, cityjson::prelude::MaterialHandle>,
) -> Result<Option<MaterialThemeMaps>> {
    let Some(grouped) = rows else {
        return Ok(None);
    };
    let Some(rows) = grouped_row_range(Some(grouped), template_geometry_id) else {
        return Ok(None);
    };
    let view = &grouped.view;
    match parse_geometry_type(geometry_type)? {
        GeometryType::MultiPoint => grouped_material_maps(
            rows,
            boundary.vertex_indices().len(),
            |row| {
                if view.primitive_type.value(row) != PRIMITIVE_TYPE_POINT {
                    return Err(Error::Conversion(format!(
                        "template point material row has unexpected primitive type {}",
                        view.primitive_type.value(row)
                    )));
                }
                Ok((
                    view.theme.value(row).to_string(),
                    view.primitive_ordinal.value(row),
                    view.material_id.value(row),
                ))
            },
            MaterialMap::add_point,
            |row, count| {
                format!("template material assignment row {row} exceeds point count {count}",)
            },
            |row| format!("duplicate template material assignment at row {row}"),
            handles,
        )
        .map(Some),
        GeometryType::MultiLineString => grouped_material_maps(
            rows,
            required_offsets(boundary.line_offsets(), "line_offsets")?.len(),
            |row| {
                if view.primitive_type.value(row) != PRIMITIVE_TYPE_LINESTRING {
                    return Err(Error::Conversion(format!(
                        "template linestring material row has unexpected primitive type {}",
                        view.primitive_type.value(row)
                    )));
                }
                Ok((
                    view.theme.value(row).to_string(),
                    view.primitive_ordinal.value(row),
                    view.material_id.value(row),
                ))
            },
            MaterialMap::add_linestring,
            |row, count| {
                format!("template material assignment row {row} exceeds linestring count {count}",)
            },
            |row| format!("duplicate template material assignment at row {row}"),
            handles,
        )
        .map(Some),
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => grouped_material_maps(
            rows,
            template_surface_count(boundary),
            |row| {
                if view.primitive_type.value(row) != PRIMITIVE_TYPE_SURFACE {
                    return Err(Error::Conversion(format!(
                        "template surface material row has unexpected primitive type {}",
                        view.primitive_type.value(row)
                    )));
                }
                Ok((
                    view.theme.value(row).to_string(),
                    view.primitive_ordinal.value(row),
                    view.material_id.value(row),
                ))
            },
            MaterialMap::add_surface,
            |row, count| {
                format!("template material assignment row {row} exceeds surface count {count}",)
            },
            |row| format!("duplicate template material assignment at row {row}"),
            handles,
        )
        .map(Some),
        GeometryType::GeometryInstance => Err(Error::Unsupported("geometry materials".to_string())),
        _ => Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
}

pub(super) fn build_template_texture_maps(
    geometry_type: &str,
    boundary: &impl BoundaryPayloadView,
    rows: Option<&GroupedBatchView<RingTextureBatchView>>,
    template_geometry_id: u64,
    handles: &HashMap<u64, cityjson::prelude::TextureHandle>,
) -> Result<Option<TextureThemeMaps>> {
    let Some(grouped) = rows else {
        return Ok(None);
    };
    let Some(rows) = grouped_row_range(Some(grouped), template_geometry_id) else {
        return Ok(None);
    };
    let view = &grouped.view;
    match parse_geometry_type(geometry_type)? {
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => {
            let ring_layouts = template_ring_layouts(boundary)?;
            let ring_lookup = ring_layouts
                .iter()
                .enumerate()
                .map(|(index, layout)| ((layout.surface_ordinal, layout.ring_ordinal), index))
                .collect::<HashMap<_, _>>();
            let mut maps = BTreeMap::<String, TextureMap<u32>>::new();

            for row_index in rows.clone() {
                let surface_ordinal = view.surface_ordinal.value(row_index);
                let ring_ordinal = view.ring_ordinal.value(row_index);
                let layout_index = *ring_lookup
                    .get(&(surface_ordinal, ring_ordinal))
                    .ok_or_else(|| {
                        Error::Conversion(format!(
                            "missing template ring layout for surface {surface_ordinal} ring {ring_ordinal}"
                        ))
                    })?;
                let layout = ring_layouts[layout_index];
                let uv_indices = view.uv_indices.value(row_index)?;
                if uv_indices.len() != layout.len {
                    return Err(Error::Conversion(format!(
                        "template texture assignment for theme {} surface {} ring {} has {} uv indices, expected {}",
                        view.theme.value(row_index),
                        surface_ordinal,
                        ring_ordinal,
                        uv_indices.len(),
                        layout.len
                    )));
                }
                let texture_id = view.texture_id.value(row_index);
                let texture = *handles
                    .get(&texture_id)
                    .ok_or_else(|| Error::Conversion(format!("missing texture {texture_id}")))?;
                let theme = view.theme.value(row_index).to_string();
                if !maps.contains_key(&theme) {
                    maps.insert(theme.clone(), empty_texture_map(&ring_layouts)?);
                }
                let map = maps.get_mut(&theme).expect("texture theme map must exist");
                if map.ring_textures()[layout_index].is_some() {
                    return Err(Error::Conversion(format!(
                        "duplicate template texture assignment for theme {theme} surface {surface_ordinal} ring {ring_ordinal}"
                    )));
                }
                if !map.set_ring_texture(layout_index, Some(texture)) {
                    return Err(Error::Conversion("missing texture ring slot".to_string()));
                }
                let slice = map.vertices_mut()[layout.start..layout.start + layout.len].iter_mut();
                for (slot, uv_id) in slice.zip(uv_indices.iter()) {
                    let uv_id = u32::try_from(*uv_id).map_err(|_| {
                        Error::Conversion(format!("uv index {uv_id} does not fit into u32"))
                    })?;
                    *slot = Some(cityjson::v2_0::VertexIndex::new(uv_id));
                }
            }

            Ok(Some(
                maps.into_iter()
                    .map(|(theme, map)| (ThemeName::new(theme), map))
                    .collect(),
            ))
        }
        GeometryType::MultiPoint | GeometryType::MultiLineString => {
            Err(Error::Unsupported("geometry textures".to_string()))
        }
        GeometryType::GeometryInstance => Err(Error::Unsupported("geometry textures".to_string())),
        _ => Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
}

pub(super) fn build_texture_maps(
    geometry_type: &str,
    boundary: &impl BoundaryPayloadView,
    rows: Option<&GroupedBatchView<RingTextureBatchView>>,
    geometry_id: u64,
    handles: &HashMap<u64, cityjson::prelude::TextureHandle>,
) -> Result<Option<TextureThemeMaps>> {
    let Some(grouped) = rows else {
        return Ok(None);
    };
    let Some(rows) = grouped_row_range(Some(grouped), geometry_id) else {
        return Ok(None);
    };
    let view = &grouped.view;
    match parse_geometry_type(geometry_type)? {
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => {
            let ring_layouts = ring_layouts(boundary)?;
            let ring_lookup = ring_layouts
                .iter()
                .enumerate()
                .map(|(index, layout)| ((layout.surface_ordinal, layout.ring_ordinal), index))
                .collect::<HashMap<_, _>>();
            let mut maps = BTreeMap::<String, TextureMap<u32>>::new();

            for row_index in rows.clone() {
                let surface_ordinal = view.surface_ordinal.value(row_index);
                let ring_ordinal = view.ring_ordinal.value(row_index);
                let layout_index = *ring_lookup
                    .get(&(surface_ordinal, ring_ordinal))
                    .ok_or_else(|| {
                        Error::Conversion(format!(
                            "missing ring layout for surface {surface_ordinal} ring {ring_ordinal}"
                        ))
                    })?;
                let layout = ring_layouts[layout_index];
                let uv_indices = view.uv_indices.value(row_index)?;
                if uv_indices.len() != layout.len {
                    return Err(Error::Conversion(format!(
                        "texture assignment for theme {} surface {} ring {} has {} uv indices, expected {}",
                        view.theme.value(row_index),
                        surface_ordinal,
                        ring_ordinal,
                        uv_indices.len(),
                        layout.len
                    )));
                }
                let texture_id = view.texture_id.value(row_index);
                let texture = *handles
                    .get(&texture_id)
                    .ok_or_else(|| Error::Conversion(format!("missing texture {texture_id}")))?;
                let theme = view.theme.value(row_index).to_string();
                if !maps.contains_key(&theme) {
                    maps.insert(theme.clone(), empty_texture_map(&ring_layouts)?);
                }
                let map = maps.get_mut(&theme).expect("texture theme map must exist");
                if map.ring_textures()[layout_index].is_some() {
                    return Err(Error::Conversion(format!(
                        "duplicate texture assignment for theme {theme} surface {surface_ordinal} ring {ring_ordinal}"
                    )));
                }
                if !map.set_ring_texture(layout_index, Some(texture)) {
                    return Err(Error::Conversion("missing texture ring slot".to_string()));
                }
                let slice = map.vertices_mut()[layout.start..layout.start + layout.len].iter_mut();
                for (slot, uv_id) in slice.zip(uv_indices.iter()) {
                    let uv_id = u32::try_from(*uv_id).map_err(|_| {
                        Error::Conversion(format!("uv index {uv_id} does not fit into u32"))
                    })?;
                    *slot = Some(cityjson::v2_0::VertexIndex::new(uv_id));
                }
            }

            Ok(Some(
                maps.into_iter()
                    .map(|(theme, map)| (ThemeName::new(theme), map))
                    .collect(),
            ))
        }
        GeometryType::MultiPoint | GeometryType::MultiLineString => {
            Err(Error::Unsupported("geometry textures".to_string()))
        }
        GeometryType::GeometryInstance => Err(Error::Unsupported("geometry textures".to_string())),
        _ => Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
}

pub(super) fn boundary_from_payload(
    row: &impl BoundaryPayloadView,
    geometry_type: &str,
) -> Result<Boundary<u32>> {
    boundary_from_parts(
        row.vertex_indices(),
        row.line_offsets(),
        row.ring_offsets(),
        row.surface_offsets(),
        row.shell_offsets(),
        row.solid_offsets(),
        geometry_type,
    )
}

pub(super) fn boundary_from_parts(
    vertex_indices: &[u32],
    line_offsets: Option<&[u32]>,
    ring_offsets: Option<&[u32]>,
    surface_offsets: Option<&[u32]>,
    shell_offsets: Option<&[u32]>,
    solid_offsets: Option<&[u32]>,
    geometry_type: &str,
) -> Result<Boundary<u32>> {
    let vertices = copy_vertex_indices(vertex_indices);

    let boundary = match parse_geometry_type(geometry_type)? {
        GeometryType::MultiPoint => {
            // SAFETY: MultiPoint has no higher-order offsets, so the owned vertex buffer is already canonical.
            unsafe { Boundary::from_parts_unchecked(vertices, vec![], vec![], vec![], vec![]) }
        }
        GeometryType::MultiLineString => {
            let lines = required_offsets(line_offsets, "line_offsets")?;
            validate_offsets(lines, vertices.len(), "line_offsets")?;
            // SAFETY: `line_offsets` was validated against the vertex buffer above.
            unsafe {
                Boundary::from_parts_unchecked(
                    vertices,
                    copy_vertex_indices(lines),
                    vec![],
                    vec![],
                    vec![],
                )
            }
        }
        GeometryType::MultiSurface | GeometryType::CompositeSurface => {
            let rings = required_offsets(ring_offsets, "ring_offsets")?;
            let surfaces = required_offsets(surface_offsets, "surface_offsets")?;
            validate_offsets(rings, vertices.len(), "ring_offsets")?;
            validate_offsets(surfaces, rings.len(), "surface_offsets")?;
            // SAFETY: ring and surface offsets were validated against their child buffers above.
            unsafe {
                Boundary::from_parts_unchecked(
                    vertices,
                    copy_vertex_indices(rings),
                    copy_vertex_indices(surfaces),
                    vec![],
                    vec![],
                )
            }
        }
        GeometryType::Solid => {
            let rings = required_offsets(ring_offsets, "ring_offsets")?;
            let surfaces = required_offsets(surface_offsets, "surface_offsets")?;
            let shells = required_offsets(shell_offsets, "shell_offsets")?;
            validate_offsets(rings, vertices.len(), "ring_offsets")?;
            validate_offsets(surfaces, rings.len(), "surface_offsets")?;
            validate_offsets(shells, surfaces.len(), "shell_offsets")?;
            // SAFETY: every offset buffer was validated against its child buffer above.
            unsafe {
                Boundary::from_parts_unchecked(
                    vertices,
                    copy_vertex_indices(rings),
                    copy_vertex_indices(surfaces),
                    copy_vertex_indices(shells),
                    vec![],
                )
            }
        }
        GeometryType::MultiSolid | GeometryType::CompositeSolid => {
            let rings = required_offsets(ring_offsets, "ring_offsets")?;
            let surfaces = required_offsets(surface_offsets, "surface_offsets")?;
            let shells = required_offsets(shell_offsets, "shell_offsets")?;
            let solids = required_offsets(solid_offsets, "solid_offsets")?;
            validate_offsets(rings, vertices.len(), "ring_offsets")?;
            validate_offsets(surfaces, rings.len(), "surface_offsets")?;
            validate_offsets(shells, surfaces.len(), "shell_offsets")?;
            validate_offsets(solids, shells.len(), "solid_offsets")?;
            // SAFETY: every offset buffer was validated against its child buffer above.
            unsafe {
                Boundary::from_parts_unchecked(
                    vertices,
                    copy_vertex_indices(rings),
                    copy_vertex_indices(surfaces),
                    copy_vertex_indices(shells),
                    copy_vertex_indices(solids),
                )
            }
        }
        GeometryType::GeometryInstance => {
            return Err(Error::Unsupported("geometry instances".to_string()));
        }
        _ => {
            return Err(Error::Unsupported("unsupported geometry type".to_string()));
        }
    };
    Ok(boundary)
}

pub(super) fn copy_vertex_indices(values: &[u32]) -> Vec<cityjson::v2_0::VertexIndex<u32>> {
    values
        .iter()
        .copied()
        .map(cityjson::v2_0::VertexIndex::new)
        .collect()
}

pub(super) fn required_offsets<'a>(value: Option<&'a [u32]>, name: &str) -> Result<&'a [u32]> {
    value.ok_or_else(|| Error::Conversion(format!("missing required {name}")))
}

pub(super) fn offset_to_usize(value: u32, child_len: usize, name: &str) -> Result<usize> {
    let offset = usize::try_from(value)
        .map_err(|_| Error::Conversion(format!("{name} value {value} does not fit into usize")))?;
    if offset > child_len {
        return Err(Error::Conversion(format!(
            "{name} value {value} exceeds child length {child_len}"
        )));
    }
    Ok(offset)
}

pub(super) fn offset_end(
    offsets: &[u32],
    index: usize,
    child_len: usize,
    name: &str,
) -> Result<usize> {
    match offsets.get(index + 1).copied() {
        Some(next) => offset_to_usize(next, child_len, name),
        None => Ok(child_len),
    }
}

pub(super) fn validate_offsets(offsets: &[u32], child_len: usize, name: &str) -> Result<()> {
    if let Some(first) = offsets.first()
        && *first != 0
    {
        return Err(Error::Conversion(format!(
            "{name} must start at zero, found {first}"
        )));
    }

    let mut previous = 0_u32;
    for offset in offsets {
        if *offset < previous {
            return Err(Error::Conversion(format!(
                "{name} must be monotonic, found {offset} after {previous}"
            )));
        }
        let _ = offset_to_usize(*offset, child_len, name)?;
        previous = *offset;
    }
    Ok(())
}

pub(super) fn surface_count(row: &impl BoundaryPayloadView) -> usize {
    row.surface_offsets().map_or(0, <[u32]>::len)
}

pub(super) fn template_surface_count(row: &impl BoundaryPayloadView) -> usize {
    row.surface_offsets().map_or(0, <[u32]>::len)
}

pub(super) fn has_projection_field(specs: Option<&ProjectedStructSpec>, name: &str) -> bool {
    specs.is_some_and(|specs| specs.fields.iter().any(|spec| spec.name == name))
}

pub(super) fn parse_geometry_type(value: &str) -> Result<GeometryType> {
    value.parse().map_err(Error::from)
}

pub(super) fn parse_lod(value: &str) -> Result<LoD> {
    Ok(match value {
        "0" => LoD::LoD0,
        "0.0" => LoD::LoD0_0,
        "0.1" => LoD::LoD0_1,
        "0.2" => LoD::LoD0_2,
        "0.3" => LoD::LoD0_3,
        "1" => LoD::LoD1,
        "1.0" => LoD::LoD1_0,
        "1.1" => LoD::LoD1_1,
        "1.2" => LoD::LoD1_2,
        "1.3" => LoD::LoD1_3,
        "2" => LoD::LoD2,
        "2.0" => LoD::LoD2_0,
        "2.1" => LoD::LoD2_1,
        "2.2" => LoD::LoD2_2,
        "2.3" => LoD::LoD2_3,
        "3" => LoD::LoD3,
        "3.0" => LoD::LoD3_0,
        "3.1" => LoD::LoD3_1,
        "3.2" => LoD::LoD3_2,
        "3.3" => LoD::LoD3_3,
        other => {
            return Err(Error::Conversion(format!("unsupported lod string {other}")));
        }
    })
}
