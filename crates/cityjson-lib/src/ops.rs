use std::collections::{BTreeSet, HashMap};

use crate::cityjson::resources::storage::OwnedStringStorage;
use crate::cityjson::v2_0::attributes::Attributes;
use crate::cityjson::v2_0::geometry::{
    Geometry, GeometryType, StoredGeometryInstance, StoredGeometryParts,
};
use crate::cityjson::v2_0::metadata::BBox;
use crate::cityjson::v2_0::{
    CityObject, CityObjectIdentifier, MaterialMap, Metadata, SemanticMap, TextureMap, VertexIndex,
};
use crate::cityjson::{
    CityModelType,
    prelude::{
        CityObjectHandle, GeometryHandle, GeometryTemplateHandle, MaterialHandle, SemanticHandle,
        TextureHandle,
    },
    v2_0::Extensions,
};
use crate::{CityModel, Error, Result};

type OwnedMetadata = Metadata<OwnedStringStorage>;
type OwnedExtensions = Extensions<OwnedStringStorage>;
type OwnedCityObject = CityObject<OwnedStringStorage>;
type OwnedGeometry = Geometry<u32, OwnedStringStorage>;

fn import_error(message: impl Into<String>) -> Error {
    Error::Import(message.into())
}

fn unsupported(message: &'static str) -> Error {
    Error::UnsupportedFeature(message.to_string())
}

fn same_transform(target: &CityModel, source: &CityModel) -> bool {
    match source.transform() {
        None => true,
        Some(source_transform) => target.transform() == Some(source_transform),
    }
}

fn append_kind_compatible(target_kind: CityModelType, source_kind: CityModelType) -> bool {
    target_kind == source_kind
        || (target_kind == CityModelType::CityJSON && source_kind == CityModelType::CityJSONFeature)
}

fn union_bbox(lhs: BBox, rhs: BBox) -> BBox {
    BBox::new(
        lhs.min_x().min(rhs.min_x()),
        lhs.min_y().min(rhs.min_y()),
        lhs.min_z().min(rhs.min_z()),
        lhs.max_x().max(rhs.max_x()),
        lhs.max_y().max(rhs.max_y()),
        lhs.max_z().max(rhs.max_z()),
    )
}

fn merge_attributes(
    target: &mut Attributes<OwnedStringStorage>,
    source: &Attributes<OwnedStringStorage>,
) {
    for (key, value) in source.iter() {
        target.insert(key.clone(), value.clone());
    }
}

fn merge_cityobject_extent(target: &mut OwnedCityObject, source: &OwnedCityObject) {
    match (
        target.geographical_extent().copied(),
        source.geographical_extent().copied(),
    ) {
        (None, Some(extent)) => target.set_geographical_extent(Some(extent)),
        (Some(lhs), Some(rhs)) if lhs != rhs => {
            target.set_geographical_extent(Some(union_bbox(lhs, rhs)))
        }
        _ => {}
    }
}

fn merge_metadata(target: &mut OwnedMetadata, source: &OwnedMetadata) {
    if target.geographical_extent().is_none()
        && let Some(extent) = source.geographical_extent().copied()
    {
        target.set_geographical_extent(extent);
    } else if let (Some(lhs), Some(rhs)) = (
        target.geographical_extent().copied(),
        source.geographical_extent().copied(),
    ) && lhs != rhs
    {
        target.set_geographical_extent(union_bbox(lhs, rhs));
    }

    if target.identifier().is_none()
        && let Some(identifier) = source.identifier().cloned()
    {
        target.set_identifier(identifier);
    }

    if target.reference_date().is_none()
        && let Some(date) = source.reference_date().cloned()
    {
        target.set_reference_date(date);
    }

    if target.reference_system().is_none()
        && let Some(crs) = source.reference_system().cloned()
    {
        target.set_reference_system(crs);
    }

    if target.title().is_none()
        && let Some(title) = source.title()
    {
        target.set_title(title.to_owned());
    }

    if target.point_of_contact().is_none()
        && let Some(contact) = source.point_of_contact().cloned()
    {
        target.set_point_of_contact(Some(contact));
    }

    if let Some(extra) = source.extra() {
        let target_extra = target.extra_mut();
        for (key, value) in extra.iter() {
            target_extra.insert(key.clone(), value.clone());
        }
    }
}

fn merge_root_extensions(target: &mut OwnedExtensions, source: &OwnedExtensions) {
    for extension in source {
        target.add(extension.clone());
    }
}

fn remap_vertex_indices(
    boundary: &crate::cityjson::v2_0::boundary::Boundary<u32>,
    vertex_map: &[VertexIndex<u32>],
) -> Result<crate::cityjson::v2_0::boundary::Boundary<u32>> {
    let mut boundary = boundary.clone();
    let remapped = boundary.vertices().iter().map(|index| {
        vertex_map
            .get(index.to_usize())
            .copied()
            .ok_or_else(|| import_error(format!("vertex index {} is out of range", index.value())))
    });
    boundary.set_vertices_from_iter(remapped.collect::<Result<Vec<_>>>()?);
    Ok(boundary)
}

fn remap_texture_map(
    map: &crate::cityjson::v2_0::geometry::TextureMapView<'_, u32>,
    uv_map: &[VertexIndex<u32>],
    texture_map: &HashMap<TextureHandle, TextureHandle>,
) -> Result<TextureMap<u32>> {
    let mut remapped = TextureMap::new();

    for vertex in map.vertices() {
        let mapped = vertex
            .map(|index| {
                uv_map.get(index.to_usize()).copied().ok_or_else(|| {
                    import_error(format!("uv vertex index {} is out of range", index.value()))
                })
            })
            .transpose()?;
        remapped.add_vertex(mapped);
    }

    for ring in map.rings() {
        remapped.add_ring(*ring);
    }

    for texture in map.ring_textures() {
        remapped.add_ring_texture(
            texture.map(|handle| texture_map.get(&handle).copied().unwrap_or(handle)),
        );
    }

    Ok(remapped)
}

fn remap_material_map<'a, I, J, K>(
    points: I,
    linestrings: J,
    surfaces: K,
    material_map: &HashMap<MaterialHandle, MaterialHandle>,
) -> MaterialMap<u32>
where
    I: IntoIterator<Item = &'a Option<MaterialHandle>>,
    J: IntoIterator<Item = &'a Option<MaterialHandle>>,
    K: IntoIterator<Item = &'a Option<MaterialHandle>>,
{
    let mut remapped = MaterialMap::new();

    for item in points {
        remapped.add_point(match item {
            Some(handle) => Some(material_map.get(handle).copied().unwrap_or(*handle)),
            None => None,
        });
    }
    for item in linestrings {
        remapped.add_linestring(match item {
            Some(handle) => Some(material_map.get(handle).copied().unwrap_or(*handle)),
            None => None,
        });
    }
    for item in surfaces {
        remapped.add_surface(match item {
            Some(handle) => Some(material_map.get(handle).copied().unwrap_or(*handle)),
            None => None,
        });
    }

    remapped
}

fn remap_semantic_map<'a, I, J, K>(
    points: I,
    linestrings: J,
    surfaces: K,
    semantic_map: &HashMap<SemanticHandle, SemanticHandle>,
) -> SemanticMap<u32>
where
    I: IntoIterator<Item = &'a Option<SemanticHandle>>,
    J: IntoIterator<Item = &'a Option<SemanticHandle>>,
    K: IntoIterator<Item = &'a Option<SemanticHandle>>,
{
    let mut remapped = SemanticMap::new();

    for item in points {
        remapped.add_point(match item {
            Some(handle) => Some(semantic_map.get(handle).copied().unwrap_or(*handle)),
            None => None,
        });
    }
    for item in linestrings {
        remapped.add_linestring(match item {
            Some(handle) => Some(semantic_map.get(handle).copied().unwrap_or(*handle)),
            None => None,
        });
    }
    for item in surfaces {
        remapped.add_surface(match item {
            Some(handle) => Some(semantic_map.get(handle).copied().unwrap_or(*handle)),
            None => None,
        });
    }

    remapped
}

fn remap_geometry(
    geometry: &OwnedGeometry,
    vertex_map: &[VertexIndex<u32>],
    template_map: &HashMap<GeometryTemplateHandle, GeometryTemplateHandle>,
    semantic_map: &HashMap<SemanticHandle, SemanticHandle>,
    material_map: &HashMap<MaterialHandle, MaterialHandle>,
    texture_map: &HashMap<TextureHandle, TextureHandle>,
    uv_map: &[VertexIndex<u32>],
) -> Result<OwnedGeometry> {
    let stored_parts = if let Some(instance) = geometry.instance() {
        let template = template_map
            .get(&instance.template())
            .copied()
            .ok_or_else(|| {
                import_error(format!(
                    "missing remap for geometry template {}",
                    instance.template()
                ))
            })?;
        Geometry::from_stored_parts(StoredGeometryParts {
            type_geometry: GeometryType::GeometryInstance,
            lod: None,
            boundaries: None,
            semantics: None,
            materials: None,
            textures: None,
            instance: Some(StoredGeometryInstance {
                template,
                reference_point: *vertex_map
                    .get(instance.reference_point().to_usize())
                    .ok_or_else(|| {
                        import_error(format!(
                            "vertex index {} is out of range",
                            instance.reference_point().value()
                        ))
                    })?,
                transformation: instance.transformation(),
            }),
        })
    } else {
        let boundaries = geometry
            .boundaries()
            .map(|boundary| remap_vertex_indices(boundary, vertex_map))
            .transpose()?;

        let semantics = geometry.semantics().map(|theme| {
            let points = theme.points();
            let linestrings = theme.linestrings();
            let surfaces = theme.surfaces();
            remap_semantic_map(
                points.iter(),
                linestrings.iter(),
                surfaces.iter(),
                semantic_map,
            )
        });

        let materials = geometry.materials().map(|themes| {
            themes
                .iter()
                .map(|(name, theme)| {
                    let points = theme.points();
                    let linestrings = theme.linestrings();
                    let surfaces = theme.surfaces();
                    (
                        name.clone(),
                        remap_material_map(
                            points.iter(),
                            linestrings.iter(),
                            surfaces.iter(),
                            material_map,
                        ),
                    )
                })
                .collect::<Vec<_>>()
        });

        let textures = geometry
            .textures()
            .map(|themes| {
                themes
                    .iter()
                    .map(|(name, theme)| {
                        remap_texture_map(&theme, uv_map, texture_map)
                            .map(|map| (name.clone(), map))
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?;

        Geometry::from_stored_parts(StoredGeometryParts {
            type_geometry: *geometry.type_geometry(),
            lod: geometry.lod().copied(),
            boundaries,
            semantics,
            materials,
            textures,
            instance: None,
        })
    };

    Ok(stored_parts)
}

fn append_vertices(target: &mut CityModel, source: &CityModel) -> Result<Vec<VertexIndex<u32>>> {
    let mut map = Vec::with_capacity(source.vertices().len());
    for vertex in source.vertices().as_slice() {
        map.push(target.add_vertex(*vertex)?);
    }
    Ok(map)
}

fn append_template_vertices(
    target: &mut CityModel,
    source: &CityModel,
) -> Result<Vec<VertexIndex<u32>>> {
    let mut map = Vec::with_capacity(source.template_vertices().len());
    for vertex in source.template_vertices().as_slice() {
        map.push(target.add_template_vertex(*vertex)?);
    }
    Ok(map)
}

fn append_uv_vertices(target: &mut CityModel, source: &CityModel) -> Result<Vec<VertexIndex<u32>>> {
    let mut map = Vec::with_capacity(source.vertices_texture().len());
    for uv in source.vertices_texture().as_slice() {
        map.push(target.add_uv_coordinate((*uv).clone())?);
    }
    Ok(map)
}

fn append_semantics(
    target: &mut CityModel,
    source: &CityModel,
) -> Result<HashMap<SemanticHandle, SemanticHandle>> {
    let mut map = HashMap::with_capacity(source.semantic_count());
    for (handle, semantic) in source.iter_semantics() {
        map.insert(handle, target.add_semantic(semantic.clone())?);
    }
    Ok(map)
}

fn append_materials(
    target: &mut CityModel,
    source: &CityModel,
) -> Result<HashMap<MaterialHandle, MaterialHandle>> {
    let mut map = HashMap::with_capacity(source.material_count());
    for (handle, material) in source.iter_materials() {
        map.insert(handle, target.add_material(material.clone())?);
    }
    Ok(map)
}

fn append_textures(
    target: &mut CityModel,
    source: &CityModel,
) -> Result<HashMap<TextureHandle, TextureHandle>> {
    let mut map = HashMap::with_capacity(source.texture_count());
    for (handle, texture) in source.iter_textures() {
        map.insert(handle, target.add_texture(texture.clone())?);
    }
    Ok(map)
}

fn append_geometry_templates(
    target: &mut CityModel,
    source: &CityModel,
    template_vertex_map: &[VertexIndex<u32>],
    template_map: &HashMap<GeometryTemplateHandle, GeometryTemplateHandle>,
    semantic_map: &HashMap<SemanticHandle, SemanticHandle>,
    material_map: &HashMap<MaterialHandle, MaterialHandle>,
    texture_map: &HashMap<TextureHandle, TextureHandle>,
    uv_map: &[VertexIndex<u32>],
) -> Result<HashMap<GeometryTemplateHandle, GeometryTemplateHandle>> {
    let mut map = HashMap::with_capacity(source.geometry_template_count());
    for (handle, geometry) in source.iter_geometry_templates() {
        let remapped = remap_geometry(
            geometry,
            template_vertex_map,
            template_map,
            semantic_map,
            material_map,
            texture_map,
            uv_map,
        )?;
        map.insert(handle, target.add_geometry_template(remapped)?);
    }
    Ok(map)
}

fn append_geometries(
    target: &mut CityModel,
    source: &CityModel,
    vertex_map: &[VertexIndex<u32>],
    template_map: &HashMap<GeometryTemplateHandle, GeometryTemplateHandle>,
    semantic_map: &HashMap<SemanticHandle, SemanticHandle>,
    material_map: &HashMap<MaterialHandle, MaterialHandle>,
    texture_map: &HashMap<TextureHandle, TextureHandle>,
    uv_map: &[VertexIndex<u32>],
) -> Result<HashMap<GeometryHandle, GeometryHandle>> {
    let mut map = HashMap::with_capacity(source.geometry_count());
    for (handle, geometry) in source.iter_geometries() {
        let remapped = remap_geometry(
            geometry,
            vertex_map,
            template_map,
            semantic_map,
            material_map,
            texture_map,
            uv_map,
        )?;
        map.insert(handle, target.add_geometry(remapped)?);
    }
    Ok(map)
}

fn merge_cityobject(
    target: &mut OwnedCityObject,
    source: &OwnedCityObject,
    cityobject_map: &HashMap<CityObjectHandle, CityObjectHandle>,
    geometry_map: &HashMap<GeometryHandle, GeometryHandle>,
) -> Result<()> {
    if target.type_cityobject() != source.type_cityobject() {
        return Err(import_error(format!(
            "conflicting CityObject types for '{}'",
            target.id()
        )));
    }

    if let Some(attributes) = source.attributes() {
        merge_attributes(target.attributes_mut(), attributes);
    }
    merge_cityobject_extent(target, source);

    if let Some(extra) = source.extra() {
        let target_extra = target.extra_mut();
        for (key, value) in extra.iter() {
            target_extra.insert(key.clone(), value.clone());
        }
    }

    if let Some(geometry_handles) = source.geometry() {
        let mut target_geometry = target
            .geometry()
            .map(|items| items.to_vec())
            .unwrap_or_default();
        for geometry in geometry_handles {
            let mapped = geometry_map.get(geometry).copied().ok_or_else(|| {
                import_error(format!(
                    "missing remap for geometry {}",
                    geometry.raw_parts().0
                ))
            })?;
            if !target_geometry.contains(&mapped) {
                target.add_geometry(mapped);
                target_geometry.push(mapped);
            }
        }
    }

    if let Some(children) = source.children() {
        let mut existing = target
            .children()
            .map(|items| items.to_vec())
            .unwrap_or_default();
        for child in children {
            let mapped = cityobject_map.get(child).copied().ok_or_else(|| {
                import_error(format!(
                    "missing remap for cityobject {}",
                    child.raw_parts().0
                ))
            })?;
            if !existing.contains(&mapped) {
                target.add_child(mapped);
                existing.push(mapped);
            }
        }
    }

    if let Some(parents) = source.parents() {
        let mut existing = target
            .parents()
            .map(|items| items.to_vec())
            .unwrap_or_default();
        for parent in parents {
            let mapped = cityobject_map.get(parent).copied().ok_or_else(|| {
                import_error(format!(
                    "missing remap for cityobject {}",
                    parent.raw_parts().0
                ))
            })?;
            if !existing.contains(&mapped) {
                target.add_parent(mapped);
                existing.push(mapped);
            }
        }
    }

    Ok(())
}

fn merge_one(target: &mut CityModel, source: &CityModel) -> Result<()> {
    if !same_transform(target, source) {
        return Err(unsupported(
            "model merge currently requires identical transform objects",
        ));
    }

    if !append_kind_compatible(target.type_citymodel(), source.type_citymodel()) {
        return Err(import_error(
            "model merge currently requires compatible root types",
        ));
    }

    if target.metadata().is_none() {
        if let Some(metadata) = source.metadata() {
            *target.metadata_mut() = metadata.clone();
        }
    } else if let Some(source_metadata) = source.metadata() {
        merge_metadata(target.metadata_mut(), source_metadata);
    }

    if target.extra().is_none() {
        if let Some(extra) = source.extra() {
            *target.extra_mut() = extra.clone();
        }
    } else if let Some(extra) = source.extra() {
        let target_extra = target.extra_mut();
        for (key, value) in extra.iter() {
            target_extra.insert(key.clone(), value.clone());
        }
    }

    if target.extensions().is_none() {
        if let Some(extensions) = source.extensions() {
            *target.extensions_mut() = extensions.clone();
        }
    } else if let Some(extensions) = source.extensions() {
        merge_root_extensions(target.extensions_mut(), extensions);
    }

    if target.transform().is_none() {
        if let Some(transform) = source.transform() {
            *target.transform_mut() = transform.clone();
        }
    }

    if target.default_material_theme().is_none()
        && let Some(theme) = source.default_material_theme().cloned()
    {
        target.set_default_material_theme(Some(theme));
    }

    if target.default_texture_theme().is_none()
        && let Some(theme) = source.default_texture_theme().cloned()
    {
        target.set_default_texture_theme(Some(theme));
    }

    let vertex_map = append_vertices(target, source)?;
    let template_vertex_map = append_template_vertices(target, source)?;
    let uv_map = append_uv_vertices(target, source)?;
    let semantic_map = append_semantics(target, source)?;
    let material_map = append_materials(target, source)?;
    let texture_map = append_textures(target, source)?;
    let empty_template_map: HashMap<GeometryTemplateHandle, GeometryTemplateHandle> =
        HashMap::new();
    let template_map = append_geometry_templates(
        target,
        source,
        &template_vertex_map,
        &empty_template_map,
        &semantic_map,
        &material_map,
        &texture_map,
        &uv_map,
    )?;
    let geometry_map = append_geometries(
        target,
        source,
        &vertex_map,
        &template_map,
        &semantic_map,
        &material_map,
        &texture_map,
        &uv_map,
    )?;

    let mut cityobject_map = HashMap::with_capacity(source.cityobjects().len());
    for (handle, source_cityobject) in source.cityobjects().iter() {
        if let Some(existing) = target
            .cityobjects()
            .iter()
            .find(|(_, cityobject)| cityobject.id() == source_cityobject.id())
            .map(|(handle, _)| handle)
        {
            cityobject_map.insert(handle, existing);
            continue;
        }

        let placeholder = CityObject::new(
            CityObjectIdentifier::new(source_cityobject.id().to_owned()),
            source_cityobject.type_cityobject().clone(),
        );
        let new_handle = target.cityobjects_mut().add(placeholder)?;
        cityobject_map.insert(handle, new_handle);
    }

    if target.id().is_none()
        && let Some(source_id) = source.id()
        && let Some(mapped) = cityobject_map.get(&source_id).copied()
    {
        target.set_id(Some(mapped));
    }

    for (handle, source_cityobject) in source.cityobjects().iter() {
        let target_handle = cityobject_map.get(&handle).copied().ok_or_else(|| {
            import_error(format!(
                "missing remap for cityobject {}",
                source_cityobject.id()
            ))
        })?;
        let target_cityobject =
            target
                .cityobjects_mut()
                .get_mut(target_handle)
                .ok_or_else(|| {
                    import_error(format!(
                        "missing target cityobject for {}",
                        source_cityobject.id()
                    ))
                })?;
        merge_cityobject(
            target_cityobject,
            source_cityobject,
            &cityobject_map,
            &geometry_map,
        )?;
    }

    Ok(())
}

pub fn cleanup(model: &CityModel) -> Result<CityModel> {
    cityjson_json::cleanup(model).map_err(Error::from)
}

pub fn subset<'a, I>(model: &CityModel, cityobject_ids: I, exclude: bool) -> Result<CityModel>
where
    I: IntoIterator<Item = &'a str>,
{
    let ids = cityobject_ids
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if ids.is_empty() {
        return Err(import_error(
            "subset requires at least one CityObject identifier",
        ));
    }

    let id_to_handle = model
        .cityobjects()
        .iter()
        .map(|(handle, cityobject)| (cityobject.id().to_owned(), handle))
        .collect::<HashMap<_, _>>();

    let mut selected = BTreeSet::new();
    let mut stack = Vec::new();
    let mut matched_any = false;

    for id in &ids {
        if let Some(handle) = id_to_handle.get(id).copied() {
            matched_any = true;
            stack.push(handle);
        }
    }

    while let Some(handle) = stack.pop() {
        let cityobject = model.cityobjects().get(handle).ok_or_else(|| {
            import_error(format!(
                "missing CityObject handle in subset traversal: {handle:?}"
            ))
        })?;
        if !selected.insert(cityobject.id().to_owned()) {
            continue;
        }

        if let Some(children) = cityobject.children() {
            for child in children {
                stack.push(*child);
            }
        }
    }

    if !matched_any {
        return Err(import_error("subset selection matched no CityObjects"));
    }

    if exclude {
        let excluded = selected;
        selected = model
            .cityobjects()
            .iter()
            .map(|(_, cityobject)| cityobject.id().to_owned())
            .filter(|id| !excluded.contains(id))
            .collect();
    }

    let mut result = model.clone();
    result.clear_cityobjects();

    let mut id_to_new_handle = HashMap::with_capacity(selected.len());
    for (_, cityobject) in model.cityobjects().iter() {
        if !selected.contains(cityobject.id()) {
            continue;
        }

        let mut cloned = cityobject.clone();
        cloned.clear_children();
        cloned.clear_parents();
        let handle = result.cityobjects_mut().add(cloned)?;
        id_to_new_handle.insert(cityobject.id().to_owned(), handle);
    }

    for (_, cityobject) in model.cityobjects().iter() {
        if !selected.contains(cityobject.id()) {
            continue;
        }

        let target_handle = *id_to_new_handle.get(cityobject.id()).ok_or_else(|| {
            import_error(format!("missing remap for CityObject {}", cityobject.id()))
        })?;
        let target = result
            .cityobjects_mut()
            .get_mut(target_handle)
            .ok_or_else(|| {
                import_error(format!("missing target CityObject {}", cityobject.id()))
            })?;

        if let Some(children) = cityobject.children() {
            for child in children {
                let child_id = model
                    .cityobjects()
                    .get(*child)
                    .ok_or_else(|| {
                        import_error(format!("missing child CityObject handle {child:?}"))
                    })?
                    .id()
                    .to_owned();
                if let Some(mapped) = id_to_new_handle.get(&child_id).copied() {
                    target.add_child(mapped);
                }
            }
        }

        if let Some(parents) = cityobject.parents() {
            for parent in parents {
                let parent_id = model
                    .cityobjects()
                    .get(*parent)
                    .ok_or_else(|| {
                        import_error(format!("missing parent CityObject handle {parent:?}"))
                    })?
                    .id()
                    .to_owned();
                if let Some(mapped) = id_to_new_handle.get(&parent_id).copied() {
                    target.add_parent(mapped);
                }
            }
        }
    }

    if let Some(root) = model.id() {
        let root_id = model
            .cityobjects()
            .get(root)
            .ok_or_else(|| import_error("feature root references a missing CityObject"))?
            .id()
            .to_owned();
        result.set_id(id_to_new_handle.get(&root_id).copied());
    }

    Ok(result)
}

pub fn append(target: &mut CityModel, source: &CityModel) -> Result<()> {
    merge_one(target, source)
}

pub fn merge<I>(models: I) -> Result<CityModel>
where
    I: IntoIterator<Item = CityModel>,
{
    let mut models = models.into_iter();
    let Some(mut merged) = models.next() else {
        return Err(import_error("merge requires at least one model"));
    };

    for model in models {
        merge_one(&mut merged, &model)?;
    }

    Ok(merged)
}
