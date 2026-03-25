use std::collections::HashMap;

use crate::de::attributes::attribute_map;
use crate::de::parse::ParseStringStorage;
use crate::de::sections::{
    MultiLineStringBoundary, MultiPointBoundary, MultiSolidBoundary, MultiSurfaceBoundary,
    RawAssignment, RawGeometry, RawMaterialTheme, RawSemantics, SolidBoundary,
};
use crate::de::validation::{parse_lod, parse_semantic_type};
use crate::errors::{Error, Result};
use cityjson::resources::handles::{
    GeometryHandle, GeometryTemplateHandle, MaterialHandle, SemanticHandle, TextureHandle,
};
use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{
    AffineTransform3D, CityModel, GeometryDraft, LineStringDraft, PointDraft, RingDraft, Semantic,
    SemanticType, ShellDraft, SolidDraft, SurfaceDraft, ThemeName, VertexIndex,
};

// ---------------------------------------------------------------------------
// Resource registry (material / texture / template handles)
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub(crate) struct GeometryResources {
    pub(crate) materials: Vec<MaterialHandle>,
    pub(crate) textures: Vec<TextureHandle>,
    pub(crate) templates: Vec<GeometryTemplateHandle>,
}

// ---------------------------------------------------------------------------
// Internal mappings built during geometry import
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(crate) struct RingTextureAssignment {
    pub(crate) texture: TextureHandle,
    pub(crate) uvs: Vec<VertexIndex<u32>>,
}

#[derive(Debug, Default)]
struct SurfaceMappings<'de> {
    semantics: Vec<Option<SemanticHandle>>,
    materials: Vec<(&'de str, Vec<Option<MaterialHandle>>)>,
    textures: Vec<(&'de str, Vec<Option<RingTextureAssignment>>)>,
}

// ---------------------------------------------------------------------------
// Top-level geometry dispatch
// ---------------------------------------------------------------------------

pub(crate) fn import_geometry<'de, SS>(
    raw: RawGeometry<'de>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<GeometryHandle>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    match raw {
        RawGeometry::MultiPoint {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => import_multi_point(
            lod,
            boundaries,
            semantics.as_ref(),
            material.as_ref(),
            texture.as_ref(),
            model,
            resources,
        ),

        RawGeometry::MultiLineString {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => import_multi_line_string(
            lod,
            boundaries,
            semantics.as_ref(),
            material.as_ref(),
            texture.as_ref(),
            model,
            resources,
        ),

        RawGeometry::MultiSurface {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => import_multi_surface(
            lod,
            boundaries,
            false,
            semantics.as_ref(),
            material,
            texture,
            model,
            resources,
        ),

        RawGeometry::CompositeSurface {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => import_multi_surface(
            lod,
            boundaries,
            true,
            semantics.as_ref(),
            material,
            texture,
            model,
            resources,
        ),

        RawGeometry::Solid {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => import_solid(
            lod,
            boundaries,
            semantics.as_ref(),
            material,
            texture,
            model,
            resources,
        ),

        RawGeometry::MultiSolid {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => import_multi_solid(
            lod,
            boundaries,
            false,
            semantics.as_ref(),
            material,
            texture,
            model,
            resources,
        ),

        RawGeometry::CompositeSolid {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => import_multi_solid(
            lod,
            boundaries,
            true,
            semantics.as_ref(),
            material,
            texture,
            model,
            resources,
        ),

        RawGeometry::GeometryInstance {
            lod,
            template,
            boundaries,
            transformation_matrix,
        } => import_geometry_instance(
            lod,
            template,
            boundaries.as_deref(),
            transformation_matrix,
            model,
            resources,
        ),
    }
}

/// Import a geometry as a template (not a regular city object geometry).
///
/// Template geometries cannot be `GeometryInstance`.
pub(crate) fn import_template_geometry<'de, SS>(
    raw: RawGeometry<'de>,
    model: &mut CityModel<u32, SS>,
) -> Result<GeometryTemplateHandle>
where
    SS: ParseStringStorage<'de>,
{
    match raw {
        RawGeometry::GeometryInstance { .. } => Err(Error::UnsupportedFeature(
            "GeometryInstance cannot be used as a geometry template",
        )),
        RawGeometry::MultiPoint {
            lod, boundaries, ..
        } => GeometryDraft::multi_point(parse_lod(lod)?, boundaries.into_iter().map(point_draft))
            .insert_template_into(model)
            .map_err(Error::from),
        RawGeometry::MultiLineString {
            lod, boundaries, ..
        } => {
            let linestrings = boundaries
                .into_iter()
                .map(|ls| LineStringDraft::new(ls.into_iter().map(VertexIndex::new)));
            GeometryDraft::multi_line_string(parse_lod(lod)?, linestrings)
                .insert_template_into(model)
                .map_err(Error::from)
        }
        RawGeometry::MultiSurface {
            lod, boundaries, ..
        } => {
            let surfaces = boundaries
                .into_iter()
                .map(surface_draft::<SS>)
                .collect::<Result<Vec<_>>>()?;
            GeometryDraft::multi_surface(parse_lod(lod)?, surfaces)
                .insert_template_into(model)
                .map_err(Error::from)
        }
        RawGeometry::CompositeSurface {
            lod, boundaries, ..
        } => {
            let surfaces = boundaries
                .into_iter()
                .map(surface_draft::<SS>)
                .collect::<Result<Vec<_>>>()?;
            GeometryDraft::composite_surface(parse_lod(lod)?, surfaces)
                .insert_template_into(model)
                .map_err(Error::from)
        }
        RawGeometry::Solid {
            lod, boundaries, ..
        } => {
            let mut shells = boundaries.into_iter().map(shell_draft::<SS>);
            let outer = shells.next().transpose()?.ok_or_else(|| {
                Error::InvalidValue("Solid geometry requires at least one shell".to_owned())
            })?;
            let inners = shells.collect::<Result<Vec<_>>>()?;
            GeometryDraft::solid(parse_lod(lod)?, outer, inners)
                .insert_template_into(model)
                .map_err(Error::from)
        }
        RawGeometry::MultiSolid {
            lod, boundaries, ..
        } => {
            let solids = boundaries
                .into_iter()
                .map(solid_draft::<SS>)
                .collect::<Result<Vec<_>>>()?;
            GeometryDraft::multi_solid(parse_lod(lod)?, solids)
                .insert_template_into(model)
                .map_err(Error::from)
        }
        RawGeometry::CompositeSolid {
            lod, boundaries, ..
        } => {
            let solids = boundaries
                .into_iter()
                .map(solid_draft::<SS>)
                .collect::<Result<Vec<_>>>()?;
            GeometryDraft::composite_solid(parse_lod(lod)?, solids)
                .insert_template_into(model)
                .map_err(Error::from)
        }
    }
}

// ---------------------------------------------------------------------------
// Geometry type importers
// ---------------------------------------------------------------------------

fn import_multi_point<'de, SS>(
    lod: Option<&'de str>,
    boundaries: MultiPointBoundary,
    semantics: Option<&RawSemantics<'de>>,
    material: Option<&HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<&HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    model: &mut CityModel<u32, SS>,
    _resources: &GeometryResources,
) -> Result<GeometryHandle>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    if material.is_some_and(|m| !m.is_empty()) {
        return Err(Error::UnsupportedFeature(
            "geometry material import is not supported for MultiPoint",
        ));
    }
    if texture.is_some_and(|t| !t.is_empty()) {
        return Err(Error::UnsupportedFeature(
            "geometry texture import is not supported for MultiPoint",
        ));
    }

    let semantic_handles = import_geometry_semantics::<SS>(semantics, model)?;
    let assignments = parse_point_assignments(semantics, &semantic_handles, boundaries.len());

    let points = boundaries.into_iter().enumerate().map(|(i, idx)| {
        let pt = PointDraft::new(VertexIndex::new(idx));
        if let Some(Some(sem)) = assignments.get(i) {
            pt.with_semantic(*sem)
        } else {
            pt
        }
    });

    GeometryDraft::multi_point(parse_lod(lod)?, points)
        .insert_into(model)
        .map_err(Error::from)
}

fn import_multi_line_string<'de, SS>(
    lod: Option<&'de str>,
    boundaries: MultiLineStringBoundary,
    semantics: Option<&RawSemantics<'de>>,
    material: Option<&HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<&HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    model: &mut CityModel<u32, SS>,
    _resources: &GeometryResources,
) -> Result<GeometryHandle>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    if material.is_some_and(|m| !m.is_empty()) {
        return Err(Error::UnsupportedFeature(
            "geometry material import is not supported for MultiLineString",
        ));
    }
    if texture.is_some_and(|t| !t.is_empty()) {
        return Err(Error::UnsupportedFeature(
            "geometry texture import is not supported for MultiLineString",
        ));
    }

    let semantic_handles = import_geometry_semantics::<SS>(semantics, model)?;
    let assignments = parse_linestring_assignments(semantics, &semantic_handles, boundaries.len());

    let linestrings = boundaries.into_iter().enumerate().map(|(i, ls)| {
        let ld = LineStringDraft::new(ls.into_iter().map(VertexIndex::new));
        if let Some(Some(sem)) = assignments.get(i) {
            ld.with_semantic(*sem)
        } else {
            ld
        }
    });

    GeometryDraft::multi_line_string(parse_lod(lod)?, linestrings)
        .insert_into(model)
        .map_err(Error::from)
}

#[allow(clippy::too_many_arguments)]
fn import_multi_surface<'de, SS>(
    lod: Option<&'de str>,
    boundaries: MultiSurfaceBoundary,
    is_composite: bool,
    semantics: Option<&RawSemantics<'de>>,
    material: Option<HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<GeometryHandle>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let mappings =
        parse_multi_surface_mappings(semantics, material, texture, &boundaries, model, resources)?;
    let mut surface_index = 0;
    let mut ring_index = 0;
    let surfaces = boundaries
        .into_iter()
        .map(|surface| {
            mapped_surface_draft::<SS>(surface, &mappings, &mut surface_index, &mut ring_index)
        })
        .collect::<Result<Vec<_>>>()?;
    let draft = if is_composite {
        GeometryDraft::composite_surface(parse_lod(lod)?, surfaces)
    } else {
        GeometryDraft::multi_surface(parse_lod(lod)?, surfaces)
    };
    draft.insert_into(model).map_err(Error::from)
}

fn import_solid<'de, SS>(
    lod: Option<&'de str>,
    boundaries: SolidBoundary,
    semantics: Option<&RawSemantics<'de>>,
    material: Option<HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<GeometryHandle>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let mappings =
        parse_solid_mappings(semantics, material, texture, &boundaries, model, resources)?;
    let mut surface_index = 0;
    let mut ring_index = 0;
    let mut shells = boundaries.into_iter().map(|shell| {
        mapped_shell_draft::<SS>(shell, &mappings, &mut surface_index, &mut ring_index)
    });
    let outer = shells.next().transpose()?.ok_or_else(|| {
        Error::InvalidValue("Solid geometry requires at least one shell".to_owned())
    })?;
    let inners = shells.collect::<Result<Vec<_>>>()?;
    GeometryDraft::solid(parse_lod(lod)?, outer, inners)
        .insert_into(model)
        .map_err(Error::from)
}

#[allow(clippy::too_many_arguments)]
fn import_multi_solid<'de, SS>(
    lod: Option<&'de str>,
    boundaries: MultiSolidBoundary,
    is_composite: bool,
    semantics: Option<&RawSemantics<'de>>,
    material: Option<HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<GeometryHandle>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let mappings =
        parse_multi_solid_mappings(semantics, material, texture, &boundaries, model, resources)?;
    let mut surface_index = 0;
    let mut ring_index = 0;
    let solids = boundaries
        .into_iter()
        .map(|solid| {
            mapped_solid_draft::<SS>(solid, &mappings, &mut surface_index, &mut ring_index)
        })
        .collect::<Result<Vec<_>>>()?;
    let draft = if is_composite {
        GeometryDraft::composite_solid(parse_lod(lod)?, solids)
    } else {
        GeometryDraft::multi_solid(parse_lod(lod)?, solids)
    };
    draft.insert_into(model).map_err(Error::from)
}

fn import_geometry_instance<'de, SS>(
    _lod: Option<&'de str>,
    template: Option<u32>,
    boundaries: Option<&[u32]>,
    transformation_matrix: Option<[f64; 16]>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<GeometryHandle>
where
    SS: ParseStringStorage<'de>,
{
    let template_idx = template.ok_or_else(|| {
        Error::InvalidValue("GeometryInstance is missing a template index".to_owned())
    })?;
    let template_handle = resources
        .templates
        .get(template_idx as usize)
        .copied()
        .ok_or_else(|| {
            Error::InvalidValue(format!("invalid geometry template index '{template_idx}'"))
        })?;

    let reference_point = boundaries
        .and_then(<[u32]>::first)
        .copied()
        .ok_or_else(|| {
            Error::InvalidValue(
                "GeometryInstance boundaries must contain a single reference-point index"
                    .to_owned(),
            )
        })?;

    GeometryDraft::instance(
        template_handle,
        VertexIndex::new(reference_point),
        transformation_matrix
            .map(AffineTransform3D::from)
            .unwrap_or_default(),
    )
    .insert_into(model)
    .map_err(Error::from)
}

// ---------------------------------------------------------------------------
// Semantic import
// ---------------------------------------------------------------------------

pub(crate) fn import_geometry_semantics<'de, SS>(
    raw: Option<&RawSemantics<'de>>,
    model: &mut CityModel<u32, SS>,
) -> Result<Vec<SemanticHandle>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };

    let mut pending_links: Vec<(Option<u64>, Vec<u64>)> = Vec::with_capacity(raw.surfaces.len());
    let mut handles = Vec::with_capacity(raw.surfaces.len());

    for surface in &raw.surfaces {
        let sem_type: SemanticType<SS> = parse_semantic_type::<SS>(surface.type_name)?;
        let mut semantic = Semantic::<SS>::new(sem_type);

        if !surface.attributes.is_empty() {
            // Remove known keys that are not attributes
            let attrs: HashMap<&'de str, _> = surface
                .attributes
                .iter()
                .filter(|(k, _)| **k != "type" && **k != "parent" && **k != "children")
                .map(|(k, v)| (*k, v))
                .collect::<HashMap<_, _>>();

            if !attrs.is_empty() {
                let attrs_cloned: HashMap<&'de str, _> = attrs
                    .into_iter()
                    .map(|(k, v)| (k, clone_raw_attribute(v)))
                    .collect();
                *semantic.attributes_mut() =
                    attribute_map::<SS>(attrs_cloned, "semantic attributes")?;
            }
        }

        handles.push(model.add_semantic(semantic).map_err(Error::from)?);
        pending_links.push((surface.parent, surface.children.clone()));
    }

    // Resolve parent/child links after all handles are known.
    for (index, (parent, children)) in pending_links.into_iter().enumerate() {
        let handle = handles[index];
        let semantic = model
            .get_semantic_mut(handle)
            .ok_or_else(|| Error::InvalidValue(format!("missing semantic handle {handle}")))?;
        if let Some(parent_index) = parent {
            if let Some(&parent_handle) = usize::try_from(parent_index)
                .ok()
                .and_then(|i| handles.get(i))
            {
                semantic.set_parent(parent_handle);
            }
        }
        if !children.is_empty() {
            let sem_children = semantic.children_mut();
            sem_children.reserve(children.len());
            for child_index in children {
                if let Some(&child_handle) = usize::try_from(child_index)
                    .ok()
                    .and_then(|i| handles.get(i))
                {
                    sem_children.push(child_handle);
                }
            }
        }
    }

    Ok(handles)
}

/// Clone a `RawAttribute` reference for building the attributes map.
///
/// This is needed because `import_geometry_semantics` borrows `raw.surfaces`
/// but needs to produce owned data for `attribute_map`.
fn clone_raw_attribute<'de>(
    attr: &crate::de::attributes::RawAttribute<'de>,
) -> crate::de::attributes::RawAttribute<'de> {
    use crate::de::attributes::RawAttribute;
    use std::borrow::Cow;
    match attr {
        RawAttribute::Null => RawAttribute::Null,
        RawAttribute::Bool(b) => RawAttribute::Bool(*b),
        RawAttribute::Number(n) => RawAttribute::Number(n.clone()),
        RawAttribute::String(cow) => RawAttribute::String(match cow {
            Cow::Borrowed(s) => Cow::Borrowed(s),
            Cow::Owned(s) => Cow::Owned(s.clone()),
        }),
        RawAttribute::Array(v) => RawAttribute::Array(v.iter().map(clone_raw_attribute).collect()),
        RawAttribute::Object(m) => RawAttribute::Object(
            m.iter()
                .map(|(k, v)| (*k, clone_raw_attribute(v)))
                .collect(),
        ),
    }
}

// ---------------------------------------------------------------------------
// Assignment helpers
// ---------------------------------------------------------------------------

fn flatten_assignment(raw: &RawAssignment, out: &mut Vec<Option<u64>>) {
    match raw {
        RawAssignment::Null => out.push(None),
        RawAssignment::Index(i) => out.push(Some(*i)),
        RawAssignment::Nested(vec) => {
            for child in vec {
                flatten_assignment(child, out);
            }
        }
    }
}

fn resolve_assignments<T: Copy>(
    raw: &RawAssignment,
    handles: &[T],
    expected_len: usize,
) -> Vec<Option<T>> {
    let mut indices = Vec::new();
    flatten_assignment(raw, &mut indices);
    indices.resize(expected_len, None);
    indices
        .into_iter()
        .map(|idx| {
            idx.and_then(|i| {
                usize::try_from(i)
                    .ok()
                    .and_then(|i| handles.get(i))
                    .copied()
            })
        })
        .collect()
}

fn parse_point_assignments(
    semantics: Option<&RawSemantics<'_>>,
    handles: &[SemanticHandle],
    expected_len: usize,
) -> Vec<Option<SemanticHandle>> {
    match semantics {
        None => vec![None; expected_len],
        Some(s) => resolve_assignments(&s.values, handles, expected_len),
    }
}

fn parse_linestring_assignments(
    semantics: Option<&RawSemantics<'_>>,
    handles: &[SemanticHandle],
    expected_len: usize,
) -> Vec<Option<SemanticHandle>> {
    match semantics {
        None => vec![None; expected_len],
        Some(s) => resolve_assignments(&s.values, handles, expected_len),
    }
}

fn parse_surface_scalar_assignments(
    semantics: Option<&RawSemantics<'_>>,
    handles: &[SemanticHandle],
    surface_count: usize,
) -> Vec<Option<SemanticHandle>> {
    match semantics {
        None => vec![None; surface_count],
        Some(s) => resolve_assignments(&s.values, handles, surface_count),
    }
}

// ---------------------------------------------------------------------------
// SurfaceMappings builders
// ---------------------------------------------------------------------------

fn parse_multi_surface_mappings<'de, SS>(
    semantics: Option<&RawSemantics<'de>>,
    material: Option<HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    boundaries: &MultiSurfaceBoundary,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<SurfaceMappings<'de>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let surface_count = boundaries.len();
    let semantic_handles = import_geometry_semantics::<SS>(semantics, model)?;
    Ok(SurfaceMappings {
        semantics: parse_surface_scalar_assignments(semantics, &semantic_handles, surface_count),
        materials: parse_material_themes(material, &resources.materials, surface_count)?,
        textures: parse_texture_themes(texture, |values| {
            let ring_count: usize = boundaries.iter().map(Vec::len).sum();
            parse_ring_texture_assignments(values, ring_count, resources)
        })?,
    })
}

fn parse_solid_mappings<'de, SS>(
    semantics: Option<&RawSemantics<'de>>,
    material: Option<HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    boundaries: &SolidBoundary,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<SurfaceMappings<'de>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let surface_count: usize = boundaries.iter().map(Vec::len).sum();
    let semantic_handles = import_geometry_semantics::<SS>(semantics, model)?;
    Ok(SurfaceMappings {
        semantics: parse_surface_scalar_assignments(semantics, &semantic_handles, surface_count),
        materials: parse_material_themes(material, &resources.materials, surface_count)?,
        textures: parse_texture_themes(texture, |values| {
            let ring_count: usize = boundaries
                .iter()
                .map(|shell| shell.iter().map(Vec::len).sum::<usize>())
                .sum();
            parse_ring_texture_assignments(values, ring_count, resources)
        })?,
    })
}

fn parse_multi_solid_mappings<'de, SS>(
    semantics: Option<&RawSemantics<'de>>,
    material: Option<HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    boundaries: &MultiSolidBoundary,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<SurfaceMappings<'de>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let surface_count: usize = boundaries
        .iter()
        .map(|solid| solid.iter().map(Vec::len).sum::<usize>())
        .sum();
    let semantic_handles = import_geometry_semantics::<SS>(semantics, model)?;
    Ok(SurfaceMappings {
        semantics: parse_surface_scalar_assignments(semantics, &semantic_handles, surface_count),
        materials: parse_material_themes(material, &resources.materials, surface_count)?,
        textures: parse_texture_themes(texture, |values| {
            let ring_count: usize = boundaries
                .iter()
                .map(|solid| {
                    solid
                        .iter()
                        .map(|shell| shell.iter().map(Vec::len).sum::<usize>())
                        .sum::<usize>()
                })
                .sum();
            parse_ring_texture_assignments(values, ring_count, resources)
        })?,
    })
}

// ---------------------------------------------------------------------------
// Material theme parsing
// ---------------------------------------------------------------------------

type MaterialThemes<'de> = Vec<(&'de str, Vec<Option<MaterialHandle>>)>;
type TextureThemes<'de> = Vec<(&'de str, Vec<Option<RingTextureAssignment>>)>;

fn parse_material_themes<'de>(
    material: Option<HashMap<&'de str, RawMaterialTheme>>,
    handles: &[MaterialHandle],
    surface_count: usize,
) -> Result<MaterialThemes<'de>> {
    let Some(material) = material else {
        return Ok(Vec::new());
    };
    let mut result = Vec::with_capacity(material.len());
    for (theme, entry) in material {
        let assignments = if let Some(single) = entry.value.as_ref() {
            let idx = match single {
                RawAssignment::Null => None,
                RawAssignment::Index(i) => usize::try_from(*i)
                    .ok()
                    .and_then(|i| handles.get(i))
                    .copied(),
                RawAssignment::Nested(_) => {
                    return Err(Error::InvalidValue(
                        "geometry material.value must be a scalar, not an array".to_owned(),
                    ))
                }
            };
            vec![idx; surface_count]
        } else if let Some(values) = entry.values.as_ref() {
            resolve_assignments(values, handles, surface_count)
        } else {
            return Err(Error::InvalidValue(format!(
                "geometry material theme '{theme}' must contain value or values"
            )));
        };
        result.push((theme, assignments));
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Texture theme parsing
// ---------------------------------------------------------------------------

fn parse_texture_themes<F>(
    texture: Option<HashMap<&str, crate::de::sections::RawTextureTheme>>,
    mut parse_values: F,
) -> Result<TextureThemes<'_>>
where
    F: FnMut(&serde_json::Value) -> Result<Vec<Option<RingTextureAssignment>>>,
{
    let Some(texture) = texture else {
        return Ok(Vec::new());
    };
    let mut result = Vec::with_capacity(texture.len());
    for (theme, entry) in texture {
        result.push((theme, parse_values(&entry.values)?));
    }
    Ok(result)
}

fn parse_ring_texture_assignments(
    values: &serde_json::Value,
    expected_len: usize,
    resources: &GeometryResources,
) -> Result<Vec<Option<RingTextureAssignment>>> {
    let mut assignments = Vec::new();
    flatten_ring_texture_assignments(values, resources, &mut assignments)?;
    if assignments.len() < expected_len {
        assignments.resize(expected_len, None);
    }
    Ok(assignments)
}

fn flatten_ring_texture_assignments(
    value: &serde_json::Value,
    resources: &GeometryResources,
    out: &mut Vec<Option<RingTextureAssignment>>,
) -> Result<()> {
    if looks_like_ring_texture_assignment(value) {
        out.push(parse_ring_texture_assignment(value, resources)?);
        return Ok(());
    }
    let values = value.as_array().ok_or_else(|| {
        Error::InvalidValue(format!(
            "geometry texture.values must be a nested array, got {value}"
        ))
    })?;
    for child in values {
        flatten_ring_texture_assignments(child, resources, out)?;
    }
    Ok(())
}

fn looks_like_ring_texture_assignment(value: &serde_json::Value) -> bool {
    let Some(values) = value.as_array() else {
        return false;
    };
    let Some(first) = values.first() else {
        return false;
    };
    match first {
        serde_json::Value::Null => values.len() == 1,
        serde_json::Value::Number(_) => values.iter().skip(1).all(serde_json::Value::is_number),
        _ => false,
    }
}

fn parse_ring_texture_assignment(
    value: &serde_json::Value,
    resources: &GeometryResources,
) -> Result<Option<RingTextureAssignment>> {
    let values = value.as_array().ok_or_else(|| {
        Error::InvalidValue("geometry texture ring value must be an array".to_owned())
    })?;
    let first = values.first().ok_or_else(|| {
        Error::InvalidValue("geometry texture ring value cannot be empty".to_owned())
    })?;
    if first.is_null() {
        return Ok(None);
    }
    let tex_u64 = first.as_u64().ok_or_else(|| {
        Error::InvalidValue("geometry texture index must be an unsigned integer".to_owned())
    })?;
    let tex_index = usize::try_from(tex_u64)
        .map_err(|_| Error::InvalidValue("geometry texture index out of range".to_owned()))?;
    let texture = resources.textures.get(tex_index).copied().ok_or_else(|| {
        Error::InvalidValue(format!("invalid geometry texture index '{tex_index}'"))
    })?;
    let mut uvs = Vec::with_capacity(values.len().saturating_sub(1));
    for uv in values.iter().skip(1) {
        let index = uv.as_u64().ok_or_else(|| {
            Error::InvalidValue(format!(
                "geometry texture uv index must be an unsigned integer, got {uv}"
            ))
        })?;
        uvs.push(VertexIndex::new(u32::try_from(index).map_err(|_| {
            Error::InvalidValue(format!("geometry texture uv index {index} out of range"))
        })?));
    }
    Ok(Some(RingTextureAssignment { texture, uvs }))
}

// ---------------------------------------------------------------------------
// Mapped draft builders (geometry with semantic / material / texture)
// ---------------------------------------------------------------------------

fn mapped_ring_draft<'de, SS>(
    vertices: MultiPointBoundary,
    mappings: &SurfaceMappings<'de>,
    ring_index: &mut usize,
) -> RingDraft<u32, SS>
where
    SS: ParseStringStorage<'de>,
{
    let current = *ring_index;
    let mut ring = RingDraft::new(vertices.into_iter().map(VertexIndex::new));
    for (theme, assignments) in &mappings.textures {
        if let Some(Some(tex)) = assignments.get(current) {
            ring = ring.with_texture(
                ThemeName::<SS>::new(SS::store(theme)),
                tex.texture,
                tex.uvs.iter().copied(),
            );
        }
    }
    *ring_index += 1;
    ring
}

fn mapped_surface_draft<'de, SS>(
    rings: MultiLineStringBoundary,
    mappings: &SurfaceMappings<'de>,
    surface_index: &mut usize,
    ring_index: &mut usize,
) -> Result<SurfaceDraft<u32, SS>>
where
    SS: ParseStringStorage<'de>,
{
    let current = *surface_index;
    let mut rings_iter = rings.into_iter();
    let outer_verts = rings_iter.next().ok_or_else(|| {
        Error::InvalidValue("surface boundary requires at least one ring".to_owned())
    })?;
    let outer = mapped_ring_draft::<SS>(outer_verts, mappings, ring_index);
    let inners = rings_iter
        .map(|r| mapped_ring_draft::<SS>(r, mappings, ring_index))
        .collect::<Vec<_>>();
    let mut surface = SurfaceDraft::new(outer, inners);
    if let Some(Some(sem)) = mappings.semantics.get(current) {
        surface = surface.with_semantic(*sem);
    }
    for (theme, assignments) in &mappings.materials {
        if let Some(Some(mat)) = assignments.get(current) {
            surface = surface.with_material(ThemeName::<SS>::new(SS::store(theme)), *mat);
        }
    }
    *surface_index += 1;
    Ok(surface)
}

fn mapped_shell_draft<'de, SS>(
    surfaces: MultiSurfaceBoundary,
    mappings: &SurfaceMappings<'de>,
    surface_index: &mut usize,
    ring_index: &mut usize,
) -> Result<ShellDraft<u32, SS>>
where
    SS: ParseStringStorage<'de>,
{
    let surfaces = surfaces
        .into_iter()
        .map(|s| mapped_surface_draft::<SS>(s, mappings, surface_index, ring_index))
        .collect::<Result<Vec<_>>>()?;
    Ok(ShellDraft::new(surfaces))
}

fn mapped_solid_draft<'de, SS>(
    shells: SolidBoundary,
    mappings: &SurfaceMappings<'de>,
    surface_index: &mut usize,
    ring_index: &mut usize,
) -> Result<SolidDraft<u32, SS>>
where
    SS: ParseStringStorage<'de>,
{
    let mut shells_iter = shells
        .into_iter()
        .map(|sh| mapped_shell_draft::<SS>(sh, mappings, surface_index, ring_index));
    let outer = shells_iter.next().transpose()?.ok_or_else(|| {
        Error::InvalidValue("solid boundary requires at least one shell".to_owned())
    })?;
    let inners = shells_iter.collect::<Result<Vec<_>>>()?;
    Ok(SolidDraft::new(outer, inners))
}

// ---------------------------------------------------------------------------
// Generic (unmapped) geometry draft helpers
// ---------------------------------------------------------------------------

fn point_draft(index: u32) -> PointDraft<u32> {
    PointDraft::new(VertexIndex::new(index))
}

fn ring_draft<SS: StringStorage>(vertices: MultiPointBoundary) -> RingDraft<u32, SS> {
    RingDraft::new(vertices.into_iter().map(VertexIndex::new))
}

fn surface_draft<SS: StringStorage>(
    rings: MultiLineStringBoundary,
) -> Result<SurfaceDraft<u32, SS>> {
    let mut rings_iter = rings.into_iter();
    let outer = rings_iter.next().map(ring_draft::<SS>).ok_or_else(|| {
        Error::InvalidValue("surface boundary requires at least one ring".to_owned())
    })?;
    let inners: Vec<_> = rings_iter.map(ring_draft::<SS>).collect();
    Ok(SurfaceDraft::new(outer, inners))
}

fn shell_draft<SS: StringStorage>(surfaces: MultiSurfaceBoundary) -> Result<ShellDraft<u32, SS>> {
    let surfaces = surfaces
        .into_iter()
        .map(surface_draft::<SS>)
        .collect::<Result<Vec<_>>>()?;
    Ok(ShellDraft::new(surfaces))
}

fn solid_draft<SS: StringStorage>(shells: SolidBoundary) -> Result<SolidDraft<u32, SS>> {
    let mut shells_iter = shells.into_iter().map(shell_draft::<SS>);
    let outer = shells_iter.next().transpose()?.ok_or_else(|| {
        Error::InvalidValue("solid boundary requires at least one shell".to_owned())
    })?;
    let inners = shells_iter.collect::<Result<Vec<_>>>()?;
    Ok(SolidDraft::new(outer, inners))
}
