use crate::error::{Error, Result};
use crate::schema::{
    CityArrowHeader, CityArrowPackageVersion, CityModelArrowParts, ProjectedFieldSpec,
    ProjectedValueType, ProjectionLayout, canonical_schema_set,
};
use arrow::array::{
    Array, ArrayRef, FixedSizeListArray, Float64Array, LargeStringArray, ListArray, RecordBatch,
    StringArray, UInt32Array, UInt64Array,
};
use arrow::datatypes::{DataType, FieldRef};
use arrow_buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use cityjson::CityModelType;
use cityjson::v2_0::geometry::{MaterialThemesView, TextureThemesView};
use cityjson::v2_0::{
    AttributeValue, BBox, Boundary, CRS, CityModelIdentifier, CityObject, CityObjectIdentifier,
    CityObjectType, Contact, ContactRole, ContactType, Extension, Geometry, GeometryType,
    ImageType, LoD, MaterialMap, Metadata, OwnedAttributeValue, OwnedCityModel, OwnedMaterial,
    OwnedSemantic, OwnedTexture, RGB, RGBA, SemanticMap, SemanticType, StoredGeometryInstance,
    StoredGeometryParts, TextureMap, TextureType, ThemeName, UVCoordinate, VertexIndexVec,
    WrapMode,
};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

const DEFAULT_CITYMODEL_ID: &str = "citymodel";
const FIELD_ROOT_EXTRA_PREFIX: &str = "root_extra__";
const FIELD_METADATA_EXTRA_PREFIX: &str = "metadata_extra__";
const FIELD_METADATA_REFERENCE_DATE: &str = "metadata_field__referenceDate_json";
const FIELD_METADATA_POINT_OF_CONTACT: &str = "metadata_field__pointOfContact_json";
const FIELD_METADATA_DEFAULT_MATERIAL_THEME: &str = "metadata_field__defaultMaterialTheme_json";
const FIELD_METADATA_DEFAULT_TEXTURE_THEME: &str = "metadata_field__defaultTextureTheme_json";
const FIELD_ATTR_PREFIX: &str = "attr__";
const FIELD_EXTRA_PREFIX: &str = "extra__";
const FIELD_JSON_SUFFIX: &str = "_json";
const FIELD_MATERIAL_NAME: &str = "payload.name";
const FIELD_MATERIAL_AMBIENT_INTENSITY: &str = "payload.ambient_intensity";
const FIELD_MATERIAL_DIFFUSE_COLOR: &str = "payload.diffuse_color_json";
const FIELD_MATERIAL_EMISSIVE_COLOR: &str = "payload.emissive_color_json";
const FIELD_MATERIAL_SPECULAR_COLOR: &str = "payload.specular_color_json";
const FIELD_MATERIAL_SHININESS: &str = "payload.shininess";
const FIELD_MATERIAL_TRANSPARENCY: &str = "payload.transparency";
const FIELD_MATERIAL_IS_SMOOTH: &str = "payload.is_smooth";
const FIELD_TEXTURE_IMAGE_TYPE: &str = "payload.image_type";
const FIELD_TEXTURE_WRAP_MODE: &str = "payload.wrap_mode";
const FIELD_TEXTURE_TEXTURE_TYPE: &str = "payload.texture_type";
const FIELD_TEXTURE_BORDER_COLOR: &str = "payload.border_color_json";

#[derive(Debug, Clone)]
struct MetadataRow {
    citymodel_id: String,
    cityjson_version: String,
    citymodel_kind: String,
    identifier: Option<String>,
    title: Option<String>,
    reference_system: Option<String>,
    geographical_extent: Option<[f64; 6]>,
    projected: Vec<Option<String>>,
}

#[derive(Debug, Clone)]
struct TransformRow {
    citymodel_id: String,
    scale: [f64; 3],
    translate: [f64; 3],
}

#[derive(Debug, Clone)]
struct ExtensionRow {
    citymodel_id: String,
    extension_name: String,
    uri: String,
    version: Option<String>,
}

#[derive(Debug, Clone)]
struct VertexRow {
    citymodel_id: String,
    vertex_id: u64,
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Clone)]
struct TemplateVertexRow {
    citymodel_id: String,
    template_vertex_id: u64,
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Clone)]
struct CityObjectRow {
    citymodel_id: String,
    cityobject_id: String,
    cityobject_ix: u64,
    object_type: String,
    geographical_extent: Option<[f64; 6]>,
    attributes: Vec<Option<String>>,
    extra: Vec<Option<String>>,
}

#[derive(Debug, Clone)]
struct CityObjectChildRow {
    citymodel_id: String,
    parent_cityobject_id: String,
    child_ordinal: u32,
    child_cityobject_id: String,
}

#[derive(Debug, Clone)]
struct GeometryRow {
    citymodel_id: String,
    geometry_id: u64,
    cityobject_id: String,
    geometry_ordinal: u32,
    geometry_type: String,
    lod: Option<String>,
}

#[derive(Debug, Clone)]
struct GeometryBoundaryRow {
    citymodel_id: String,
    geometry_id: u64,
    vertex_indices: Vec<u64>,
    line_lengths: Option<Vec<u32>>,
    ring_lengths: Option<Vec<u32>>,
    surface_lengths: Option<Vec<u32>>,
    shell_lengths: Option<Vec<u32>>,
    solid_lengths: Option<Vec<u32>>,
}

#[derive(Debug, Clone)]
struct GeometryInstanceRow {
    citymodel_id: String,
    geometry_id: u64,
    cityobject_id: String,
    geometry_ordinal: u32,
    lod: Option<String>,
    template_geometry_id: u64,
    reference_point_vertex_id: u64,
    transform_matrix: Option<[f64; 16]>,
}

#[derive(Debug, Clone)]
struct TemplateGeometryRow {
    citymodel_id: String,
    template_geometry_id: u64,
    geometry_type: String,
    lod: Option<String>,
}

#[derive(Debug, Clone)]
struct TemplateGeometryBoundaryRow {
    citymodel_id: String,
    template_geometry_id: u64,
    vertex_indices: Vec<u64>,
    line_lengths: Option<Vec<u32>>,
    ring_lengths: Option<Vec<u32>>,
    surface_lengths: Option<Vec<u32>>,
    shell_lengths: Option<Vec<u32>>,
    solid_lengths: Option<Vec<u32>>,
}

#[derive(Debug, Clone)]
struct SemanticRow {
    citymodel_id: String,
    semantic_id: u64,
    semantic_type: String,
    attributes: Vec<Option<String>>,
}

#[derive(Debug, Clone)]
struct SemanticChildRow {
    citymodel_id: String,
    parent_semantic_id: u64,
    child_ordinal: u32,
    child_semantic_id: u64,
}

#[derive(Debug, Clone)]
struct GeometrySurfaceSemanticRow {
    citymodel_id: String,
    geometry_id: u64,
    surface_ordinal: u32,
    semantic_id: Option<u64>,
}

#[derive(Debug, Clone)]
struct MaterialRow {
    citymodel_id: String,
    material_id: u64,
    name: String,
    ambient_intensity: Option<f64>,
    diffuse_color: Option<String>,
    emissive_color: Option<String>,
    specular_color: Option<String>,
    shininess: Option<f64>,
    transparency: Option<f64>,
    is_smooth: Option<bool>,
}

#[derive(Debug, Clone)]
struct GeometrySurfaceMaterialRow {
    citymodel_id: String,
    geometry_id: u64,
    surface_ordinal: u32,
    theme: String,
    material_id: u64,
}

#[derive(Debug, Clone)]
struct TextureRow {
    citymodel_id: String,
    texture_id: u64,
    image_uri: String,
    image_type: String,
    wrap_mode: Option<String>,
    texture_type: Option<String>,
    border_color: Option<String>,
}

#[derive(Debug, Clone)]
struct TextureVertexRow {
    citymodel_id: String,
    uv_id: u64,
    u: f64,
    v: f64,
}

#[derive(Debug, Clone)]
struct GeometryRingTextureRow {
    citymodel_id: String,
    geometry_id: u64,
    surface_ordinal: u32,
    ring_ordinal: u32,
    theme: String,
    texture_id: u64,
    uv_indices: Vec<u64>,
}

pub fn to_parts(model: &OwnedCityModel) -> Result<CityModelArrowParts> {
    reject_unsupported_modules(model)?;

    let citymodel_id = infer_citymodel_id(model);
    let header = CityArrowHeader::new(
        CityArrowPackageVersion::V1Alpha1,
        citymodel_id.clone(),
        model
            .version()
            .unwrap_or(cityjson::CityJSONVersion::V2_0)
            .to_string(),
    );

    let projection = discover_projection_layout(model);
    let schemas = canonical_schema_set(&projection);

    let geometry_id_map = geometry_id_map(model);
    let template_geometry_id_map = template_geometry_id_map(model);
    let semantic_id_map = semantic_id_map(model);
    let material_id_map = material_id_map(model);
    let texture_id_map = texture_id_map(model);

    let metadata_row = metadata_row(model, &header, &projection, &geometry_id_map)?;
    let transform_row = model.transform().map(|transform| TransformRow {
        citymodel_id: citymodel_id.clone(),
        scale: transform.scale(),
        translate: transform.translate(),
    });
    let extension_rows = extension_rows(model, &citymodel_id);
    let vertex_rows = vertex_rows(model, &citymodel_id);
    let cityobject_rows = cityobject_rows(model, &citymodel_id, &projection, &geometry_id_map)?;
    let cityobject_child_rows = cityobject_child_rows(model, &citymodel_id);
    let material_rows = material_rows(model, &citymodel_id);
    let texture_rows = texture_rows(model, &citymodel_id);
    let texture_vertex_rows = texture_vertex_rows(model, &citymodel_id);
    let (
        geometry_rows,
        boundary_rows,
        geometry_instance_rows,
        surface_semantic_rows,
        surface_material_rows,
        ring_texture_rows,
    ) = geometry_rows(
        model,
        &citymodel_id,
        &semantic_id_map,
        &material_id_map,
        &texture_id_map,
        &template_geometry_id_map,
    )?;
    let template_vertex_rows = template_vertex_rows(model, &citymodel_id);
    let (template_geometry_rows, template_boundary_rows) =
        template_geometry_rows(model, &citymodel_id, &template_geometry_id_map)?;
    let semantic_rows = semantic_rows(model, &citymodel_id, &projection, &geometry_id_map)?;
    let semantic_child_rows = semantic_child_rows(model, &citymodel_id, &semantic_id_map);

    Ok(CityModelArrowParts {
        header,
        projection: projection.clone(),
        metadata: metadata_batch(&schemas.metadata, metadata_row)?,
        transform: transform_row
            .map(|row| transform_batch(&schemas.transform, row))
            .transpose()?,
        extensions: optional_batch(extension_rows, |rows| {
            extensions_batch(&schemas.extensions, rows)
        })?,
        vertices: vertices_batch(&schemas.vertices, &vertex_rows)?,
        cityobjects: cityobjects_batch(&schemas.cityobjects, &cityobject_rows, &projection)?,
        cityobject_children: optional_batch(cityobject_child_rows, |rows| {
            cityobject_children_batch(&schemas.cityobject_children, rows)
        })?,
        geometries: geometries_batch(&schemas.geometries, &geometry_rows)?,
        geometry_boundaries: geometry_boundaries_batch(
            &schemas.geometry_boundaries,
            &boundary_rows,
        )?,
        geometry_instances: optional_batch(geometry_instance_rows, |rows| {
            geometry_instances_batch(&schemas.geometry_instances, &rows)
        })?,
        template_vertices: optional_batch(template_vertex_rows, |rows| {
            template_vertices_batch(&schemas.template_vertices, &rows)
        })?,
        template_geometries: optional_batch(template_geometry_rows, |rows| {
            template_geometries_batch(&schemas.template_geometries, &rows)
        })?,
        template_geometry_boundaries: optional_batch(template_boundary_rows, |rows| {
            template_geometry_boundaries_batch(&schemas.template_geometry_boundaries, &rows)
        })?,
        semantics: optional_batch(semantic_rows, |rows| {
            semantics_batch(&schemas.semantics, &rows, &projection)
        })?,
        semantic_children: optional_batch(semantic_child_rows, |rows| {
            semantic_children_batch(&schemas.semantic_children, rows)
        })?,
        geometry_surface_semantics: optional_batch(surface_semantic_rows, |rows| {
            geometry_surface_semantics_batch(&schemas.geometry_surface_semantics, rows)
        })?,
        materials: optional_batch(material_rows, |rows| {
            materials_batch(&schemas.materials, &rows, &projection)
        })?,
        geometry_surface_materials: optional_batch(surface_material_rows, |rows| {
            geometry_surface_materials_batch(&schemas.geometry_surface_materials, rows)
        })?,
        textures: optional_batch(texture_rows, |rows| {
            textures_batch(&schemas.textures, &rows, &projection)
        })?,
        texture_vertices: optional_batch(texture_vertex_rows, |rows| {
            texture_vertices_batch(&schemas.texture_vertices, rows)
        })?,
        geometry_ring_textures: optional_batch(ring_texture_rows, |rows| {
            geometry_ring_textures_batch(&schemas.geometry_ring_textures, rows)
        })?,
    })
}

pub fn from_parts(parts: &CityModelArrowParts) -> Result<OwnedCityModel> {
    ensure_supported_part_table_combinations(parts)?;
    validate_appearance_projection_layout(&parts.projection)?;

    let kind = CityModelType::try_from(read_string_scalar(&parts.metadata, "citymodel_kind", 0)?)?;
    let mut model = OwnedCityModel::new(kind);

    let metadata_row = read_metadata_row(&parts.metadata, &parts.projection)?;
    apply_metadata_row(
        &mut model,
        &metadata_row,
        &parts.projection,
        &HashMap::new(),
    )?;

    if let Some(transform) = &parts.transform {
        let row = read_transform_row(transform)?;
        let target = model.transform_mut();
        target.set_scale(row.scale);
        target.set_translate(row.translate);
    }

    if let Some(extensions) = &parts.extensions {
        for row in read_extension_rows(extensions)? {
            model.extensions_mut().add(Extension::new(
                row.extension_name,
                row.uri,
                row.version.unwrap_or_default(),
            ));
        }
    }

    for row in read_vertex_rows(&parts.vertices)? {
        let coordinate = cityjson::v2_0::RealWorldCoordinate::new(row.x, row.y, row.z);
        model.add_vertex(coordinate)?;
    }

    if let Some(batch) = &parts.template_vertices {
        let mut rows = read_template_vertex_rows(batch)?;
        rows.sort_by_key(|row| row.template_vertex_id);
        for row in rows {
            let coordinate = cityjson::v2_0::RealWorldCoordinate::new(row.x, row.y, row.z);
            model.add_template_vertex(coordinate)?;
        }
    }

    if let Some(batch) = &parts.texture_vertices {
        let mut rows = read_texture_vertex_rows(batch)?;
        rows.sort_by_key(|row| row.uv_id);
        for row in rows {
            model.add_uv_coordinate(UVCoordinate::new(row.u as f32, row.v as f32))?;
        }
    }

    let mut semantic_handle_by_id = HashMap::new();
    if let Some(batch) = &parts.semantics {
        let mut rows = read_semantic_rows(batch, &parts.projection)?;
        rows.sort_by_key(|row| row.semantic_id);
        for row in rows {
            let mut semantic = OwnedSemantic::new(parse_semantic_type(&row.semantic_type));
            apply_projected_attributes(
                semantic.attributes_mut(),
                &parts.projection.semantic_attributes,
                &row.attributes,
                FIELD_ATTR_PREFIX,
                &HashMap::new(),
            )?;
            let handle = model.add_semantic(semantic)?;
            semantic_handle_by_id.insert(row.semantic_id, handle);
        }
        if let Some(children) = &parts.semantic_children {
            for row in read_semantic_child_rows(children)? {
                let parent = *semantic_handle_by_id
                    .get(&row.parent_semantic_id)
                    .ok_or_else(|| {
                        Error::Conversion(format!(
                            "missing semantic {} for child relation",
                            row.parent_semantic_id
                        ))
                    })?;
                let child = *semantic_handle_by_id
                    .get(&row.child_semantic_id)
                    .ok_or_else(|| {
                        Error::Conversion(format!(
                            "missing semantic {} for child relation",
                            row.child_semantic_id
                        ))
                    })?;
                model
                    .get_semantic_mut(parent)
                    .ok_or_else(|| Error::Conversion("semantic parent handle missing".to_string()))?
                    .children_mut()
                    .push(child);
                model
                    .get_semantic_mut(child)
                    .ok_or_else(|| Error::Conversion("semantic child handle missing".to_string()))?
                    .set_parent(parent);
            }
        }
    }

    let mut material_handle_by_id = HashMap::new();
    if let Some(batch) = &parts.materials {
        let mut rows = read_material_rows(batch, &parts.projection)?;
        rows.sort_by_key(|row| row.material_id);
        for row in rows {
            let mut material = OwnedMaterial::new(row.name);
            material.set_ambient_intensity(row.ambient_intensity.map(|value| value as f32));
            material.set_diffuse_color(
                row.diffuse_color
                    .as_deref()
                    .map(parse_rgb_json)
                    .transpose()?,
            );
            material.set_emissive_color(
                row.emissive_color
                    .as_deref()
                    .map(parse_rgb_json)
                    .transpose()?,
            );
            material.set_specular_color(
                row.specular_color
                    .as_deref()
                    .map(parse_rgb_json)
                    .transpose()?,
            );
            material.set_shininess(row.shininess.map(|value| value as f32));
            material.set_transparency(row.transparency.map(|value| value as f32));
            material.set_is_smooth(row.is_smooth);
            let handle = model.add_material(material)?;
            material_handle_by_id.insert(row.material_id, handle);
        }
    }

    let mut texture_handle_by_id = HashMap::new();
    if let Some(batch) = &parts.textures {
        let mut rows = read_texture_rows(batch, &parts.projection)?;
        rows.sort_by_key(|row| row.texture_id);
        for row in rows {
            let mut texture = OwnedTexture::new(row.image_uri, parse_image_type(&row.image_type)?);
            texture.set_wrap_mode(row.wrap_mode.as_deref().map(parse_wrap_mode).transpose()?);
            texture.set_texture_type(
                row.texture_type
                    .as_deref()
                    .map(parse_texture_mapping_type)
                    .transpose()?,
            );
            texture.set_border_color(
                row.border_color
                    .as_deref()
                    .map(parse_rgba_json)
                    .transpose()?,
            );
            let handle = model.add_texture(texture)?;
            texture_handle_by_id.insert(row.texture_id, handle);
        }
    }

    let geometry_boundaries = read_geometry_boundary_rows(&parts.geometry_boundaries)?;
    let boundary_by_geometry_id: HashMap<_, _> = geometry_boundaries
        .into_iter()
        .map(|row| (row.geometry_id, row))
        .collect();
    let template_boundary_by_geometry_id: HashMap<_, _> = parts
        .template_geometry_boundaries
        .as_ref()
        .map(read_template_geometry_boundary_rows)
        .transpose()?
        .unwrap_or_default()
        .into_iter()
        .map(|row| (row.template_geometry_id, row))
        .collect();
    let surface_semantics_by_geometry_id = parts
        .geometry_surface_semantics
        .as_ref()
        .map(read_geometry_surface_semantic_rows)
        .transpose()?
        .unwrap_or_default()
        .into_iter()
        .fold(
            HashMap::<u64, Vec<GeometrySurfaceSemanticRow>>::new(),
            |mut acc, row| {
                acc.entry(row.geometry_id).or_default().push(row);
                acc
            },
        );
    let surface_materials_by_geometry_id = parts
        .geometry_surface_materials
        .as_ref()
        .map(read_geometry_surface_material_rows)
        .transpose()?
        .unwrap_or_default()
        .into_iter()
        .fold(
            HashMap::<u64, Vec<GeometrySurfaceMaterialRow>>::new(),
            |mut acc, row| {
                acc.entry(row.geometry_id).or_default().push(row);
                acc
            },
        );
    let ring_textures_by_geometry_id = parts
        .geometry_ring_textures
        .as_ref()
        .map(read_geometry_ring_texture_rows)
        .transpose()?
        .unwrap_or_default()
        .into_iter()
        .fold(
            HashMap::<u64, Vec<GeometryRingTextureRow>>::new(),
            |mut acc, row| {
                acc.entry(row.geometry_id).or_default().push(row);
                acc
            },
        );

    let mut template_handle_by_id = HashMap::new();
    if let Some(batch) = &parts.template_geometries {
        let mut rows = read_template_geometry_rows(batch)?;
        rows.sort_by_key(|row| row.template_geometry_id);
        for row in rows {
            let boundary = template_boundary_by_geometry_id
                .get(&row.template_geometry_id)
                .ok_or_else(|| {
                    Error::Conversion(format!(
                        "missing boundary row for template geometry {}",
                        row.template_geometry_id
                    ))
                })?;
            let geometry = Geometry::from_stored_parts(StoredGeometryParts {
                type_geometry: parse_geometry_type(&row.geometry_type)?,
                lod: row.lod.as_deref().map(parse_lod).transpose()?,
                boundaries: Some(template_boundary_from_row(boundary, &row.geometry_type)?),
                semantics: None,
                materials: None,
                textures: None,
                instance: None,
            });
            let handle = model.add_geometry_template(geometry)?;
            template_handle_by_id.insert(row.template_geometry_id, handle);
        }
    }

    let mut geometry_handle_by_id = HashMap::new();
    let mut geometry_rows = read_geometry_rows(&parts.geometries)?;
    geometry_rows.sort_by_key(|row| row.geometry_id);
    for row in geometry_rows {
        let boundary = boundary_by_geometry_id
            .get(&row.geometry_id)
            .ok_or_else(|| {
                Error::Conversion(format!(
                    "missing boundary row for geometry {}",
                    row.geometry_id
                ))
            })?;
        let semantics = build_semantic_map(
            &row.geometry_type,
            boundary,
            surface_semantics_by_geometry_id.get(&row.geometry_id),
            &semantic_handle_by_id,
        )?;
        let materials = build_material_maps(
            &row.geometry_type,
            boundary,
            surface_materials_by_geometry_id.get(&row.geometry_id),
            &material_handle_by_id,
        )?;
        let textures = build_texture_maps(
            &row.geometry_type,
            boundary,
            ring_textures_by_geometry_id.get(&row.geometry_id),
            &texture_handle_by_id,
        )?;
        let geometry = Geometry::from_stored_parts(StoredGeometryParts {
            type_geometry: parse_geometry_type(&row.geometry_type)?,
            lod: row.lod.as_deref().map(parse_lod).transpose()?,
            boundaries: Some(boundary_from_row(boundary, &row.geometry_type)?),
            semantics,
            materials,
            textures,
            instance: None,
        });
        let handle = model.add_geometry(geometry)?;
        if geometry_handle_by_id
            .insert(row.geometry_id, handle)
            .is_some()
        {
            return Err(Error::Conversion(format!(
                "duplicate geometry id {}",
                row.geometry_id
            )));
        }
    }

    if let Some(batch) = &parts.geometry_instances {
        let mut rows = read_geometry_instance_rows(batch)?;
        rows.sort_by_key(|row| row.geometry_id);
        for row in rows {
            let template = *template_handle_by_id
                .get(&row.template_geometry_id)
                .ok_or_else(|| {
                    Error::Conversion(format!(
                        "missing template geometry {}",
                        row.template_geometry_id
                    ))
                })?;
            let reference_point = u32::try_from(row.reference_point_vertex_id).map_err(|_| {
                Error::Conversion(format!(
                    "reference point vertex id {} does not fit into u32",
                    row.reference_point_vertex_id
                ))
            })?;
            let geometry = Geometry::from_stored_parts(StoredGeometryParts {
                type_geometry: GeometryType::GeometryInstance,
                lod: row.lod.as_deref().map(parse_lod).transpose()?,
                boundaries: None,
                semantics: None,
                materials: None,
                textures: None,
                instance: Some(StoredGeometryInstance {
                    template,
                    reference_point: cityjson::v2_0::VertexIndex::new(reference_point),
                    transformation: row
                        .transform_matrix
                        .map(cityjson::v2_0::AffineTransform3D::from)
                        .unwrap_or_default(),
                }),
            });
            let handle = model.add_geometry(geometry)?;
            if geometry_handle_by_id
                .insert(row.geometry_id, handle)
                .is_some()
            {
                return Err(Error::Conversion(format!(
                    "duplicate geometry id {}",
                    row.geometry_id
                )));
            }
        }
    }

    let mut cityobject_handle_by_id = HashMap::new();
    let mut cityobject_rows = read_cityobject_rows(&parts.cityobjects, &parts.projection)?;
    cityobject_rows.sort_by_key(|row| row.cityobject_ix);
    for row in cityobject_rows {
        let mut object = CityObject::new(
            CityObjectIdentifier::new(row.cityobject_id.clone()),
            row.object_type.parse::<CityObjectType<_>>()?,
        );
        if let Some(extent) = row.geographical_extent {
            object.set_geographical_extent(Some(BBox::from(extent)));
        }
        apply_projected_attributes(
            object.attributes_mut(),
            &parts.projection.cityobject_attributes,
            &row.attributes,
            FIELD_ATTR_PREFIX,
            &geometry_handle_by_id,
        )?;
        apply_projected_attributes(
            object.extra_mut(),
            &parts.projection.cityobject_extra,
            &row.extra,
            FIELD_EXTRA_PREFIX,
            &geometry_handle_by_id,
        )?;
        let handle = model.cityobjects_mut().add(object)?;
        cityobject_handle_by_id.insert(row.cityobject_id.clone(), handle);
    }

    let mut geometry_attachments = read_geometry_rows(&parts.geometries)?
        .into_iter()
        .map(|row| (row.cityobject_id, row.geometry_id, row.geometry_ordinal))
        .collect::<Vec<_>>();
    if let Some(batch) = &parts.geometry_instances {
        geometry_attachments.extend(
            read_geometry_instance_rows(batch)?
                .into_iter()
                .map(|row| (row.cityobject_id, row.geometry_id, row.geometry_ordinal)),
        );
    }
    geometry_attachments.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then(left.2.cmp(&right.2))
            .then(left.1.cmp(&right.1))
    });

    for (cityobject_id, geometry_id, _) in geometry_attachments {
        let object = cityobject_handle_by_id
            .get(&cityobject_id)
            .copied()
            .ok_or_else(|| Error::Conversion(format!("missing cityobject {}", cityobject_id)))?;
        let geometry = geometry_handle_by_id
            .get(&geometry_id)
            .copied()
            .ok_or_else(|| Error::Conversion(format!("missing geometry {}", geometry_id)))?;
        model
            .cityobjects_mut()
            .get_mut(object)
            .ok_or_else(|| Error::Conversion("missing cityobject handle".to_string()))?
            .add_geometry(geometry);
    }

    if let Some(children) = &parts.cityobject_children {
        for row in read_cityobject_child_rows(children)? {
            let parent = cityobject_handle_by_id
                .get(&row.parent_cityobject_id)
                .copied()
                .ok_or_else(|| {
                    Error::Conversion(format!(
                        "missing parent cityobject {}",
                        row.parent_cityobject_id
                    ))
                })?;
            let child = cityobject_handle_by_id
                .get(&row.child_cityobject_id)
                .copied()
                .ok_or_else(|| {
                    Error::Conversion(format!(
                        "missing child cityobject {}",
                        row.child_cityobject_id
                    ))
                })?;
            model
                .cityobjects_mut()
                .get_mut(parent)
                .ok_or_else(|| Error::Conversion("missing parent handle".to_string()))?
                .add_child(child);
            model
                .cityobjects_mut()
                .get_mut(child)
                .ok_or_else(|| Error::Conversion("missing child handle".to_string()))?
                .add_parent(parent);
        }
    }

    Ok(model)
}

fn reject_unsupported_modules(model: &OwnedCityModel) -> Result<()> {
    for (_, geometry) in model.iter_geometries() {
        if let Some(materials) = geometry.materials() {
            ensure_surface_backed_geometry(geometry.type_geometry(), "geometry materials")?;
            for (_, map) in materials.iter() {
                if !map.points().is_empty() || !map.linestrings().is_empty() {
                    return Err(Error::Unsupported(
                        "point and linestring material mappings".to_string(),
                    ));
                }
            }
        }
        if geometry.textures().is_some() {
            ensure_surface_backed_geometry(geometry.type_geometry(), "geometry textures")?;
        }
        if let Some(semantics) = geometry.semantics() {
            if !semantics.points().is_empty() || !semantics.linestrings().is_empty() {
                return Err(Error::Unsupported(
                    "point and linestring semantic mappings".to_string(),
                ));
            }
        }
    }
    for (_, geometry) in model.iter_geometry_templates() {
        if geometry.instance().is_some() {
            return Err(Error::Unsupported(
                "geometry instances in template geometry pool".to_string(),
            ));
        }
        if geometry.semantics().is_some() {
            return Err(Error::Unsupported(
                "template geometry semantics".to_string(),
            ));
        }
        if geometry.materials().is_some() {
            return Err(Error::Unsupported(
                "template geometry materials".to_string(),
            ));
        }
        if geometry.textures().is_some() {
            return Err(Error::Unsupported("template geometry textures".to_string()));
        }
    }
    Ok(())
}

fn ensure_supported_part_table_combinations(parts: &CityModelArrowParts) -> Result<()> {
    match (
        parts.template_geometries.as_ref(),
        parts.template_geometry_boundaries.as_ref(),
    ) {
        (Some(_), Some(_)) | (None, None) => {}
        _ => {
            return Err(Error::Unsupported(
                "template_geometries and template_geometry_boundaries must either both be present or both be absent".to_string(),
            ))
        }
    }
    if parts.geometry_surface_materials.is_some() && parts.materials.is_none() {
        return Err(Error::Unsupported(
            "geometry_surface_materials without materials".to_string(),
        ));
    }
    if parts.geometry_ring_textures.is_some() && parts.textures.is_none() {
        return Err(Error::Unsupported(
            "geometry_ring_textures without textures".to_string(),
        ));
    }
    if parts.geometry_ring_textures.is_some() && parts.texture_vertices.is_none() {
        return Err(Error::Unsupported(
            "geometry_ring_textures without texture_vertices".to_string(),
        ));
    }
    Ok(())
}

fn infer_citymodel_id(model: &OwnedCityModel) -> String {
    model
        .metadata()
        .and_then(|metadata| metadata.identifier().map(ToString::to_string))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_CITYMODEL_ID.to_string())
}

fn geometry_id_map(model: &OwnedCityModel) -> HashMap<cityjson::prelude::GeometryHandle, u64> {
    model
        .iter_geometries()
        .enumerate()
        .map(|(index, (handle, _))| (handle, index as u64))
        .collect()
}

fn template_geometry_id_map(
    model: &OwnedCityModel,
) -> HashMap<cityjson::prelude::GeometryTemplateHandle, u64> {
    model
        .iter_geometry_templates()
        .enumerate()
        .map(|(index, (handle, _))| (handle, index as u64))
        .collect()
}

fn semantic_id_map(model: &OwnedCityModel) -> HashMap<cityjson::prelude::SemanticHandle, u64> {
    model
        .iter_semantics()
        .enumerate()
        .map(|(index, (handle, _))| (handle, index as u64))
        .collect()
}

fn material_id_map(model: &OwnedCityModel) -> HashMap<cityjson::prelude::MaterialHandle, u64> {
    model
        .iter_materials()
        .enumerate()
        .map(|(index, (handle, _))| (handle, index as u64))
        .collect()
}

fn texture_id_map(model: &OwnedCityModel) -> HashMap<cityjson::prelude::TextureHandle, u64> {
    model
        .iter_textures()
        .enumerate()
        .map(|(index, (handle, _))| (handle, index as u64))
        .collect()
}

fn discover_projection_layout(model: &OwnedCityModel) -> ProjectionLayout {
    let mut layout = ProjectionLayout::default();

    layout.metadata_extra = discover_metadata_projection(model);
    layout.cityobject_attributes = discover_attribute_projection(
        model
            .cityobjects()
            .iter()
            .filter_map(|(_, object)| object.attributes()),
        FIELD_ATTR_PREFIX,
    );
    layout.cityobject_extra = discover_attribute_projection(
        model
            .cityobjects()
            .iter()
            .filter_map(|(_, object)| object.extra()),
        FIELD_EXTRA_PREFIX,
    );
    layout.semantic_attributes = discover_attribute_projection(
        model
            .iter_semantics()
            .filter_map(|(_, semantic)| semantic.attributes()),
        FIELD_ATTR_PREFIX,
    );
    if model.material_count() > 0 {
        layout.material_payload = canonical_material_projection();
    }
    if model.texture_count() > 0 {
        layout.texture_payload = canonical_texture_projection();
    }
    layout
}

fn canonical_material_projection() -> Vec<ProjectedFieldSpec> {
    vec![
        ProjectedFieldSpec::new(FIELD_MATERIAL_NAME, ProjectedValueType::LargeUtf8, false),
        ProjectedFieldSpec::new(
            FIELD_MATERIAL_AMBIENT_INTENSITY,
            ProjectedValueType::Float64,
            true,
        ),
        ProjectedFieldSpec::new(
            FIELD_MATERIAL_DIFFUSE_COLOR,
            ProjectedValueType::LargeUtf8,
            true,
        ),
        ProjectedFieldSpec::new(
            FIELD_MATERIAL_EMISSIVE_COLOR,
            ProjectedValueType::LargeUtf8,
            true,
        ),
        ProjectedFieldSpec::new(
            FIELD_MATERIAL_SPECULAR_COLOR,
            ProjectedValueType::LargeUtf8,
            true,
        ),
        ProjectedFieldSpec::new(FIELD_MATERIAL_SHININESS, ProjectedValueType::Float64, true),
        ProjectedFieldSpec::new(
            FIELD_MATERIAL_TRANSPARENCY,
            ProjectedValueType::Float64,
            true,
        ),
        ProjectedFieldSpec::new(FIELD_MATERIAL_IS_SMOOTH, ProjectedValueType::Boolean, true),
    ]
}

fn canonical_texture_projection() -> Vec<ProjectedFieldSpec> {
    vec![
        ProjectedFieldSpec::new(
            FIELD_TEXTURE_IMAGE_TYPE,
            ProjectedValueType::LargeUtf8,
            false,
        ),
        ProjectedFieldSpec::new(FIELD_TEXTURE_WRAP_MODE, ProjectedValueType::LargeUtf8, true),
        ProjectedFieldSpec::new(
            FIELD_TEXTURE_TEXTURE_TYPE,
            ProjectedValueType::LargeUtf8,
            true,
        ),
        ProjectedFieldSpec::new(
            FIELD_TEXTURE_BORDER_COLOR,
            ProjectedValueType::LargeUtf8,
            true,
        ),
    ]
}

fn validate_appearance_projection_layout(layout: &ProjectionLayout) -> Result<()> {
    let supported_material = canonical_material_projection()
        .into_iter()
        .map(|spec| spec.name)
        .collect::<BTreeSet<_>>();
    for spec in &layout.material_payload {
        if !supported_material.contains(&spec.name) {
            return Err(Error::Unsupported(format!(
                "material payload column {}",
                spec.name
            )));
        }
    }

    let supported_texture = canonical_texture_projection()
        .into_iter()
        .map(|spec| spec.name)
        .collect::<BTreeSet<_>>();
    for spec in &layout.texture_payload {
        if !supported_texture.contains(&spec.name) {
            return Err(Error::Unsupported(format!(
                "texture payload column {}",
                spec.name
            )));
        }
    }
    Ok(())
}

fn discover_metadata_projection(model: &OwnedCityModel) -> Vec<ProjectedFieldSpec> {
    let mut fields = Vec::new();

    if model
        .metadata()
        .and_then(|metadata| metadata.reference_date())
        .is_some()
    {
        fields.push(ProjectedFieldSpec::new(
            FIELD_METADATA_REFERENCE_DATE,
            ProjectedValueType::LargeUtf8,
            true,
        ));
    }
    if model
        .metadata()
        .and_then(|metadata| metadata.point_of_contact())
        .is_some()
    {
        fields.push(ProjectedFieldSpec::new(
            FIELD_METADATA_POINT_OF_CONTACT,
            ProjectedValueType::LargeUtf8,
            true,
        ));
    }
    if model.default_material_theme().is_some() {
        fields.push(ProjectedFieldSpec::new(
            FIELD_METADATA_DEFAULT_MATERIAL_THEME,
            ProjectedValueType::LargeUtf8,
            true,
        ));
    }
    if model.default_texture_theme().is_some() {
        fields.push(ProjectedFieldSpec::new(
            FIELD_METADATA_DEFAULT_TEXTURE_THEME,
            ProjectedValueType::LargeUtf8,
            true,
        ));
    }

    let mut root_keys: BTreeSet<String> = BTreeSet::new();
    if let Some(extra) = model.extra() {
        for key in extra.keys() {
            root_keys.insert(key.to_string());
        }
    }
    for key in root_keys {
        fields.push(ProjectedFieldSpec::new(
            format!(
                "{FIELD_ROOT_EXTRA_PREFIX}{}{FIELD_JSON_SUFFIX}",
                encode_key(&key)
            ),
            ProjectedValueType::LargeUtf8,
            true,
        ));
    }

    let mut metadata_keys: BTreeSet<String> = BTreeSet::new();
    if let Some(metadata) = model.metadata()
        && let Some(extra) = metadata.extra()
    {
        for key in extra.keys() {
            metadata_keys.insert(key.to_string());
        }
    }
    for key in metadata_keys {
        fields.push(ProjectedFieldSpec::new(
            format!(
                "{FIELD_METADATA_EXTRA_PREFIX}{}{FIELD_JSON_SUFFIX}",
                encode_key(&key)
            ),
            ProjectedValueType::LargeUtf8,
            true,
        ));
    }

    fields
}

fn discover_attribute_projection<'a, I>(attributes: I, prefix: &str) -> Vec<ProjectedFieldSpec>
where
    I: IntoIterator<Item = &'a cityjson::v2_0::OwnedAttributes>,
{
    let mut keys: BTreeSet<String> = BTreeSet::new();
    for attrs in attributes {
        for key in attrs.keys() {
            keys.insert(key.to_string());
        }
    }
    keys.into_iter()
        .map(|key| {
            ProjectedFieldSpec::new(
                format!("{prefix}{}{FIELD_JSON_SUFFIX}", encode_key(&key)),
                ProjectedValueType::LargeUtf8,
                true,
            )
        })
        .collect()
}

fn metadata_row(
    model: &OwnedCityModel,
    header: &CityArrowHeader,
    layout: &ProjectionLayout,
    geometry_id_map: &HashMap<cityjson::prelude::GeometryHandle, u64>,
) -> Result<MetadataRow> {
    let metadata = model.metadata();
    Ok(MetadataRow {
        citymodel_id: header.citymodel_id.clone(),
        cityjson_version: header.cityjson_version.clone(),
        citymodel_kind: model.type_citymodel().to_string(),
        identifier: metadata.and_then(|item| item.identifier().map(ToString::to_string)),
        title: metadata.and_then(Metadata::title).map(ToString::to_string),
        reference_system: metadata
            .and_then(|item| item.reference_system().map(ToString::to_string)),
        geographical_extent: metadata
            .and_then(Metadata::geographical_extent)
            .map(|bbox| bbox.as_slice().try_into().expect("bbox is 6 long")),
        projected: project_metadata_columns(model, layout, geometry_id_map)?,
    })
}

fn extension_rows(model: &OwnedCityModel, citymodel_id: &str) -> Vec<ExtensionRow> {
    model
        .extensions()
        .into_iter()
        .flat_map(|extensions| extensions.iter())
        .map(|extension| ExtensionRow {
            citymodel_id: citymodel_id.to_string(),
            extension_name: extension.name().to_string(),
            uri: extension.url().to_string(),
            version: Some(extension.version().to_string()),
        })
        .collect()
}

fn material_rows(model: &OwnedCityModel, citymodel_id: &str) -> Vec<MaterialRow> {
    model
        .iter_materials()
        .enumerate()
        .map(|(index, (_, material))| MaterialRow {
            citymodel_id: citymodel_id.to_string(),
            material_id: index as u64,
            name: material.name().to_string(),
            ambient_intensity: material.ambient_intensity().map(f64::from),
            diffuse_color: material.diffuse_color().map(rgb_to_json),
            emissive_color: material.emissive_color().map(rgb_to_json),
            specular_color: material.specular_color().map(rgb_to_json),
            shininess: material.shininess().map(f64::from),
            transparency: material.transparency().map(f64::from),
            is_smooth: material.is_smooth(),
        })
        .collect()
}

fn texture_rows(model: &OwnedCityModel, citymodel_id: &str) -> Vec<TextureRow> {
    model
        .iter_textures()
        .enumerate()
        .map(|(index, (_, texture))| TextureRow {
            citymodel_id: citymodel_id.to_string(),
            texture_id: index as u64,
            image_uri: texture.image().to_string(),
            image_type: texture.image_type().to_string(),
            wrap_mode: texture.wrap_mode().map(|value| value.to_string()),
            texture_type: texture.texture_type().map(|value| value.to_string()),
            border_color: texture.border_color().map(rgba_to_json),
        })
        .collect()
}

fn texture_vertex_rows(model: &OwnedCityModel, citymodel_id: &str) -> Vec<TextureVertexRow> {
    model
        .vertices_texture()
        .as_slice()
        .iter()
        .enumerate()
        .map(|(index, coordinate)| TextureVertexRow {
            citymodel_id: citymodel_id.to_string(),
            uv_id: index as u64,
            u: f64::from(coordinate.u()),
            v: f64::from(coordinate.v()),
        })
        .collect()
}

fn vertex_rows(model: &OwnedCityModel, citymodel_id: &str) -> Vec<VertexRow> {
    model
        .vertices()
        .as_slice()
        .iter()
        .enumerate()
        .map(|(index, coordinate)| VertexRow {
            citymodel_id: citymodel_id.to_string(),
            vertex_id: index as u64,
            x: coordinate.x(),
            y: coordinate.y(),
            z: coordinate.z(),
        })
        .collect()
}

fn cityobject_rows(
    model: &OwnedCityModel,
    citymodel_id: &str,
    projection: &ProjectionLayout,
    geometry_id_map: &HashMap<cityjson::prelude::GeometryHandle, u64>,
) -> Result<Vec<CityObjectRow>> {
    model
        .cityobjects()
        .iter()
        .enumerate()
        .map(|(index, (_, object))| {
            Ok(CityObjectRow {
                citymodel_id: citymodel_id.to_string(),
                cityobject_id: object.id().to_string(),
                cityobject_ix: index as u64,
                object_type: object.type_cityobject().to_string(),
                geographical_extent: object
                    .geographical_extent()
                    .map(|bbox| bbox.as_slice().try_into().expect("bbox is 6 long")),
                attributes: project_attribute_columns(
                    object.attributes(),
                    &projection.cityobject_attributes,
                    FIELD_ATTR_PREFIX,
                    geometry_id_map,
                )?,
                extra: project_attribute_columns(
                    object.extra(),
                    &projection.cityobject_extra,
                    FIELD_EXTRA_PREFIX,
                    geometry_id_map,
                )?,
            })
        })
        .collect()
}

fn cityobject_child_rows(model: &OwnedCityModel, citymodel_id: &str) -> Vec<CityObjectChildRow> {
    let mut rows = Vec::new();
    for (_, object) in model.cityobjects().iter() {
        if let Some(children) = object.children() {
            for (ordinal, child) in children.iter().enumerate() {
                if let Some(child_object) = model.cityobjects().get(*child) {
                    rows.push(CityObjectChildRow {
                        citymodel_id: citymodel_id.to_string(),
                        parent_cityobject_id: object.id().to_string(),
                        child_ordinal: ordinal as u32,
                        child_cityobject_id: child_object.id().to_string(),
                    });
                }
            }
        }
    }
    rows
}

fn geometry_rows(
    model: &OwnedCityModel,
    citymodel_id: &str,
    semantic_id_map: &HashMap<cityjson::prelude::SemanticHandle, u64>,
    material_id_map: &HashMap<cityjson::prelude::MaterialHandle, u64>,
    texture_id_map: &HashMap<cityjson::prelude::TextureHandle, u64>,
    template_geometry_id_map: &HashMap<cityjson::prelude::GeometryTemplateHandle, u64>,
) -> Result<(
    Vec<GeometryRow>,
    Vec<GeometryBoundaryRow>,
    Vec<GeometryInstanceRow>,
    Vec<GeometrySurfaceSemanticRow>,
    Vec<GeometrySurfaceMaterialRow>,
    Vec<GeometryRingTextureRow>,
)> {
    let mut geometry_rows = Vec::new();
    let mut boundary_rows = Vec::new();
    let mut instance_rows = Vec::new();
    let mut semantic_rows = Vec::new();
    let mut material_rows = Vec::new();
    let mut texture_rows = Vec::new();
    let geometry_id_map = geometry_id_map(model);

    for (_, object) in model.cityobjects().iter() {
        if let Some(geometries) = object.geometry() {
            for (ordinal, geometry_handle) in geometries.iter().enumerate() {
                let geometry_id = *geometry_id_map.get(geometry_handle).ok_or_else(|| {
                    Error::Conversion("geometry handle missing from id map".to_string())
                })?;
                let geometry = model.get_geometry(*geometry_handle).ok_or_else(|| {
                    Error::Conversion(format!("missing geometry for handle {:?}", geometry_handle))
                })?;
                if *geometry.type_geometry() == GeometryType::GeometryInstance {
                    let instance = geometry.instance().ok_or_else(|| {
                        Error::Conversion("geometry instance missing instance payload".to_string())
                    })?;
                    let template_geometry_id = *template_geometry_id_map
                        .get(&instance.template())
                        .ok_or_else(|| {
                            Error::Conversion(format!(
                                "missing template id for instance geometry {:?}",
                                geometry_handle
                            ))
                        })?;
                    instance_rows.push(GeometryInstanceRow {
                        citymodel_id: citymodel_id.to_string(),
                        geometry_id,
                        cityobject_id: object.id().to_string(),
                        geometry_ordinal: ordinal as u32,
                        lod: geometry.lod().map(ToString::to_string),
                        template_geometry_id,
                        reference_point_vertex_id: u64::from(instance.reference_point().value()),
                        transform_matrix: Some(instance.transformation().into()),
                    });
                } else {
                    let boundary = geometry.boundaries().ok_or_else(|| {
                        Error::Conversion(
                            "boundary-carrying geometry missing boundaries".to_string(),
                        )
                    })?;
                    let boundary_row = geometry_boundary_row(
                        citymodel_id,
                        geometry_id,
                        geometry.type_geometry(),
                        boundary,
                    );
                    if let Some(semantics) = geometry.semantics() {
                        for (surface_ordinal, semantic_id) in
                            semantics.surfaces().iter().enumerate()
                        {
                            semantic_rows.push(GeometrySurfaceSemanticRow {
                                citymodel_id: citymodel_id.to_string(),
                                geometry_id,
                                surface_ordinal: surface_ordinal as u32,
                                semantic_id: semantic_id
                                    .and_then(|handle| semantic_id_map.get(&handle).copied()),
                            });
                        }
                    }
                    material_rows.extend(geometry_surface_material_rows(
                        citymodel_id,
                        geometry_id,
                        geometry.type_geometry(),
                        &boundary_row,
                        geometry.materials(),
                        material_id_map,
                    )?);
                    texture_rows.extend(geometry_ring_texture_rows(
                        citymodel_id,
                        geometry_id,
                        geometry.type_geometry(),
                        &boundary_row,
                        geometry.textures(),
                        texture_id_map,
                    )?);
                    geometry_rows.push(GeometryRow {
                        citymodel_id: citymodel_id.to_string(),
                        geometry_id,
                        cityobject_id: object.id().to_string(),
                        geometry_ordinal: ordinal as u32,
                        geometry_type: geometry.type_geometry().to_string(),
                        lod: geometry.lod().map(ToString::to_string),
                    });
                    boundary_rows.push(boundary_row);
                }
            }
        }
    }

    Ok((
        geometry_rows,
        boundary_rows,
        instance_rows,
        semantic_rows,
        material_rows,
        texture_rows,
    ))
}

#[derive(Debug, Clone, Copy)]
struct RingLayout {
    start: usize,
    len: usize,
    surface_ordinal: u32,
    ring_ordinal: u32,
}

fn geometry_surface_material_rows(
    citymodel_id: &str,
    geometry_id: u64,
    geometry_type: &GeometryType,
    boundary_row: &GeometryBoundaryRow,
    materials: Option<MaterialThemesView<'_, u32, cityjson::prelude::OwnedStringStorage>>,
    material_id_map: &HashMap<cityjson::prelude::MaterialHandle, u64>,
) -> Result<Vec<GeometrySurfaceMaterialRow>> {
    let Some(materials) = materials else {
        return Ok(Vec::new());
    };
    ensure_surface_backed_geometry(geometry_type, "geometry materials")?;
    let surface_count = surface_count(boundary_row)?;
    let mut rows = Vec::new();

    for (theme, map) in materials.iter() {
        if !map.points().is_empty() || !map.linestrings().is_empty() {
            return Err(Error::Unsupported(
                "point and linestring material mappings".to_string(),
            ));
        }
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
            let material_id = *material_id_map.get(material_handle).ok_or_else(|| {
                Error::Conversion("material handle missing from id map".to_string())
            })?;
            rows.push(GeometrySurfaceMaterialRow {
                citymodel_id: citymodel_id.to_string(),
                geometry_id,
                surface_ordinal: surface_ordinal as u32,
                theme: theme.as_ref().to_string(),
                material_id,
            });
        }
    }

    Ok(rows)
}

fn geometry_ring_texture_rows(
    citymodel_id: &str,
    geometry_id: u64,
    geometry_type: &GeometryType,
    boundary_row: &GeometryBoundaryRow,
    textures: Option<TextureThemesView<'_, u32, cityjson::prelude::OwnedStringStorage>>,
    texture_id_map: &HashMap<cityjson::prelude::TextureHandle, u64>,
) -> Result<Vec<GeometryRingTextureRow>> {
    let Some(textures) = textures else {
        return Ok(Vec::new());
    };
    ensure_surface_backed_geometry(geometry_type, "geometry textures")?;
    let ring_layouts = ring_layouts(boundary_row)?;
    let mut rows = Vec::new();

    for (theme, map) in textures.iter() {
        if map.rings().len() != ring_layouts.len() {
            return Err(Error::Conversion(format!(
                "texture theme {} has {} rings, expected {}",
                theme,
                map.rings().len(),
                ring_layouts.len()
            )));
        }
        if map.vertices().len() != boundary_row.vertex_indices.len() {
            return Err(Error::Conversion(format!(
                "texture theme {} has {} uv assignments, expected {}",
                theme,
                map.vertices().len(),
                boundary_row.vertex_indices.len()
            )));
        }
        let ring_textures = map.ring_textures();
        for (ring_index, layout) in ring_layouts.iter().enumerate() {
            let Some(texture_handle) = ring_textures[ring_index] else {
                continue;
            };
            let texture_id = *texture_id_map.get(&texture_handle).ok_or_else(|| {
                Error::Conversion("texture handle missing from id map".to_string())
            })?;
            let uv_indices = map.vertices()[layout.start..layout.start + layout.len]
                .iter()
                .map(|value: &Option<cityjson::v2_0::VertexIndex<u32>>| {
                    value
                        .map(|uv: cityjson::v2_0::VertexIndex<u32>| u64::from(uv.value()))
                        .ok_or_else(|| {
                            Error::Conversion(format!(
                                "textured ring {} for theme {} contains missing uv indices",
                                ring_index, theme
                            ))
                        })
                })
                .collect::<Result<Vec<_>>>()?;
            rows.push(GeometryRingTextureRow {
                citymodel_id: citymodel_id.to_string(),
                geometry_id,
                surface_ordinal: layout.surface_ordinal,
                ring_ordinal: layout.ring_ordinal,
                theme: theme.as_ref().to_string(),
                texture_id,
                uv_indices,
            });
        }
    }

    Ok(rows)
}

fn ensure_surface_backed_geometry(geometry_type: &GeometryType, feature: &str) -> Result<()> {
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

fn ring_layouts(boundary_row: &GeometryBoundaryRow) -> Result<Vec<RingLayout>> {
    let ring_lengths = required_lengths(&boundary_row.ring_lengths, "ring_lengths")?;
    let surface_lengths = required_lengths(&boundary_row.surface_lengths, "surface_lengths")?;
    let mut layouts = Vec::with_capacity(ring_lengths.len());
    let mut vertex_start = 0_usize;
    let mut ring_index = 0_usize;

    for (surface_ordinal, ring_count) in surface_lengths.iter().enumerate() {
        for ring_ordinal in 0..*ring_count {
            let len = *ring_lengths.get(ring_index).ok_or_else(|| {
                Error::Conversion(format!(
                    "surface topology references missing ring {}",
                    ring_index
                ))
            })? as usize;
            layouts.push(RingLayout {
                start: vertex_start,
                len,
                surface_ordinal: surface_ordinal as u32,
                ring_ordinal,
            });
            vertex_start += len;
            ring_index += 1;
        }
    }

    if ring_index != ring_lengths.len() {
        return Err(Error::Conversion(format!(
            "ring topology consumed {} rings, but {} ring lengths are present",
            ring_index,
            ring_lengths.len()
        )));
    }
    if vertex_start != boundary_row.vertex_indices.len() {
        return Err(Error::Conversion(format!(
            "ring topology consumed {} vertices, but {} boundary vertices are present",
            vertex_start,
            boundary_row.vertex_indices.len()
        )));
    }

    Ok(layouts)
}

fn rgb_to_json(value: RGB) -> String {
    JsonValue::Array(
        value
            .to_array()
            .into_iter()
            .map(|component| {
                JsonValue::Number(
                    JsonNumber::from_f64(f64::from(component)).expect("finite rgb component"),
                )
            })
            .collect(),
    )
    .to_string()
}

fn rgba_to_json(value: RGBA) -> String {
    JsonValue::Array(
        value
            .to_array()
            .into_iter()
            .map(|component| {
                JsonValue::Number(
                    JsonNumber::from_f64(f64::from(component)).expect("finite rgba component"),
                )
            })
            .collect(),
    )
    .to_string()
}

#[derive(Debug, Clone)]
struct FlattenedBoundary {
    vertex_indices: Vec<u64>,
    line_lengths: Option<Vec<u32>>,
    ring_lengths: Option<Vec<u32>>,
    surface_lengths: Option<Vec<u32>>,
    shell_lengths: Option<Vec<u32>>,
    solid_lengths: Option<Vec<u32>>,
}

fn geometry_boundary_row(
    citymodel_id: &str,
    geometry_id: u64,
    geometry_type: &GeometryType,
    boundary: &Boundary<u32>,
) -> GeometryBoundaryRow {
    let payload = flatten_boundary(geometry_type, boundary);
    GeometryBoundaryRow {
        citymodel_id: citymodel_id.to_string(),
        geometry_id,
        vertex_indices: payload.vertex_indices,
        line_lengths: payload.line_lengths,
        ring_lengths: payload.ring_lengths,
        surface_lengths: payload.surface_lengths,
        shell_lengths: payload.shell_lengths,
        solid_lengths: payload.solid_lengths,
    }
}

fn template_geometry_rows(
    model: &OwnedCityModel,
    citymodel_id: &str,
    template_geometry_id_map: &HashMap<cityjson::prelude::GeometryTemplateHandle, u64>,
) -> Result<(Vec<TemplateGeometryRow>, Vec<TemplateGeometryBoundaryRow>)> {
    let mut geometry_rows = Vec::new();
    let mut boundary_rows = Vec::new();
    for (handle, geometry) in model.iter_geometry_templates() {
        let template_geometry_id = *template_geometry_id_map.get(&handle).ok_or_else(|| {
            Error::Conversion("template geometry handle missing from id map".to_string())
        })?;
        let boundary = geometry
            .boundaries()
            .ok_or_else(|| Error::Conversion("template geometry missing boundaries".to_string()))?;
        geometry_rows.push(TemplateGeometryRow {
            citymodel_id: citymodel_id.to_string(),
            template_geometry_id,
            geometry_type: geometry.type_geometry().to_string(),
            lod: geometry.lod().map(ToString::to_string),
        });
        boundary_rows.push(template_geometry_boundary_row(
            citymodel_id,
            template_geometry_id,
            geometry.type_geometry(),
            boundary,
        ));
    }
    Ok((geometry_rows, boundary_rows))
}

fn template_vertex_rows(model: &OwnedCityModel, citymodel_id: &str) -> Vec<TemplateVertexRow> {
    model
        .template_vertices()
        .as_slice()
        .iter()
        .enumerate()
        .map(|(index, coordinate)| TemplateVertexRow {
            citymodel_id: citymodel_id.to_string(),
            template_vertex_id: index as u64,
            x: coordinate.x(),
            y: coordinate.y(),
            z: coordinate.z(),
        })
        .collect()
}

fn template_geometry_boundary_row(
    citymodel_id: &str,
    template_geometry_id: u64,
    geometry_type: &GeometryType,
    boundary: &Boundary<u32>,
) -> TemplateGeometryBoundaryRow {
    let payload = flatten_boundary(geometry_type, boundary);
    TemplateGeometryBoundaryRow {
        citymodel_id: citymodel_id.to_string(),
        template_geometry_id,
        vertex_indices: payload.vertex_indices,
        line_lengths: payload.line_lengths,
        ring_lengths: payload.ring_lengths,
        surface_lengths: payload.surface_lengths,
        shell_lengths: payload.shell_lengths,
        solid_lengths: payload.solid_lengths,
    }
}

fn flatten_boundary(geometry_type: &GeometryType, boundary: &Boundary<u32>) -> FlattenedBoundary {
    let vertices = boundary
        .vertices_raw()
        .iter()
        .copied()
        .map(u64::from)
        .collect();
    let ring_lengths = offsets_to_lengths(boundary.rings_raw(), boundary.vertices_raw().len());
    let surface_lengths = offsets_to_lengths(boundary.surfaces_raw(), boundary.rings_raw().len());
    let shell_lengths = offsets_to_lengths(boundary.shells_raw(), boundary.surfaces_raw().len());
    let solid_lengths = offsets_to_lengths(boundary.solids_raw(), boundary.shells_raw().len());

    let (line_lengths, ring_lengths, surface_lengths, shell_lengths, solid_lengths) =
        match geometry_type {
            GeometryType::MultiPoint => (None, None, None, None, None),
            GeometryType::MultiLineString => (Some(ring_lengths), None, None, None, None),
            GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                (None, Some(ring_lengths), Some(surface_lengths), None, None)
            }
            GeometryType::Solid => (
                None,
                Some(ring_lengths),
                Some(surface_lengths),
                Some(shell_lengths),
                None,
            ),
            GeometryType::MultiSolid | GeometryType::CompositeSolid => (
                None,
                Some(ring_lengths),
                Some(surface_lengths),
                Some(shell_lengths),
                Some(solid_lengths),
            ),
            GeometryType::GeometryInstance => unreachable!("instances rejected earlier"),
            _ => unreachable!("unsupported geometry type rejected earlier"),
        };

    FlattenedBoundary {
        vertex_indices: vertices,
        line_lengths,
        ring_lengths,
        surface_lengths,
        shell_lengths,
        solid_lengths,
    }
}

fn semantic_rows(
    model: &OwnedCityModel,
    citymodel_id: &str,
    projection: &ProjectionLayout,
    geometry_id_map: &HashMap<cityjson::prelude::GeometryHandle, u64>,
) -> Result<Vec<SemanticRow>> {
    model
        .iter_semantics()
        .enumerate()
        .map(|(index, (_, semantic))| {
            Ok(SemanticRow {
                citymodel_id: citymodel_id.to_string(),
                semantic_id: index as u64,
                semantic_type: semantic.type_semantic().to_string(),
                attributes: project_attribute_columns(
                    semantic.attributes(),
                    &projection.semantic_attributes,
                    FIELD_ATTR_PREFIX,
                    geometry_id_map,
                )?,
            })
        })
        .collect()
}

fn semantic_child_rows(
    model: &OwnedCityModel,
    citymodel_id: &str,
    semantic_id_map: &HashMap<cityjson::prelude::SemanticHandle, u64>,
) -> Vec<SemanticChildRow> {
    let mut rows = Vec::new();
    for (handle, semantic) in model.iter_semantics() {
        if let Some(children) = semantic.children() {
            let parent_id = semantic_id_map.get(&handle).copied().unwrap_or_default();
            for (ordinal, child) in children.iter().enumerate() {
                if let Some(child_id) = semantic_id_map.get(child).copied() {
                    rows.push(SemanticChildRow {
                        citymodel_id: citymodel_id.to_string(),
                        parent_semantic_id: parent_id,
                        child_ordinal: ordinal as u32,
                        child_semantic_id: child_id,
                    });
                }
            }
        }
    }
    rows
}

fn offsets_to_lengths(raw: cityjson::v2_0::RawVertexView<'_, u32>, child_len: usize) -> Vec<u32> {
    let raw = &*raw;
    if raw.is_empty() {
        return Vec::new();
    }
    let mut lengths = Vec::with_capacity(raw.len());
    for window in raw.windows(2) {
        lengths.push(window[1] - window[0]);
    }
    lengths.push(child_len as u32 - raw[raw.len() - 1]);
    lengths
}

fn project_metadata_columns(
    model: &OwnedCityModel,
    layout: &ProjectionLayout,
    geometry_id_map: &HashMap<cityjson::prelude::GeometryHandle, u64>,
) -> Result<Vec<Option<String>>> {
    let metadata = model.metadata();
    let metadata_extra = metadata.and_then(Metadata::extra);
    let root_extra = model.extra();
    let point_of_contact = metadata.and_then(Metadata::point_of_contact);

    layout
        .metadata_extra
        .iter()
        .map(|spec| {
            if spec.name == FIELD_METADATA_REFERENCE_DATE {
                return Ok(metadata
                    .and_then(Metadata::reference_date)
                    .map(ToString::to_string)
                    .map(json_string));
            }
            if spec.name == FIELD_METADATA_POINT_OF_CONTACT {
                return point_of_contact
                    .map(contact_to_json)
                    .transpose()
                    .map(|value| value.map(|item| item.to_string()));
            }
            if spec.name == FIELD_METADATA_DEFAULT_MATERIAL_THEME {
                return Ok(model
                    .default_material_theme()
                    .map(ToString::to_string)
                    .map(json_string));
            }
            if spec.name == FIELD_METADATA_DEFAULT_TEXTURE_THEME {
                return Ok(model
                    .default_texture_theme()
                    .map(ToString::to_string)
                    .map(json_string));
            }
            if let Some(key) = decode_projection_name(&spec.name, FIELD_ROOT_EXTRA_PREFIX) {
                return project_one_attribute(root_extra, &key, geometry_id_map);
            }
            if let Some(key) = decode_projection_name(&spec.name, FIELD_METADATA_EXTRA_PREFIX) {
                return project_one_attribute(metadata_extra, &key, geometry_id_map);
            }
            Err(Error::Conversion(format!(
                "unrecognized metadata projection column {}",
                spec.name
            )))
        })
        .collect()
}

fn project_attribute_columns(
    attributes: Option<&cityjson::v2_0::OwnedAttributes>,
    layout: &[ProjectedFieldSpec],
    prefix: &str,
    geometry_id_map: &HashMap<cityjson::prelude::GeometryHandle, u64>,
) -> Result<Vec<Option<String>>> {
    layout
        .iter()
        .map(|spec| {
            let key = decode_projection_name(&spec.name, prefix).ok_or_else(|| {
                Error::Conversion(format!("invalid projection column {}", spec.name))
            })?;
            project_one_attribute(attributes, &key, geometry_id_map)
        })
        .collect()
}

fn project_one_attribute(
    attributes: Option<&cityjson::v2_0::OwnedAttributes>,
    key: &str,
    geometry_id_map: &HashMap<cityjson::prelude::GeometryHandle, u64>,
) -> Result<Option<String>> {
    attributes
        .and_then(|attributes| attributes.get(key))
        .map(|value| attribute_to_json(value, geometry_id_map).map(|json| json.to_string()))
        .transpose()
}

fn attribute_to_json(
    value: &OwnedAttributeValue,
    geometry_id_map: &HashMap<cityjson::prelude::GeometryHandle, u64>,
) -> Result<JsonValue> {
    Ok(match value {
        AttributeValue::Null => JsonValue::Null,
        AttributeValue::Bool(value) => JsonValue::Bool(*value),
        AttributeValue::Unsigned(value) => JsonValue::Number(JsonNumber::from(*value)),
        AttributeValue::Integer(value) => JsonValue::Number(JsonNumber::from(*value)),
        AttributeValue::Float(value) => JsonNumber::from_f64(*value)
            .map(JsonValue::Number)
            .ok_or_else(|| Error::Conversion(format!("cannot encode float attribute {value}")))?,
        AttributeValue::String(value) => JsonValue::String(value.clone()),
        AttributeValue::Vec(values) => JsonValue::Array(
            values
                .iter()
                .map(|value| attribute_to_json(value, geometry_id_map))
                .collect::<Result<Vec<_>>>()?,
        ),
        AttributeValue::Map(values) => JsonValue::Object(
            values
                .iter()
                .map(|(key, value)| Ok((key.clone(), attribute_to_json(value, geometry_id_map)?)))
                .collect::<Result<JsonMap<_, _>>>()?,
        ),
        AttributeValue::Geometry(handle) => {
            let geometry_id = geometry_id_map.get(handle).copied().ok_or_else(|| {
                Error::Conversion("attribute geometry handle missing from map".to_string())
            })?;
            let mut object = JsonMap::new();
            object.insert(
                "__cityarrow_geometry_id".to_string(),
                JsonValue::Number(JsonNumber::from(geometry_id)),
            );
            JsonValue::Object(object)
        }
        _ => {
            return Err(Error::Unsupported(
                "unsupported attribute value variant".to_string(),
            ));
        }
    })
}

fn json_to_attribute(
    value: &JsonValue,
    geometry_handles: &HashMap<u64, cityjson::prelude::GeometryHandle>,
) -> Result<OwnedAttributeValue> {
    Ok(match value {
        JsonValue::Null => AttributeValue::Null,
        JsonValue::Bool(value) => AttributeValue::Bool(*value),
        JsonValue::Number(value) => {
            if let Some(unsigned) = value.as_u64() {
                AttributeValue::Unsigned(unsigned)
            } else if let Some(integer) = value.as_i64() {
                AttributeValue::Integer(integer)
            } else {
                AttributeValue::Float(
                    value.as_f64().ok_or_else(|| {
                        Error::Conversion("failed to decode json number".to_string())
                    })?,
                )
            }
        }
        JsonValue::String(value) => AttributeValue::String(value.clone()),
        JsonValue::Array(values) => AttributeValue::Vec(
            values
                .iter()
                .map(|value| json_to_attribute(value, geometry_handles))
                .collect::<Result<Vec<_>>>()?,
        ),
        JsonValue::Object(values) => {
            if values.len() == 1 && values.contains_key("__cityarrow_geometry_id") {
                let geometry_id = values["__cityarrow_geometry_id"].as_u64().ok_or_else(|| {
                    Error::Conversion("invalid geometry id attribute payload".to_string())
                })?;
                let handle = geometry_handles.get(&geometry_id).copied().ok_or_else(|| {
                    Error::Conversion(format!(
                        "missing geometry handle {} for attribute reconstruction",
                        geometry_id
                    ))
                })?;
                AttributeValue::Geometry(handle)
            } else {
                AttributeValue::Map(
                    values
                        .iter()
                        .map(|(key, value)| {
                            Ok((key.clone(), json_to_attribute(value, geometry_handles)?))
                        })
                        .collect::<Result<HashMap<_, _>>>()?,
                )
            }
        }
    })
}

fn json_string(value: String) -> String {
    JsonValue::String(value).to_string()
}

fn metadata_batch(schema: &Arc<arrow::datatypes::Schema>, row: MetadataRow) -> Result<RecordBatch> {
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(LargeStringArray::from(vec![Some(row.citymodel_id)])),
        Arc::new(StringArray::from(vec![Some(row.cityjson_version)])),
        Arc::new(StringArray::from(vec![Some(row.citymodel_kind)])),
        Arc::new(LargeStringArray::from(vec![row.identifier])),
        Arc::new(LargeStringArray::from(vec![row.title])),
        Arc::new(LargeStringArray::from(vec![row.reference_system])),
        Arc::new(fixed_size_f64_array(
            field_from_schema(schema, "geographical_extent")?,
            6,
            vec![row.geographical_extent],
        )?),
    ];
    for value in row.projected {
        arrays.push(Arc::new(LargeStringArray::from(vec![value])));
    }
    RecordBatch::try_new(schema.clone(), arrays).map_err(Error::from)
}

fn transform_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    row: TransformRow,
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(LargeStringArray::from(vec![Some(row.citymodel_id)])),
            Arc::new(fixed_size_f64_array(
                field_from_schema(schema, "scale")?,
                3,
                vec![Some(row.scale)],
            )?),
            Arc::new(fixed_size_f64_array(
                field_from_schema(schema, "translate")?,
                3,
                vec![Some(row.translate)],
            )?),
        ],
    )
    .map_err(Error::from)
}

fn extensions_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: Vec<ExtensionRow>,
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.citymodel_id.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.extension_name.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.uri.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.into_iter().map(|row| row.version).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn vertices_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[VertexRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.citymodel_id.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.vertex_id).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.x).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.y).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.z).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn cityobjects_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[CityObjectRow],
    projection: &ProjectionLayout,
) -> Result<RecordBatch> {
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| Some(row.citymodel_id.clone()))
                .collect::<Vec<_>>(),
        )),
        Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| Some(row.cityobject_id.clone()))
                .collect::<Vec<_>>(),
        )),
        Arc::new(UInt64Array::from(
            rows.iter().map(|row| row.cityobject_ix).collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            rows.iter()
                .map(|row| Some(row.object_type.clone()))
                .collect::<Vec<_>>(),
        )),
        Arc::new(fixed_size_f64_array(
            field_from_schema(schema, "geographical_extent")?,
            6,
            rows.iter().map(|row| row.geographical_extent).collect(),
        )?),
    ];

    for column_index in 0..projection.cityobject_attributes.len() {
        arrays.push(Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| row.attributes[column_index].clone())
                .collect::<Vec<_>>(),
        )));
    }
    for column_index in 0..projection.cityobject_extra.len() {
        arrays.push(Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| row.extra[column_index].clone())
                .collect::<Vec<_>>(),
        )));
    }

    RecordBatch::try_new(schema.clone(), arrays).map_err(Error::from)
}

fn cityobject_children_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: Vec<CityObjectChildRow>,
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.citymodel_id.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.parent_cityobject_id.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter().map(|row| row.child_ordinal).collect::<Vec<_>>(),
            )),
            Arc::new(LargeStringArray::from(
                rows.into_iter()
                    .map(|row| Some(row.child_cityobject_id))
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn geometries_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[GeometryRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.citymodel_id.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.geometry_id).collect::<Vec<_>>(),
            )),
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.cityobject_id.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter()
                    .map(|row| row.geometry_ordinal)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.geometry_type.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter().map(|row| row.lod.clone()).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn geometry_boundaries_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[GeometryBoundaryRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.citymodel_id.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.geometry_id).collect::<Vec<_>>(),
            )),
            Arc::new(list_u64_array(
                field_from_schema(schema, "vertex_indices")?,
                rows.iter()
                    .map(|row| Some(row.vertex_indices.clone()))
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                field_from_schema(schema, "line_lengths")?,
                rows.iter()
                    .map(|row| row.line_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                field_from_schema(schema, "ring_lengths")?,
                rows.iter()
                    .map(|row| row.ring_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                field_from_schema(schema, "surface_lengths")?,
                rows.iter()
                    .map(|row| row.surface_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                field_from_schema(schema, "shell_lengths")?,
                rows.iter()
                    .map(|row| row.shell_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                field_from_schema(schema, "solid_lengths")?,
                rows.iter()
                    .map(|row| row.solid_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
        ],
    )
    .map_err(Error::from)
}

fn geometry_instances_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[GeometryInstanceRow],
) -> Result<RecordBatch> {
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| Some(row.citymodel_id.clone()))
                .collect::<Vec<_>>(),
        )),
        Arc::new(UInt64Array::from(
            rows.iter().map(|row| row.geometry_id).collect::<Vec<_>>(),
        )),
        Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| Some(row.cityobject_id.clone()))
                .collect::<Vec<_>>(),
        )),
        Arc::new(UInt32Array::from(
            rows.iter()
                .map(|row| row.geometry_ordinal)
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            rows.iter().map(|row| row.lod.clone()).collect::<Vec<_>>(),
        )),
        Arc::new(UInt64Array::from(
            rows.iter()
                .map(|row| row.template_geometry_id)
                .collect::<Vec<_>>(),
        )),
        Arc::new(UInt64Array::from(
            rows.iter()
                .map(|row| row.reference_point_vertex_id)
                .collect::<Vec<_>>(),
        )),
        Arc::new(fixed_size_f64_array(
            field_from_schema(schema, "transform_matrix")?,
            16,
            rows.iter().map(|row| row.transform_matrix).collect(),
        )?),
    ];
    RecordBatch::try_new(schema.clone(), arrays.split_off(0)).map_err(Error::from)
}

fn template_vertices_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[TemplateVertexRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.citymodel_id.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter()
                    .map(|row| row.template_vertex_id)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.x).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.y).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.z).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn template_geometries_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[TemplateGeometryRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.citymodel_id.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter()
                    .map(|row| row.template_geometry_id)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.geometry_type.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter().map(|row| row.lod.clone()).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn template_geometry_boundaries_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[TemplateGeometryBoundaryRow],
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.citymodel_id.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter()
                    .map(|row| row.template_geometry_id)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(list_u64_array(
                field_from_schema(schema, "vertex_indices")?,
                rows.iter()
                    .map(|row| Some(row.vertex_indices.clone()))
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                field_from_schema(schema, "line_lengths")?,
                rows.iter()
                    .map(|row| row.line_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                field_from_schema(schema, "ring_lengths")?,
                rows.iter()
                    .map(|row| row.ring_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                field_from_schema(schema, "surface_lengths")?,
                rows.iter()
                    .map(|row| row.surface_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                field_from_schema(schema, "shell_lengths")?,
                rows.iter()
                    .map(|row| row.shell_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
            Arc::new(list_u32_array(
                field_from_schema(schema, "solid_lengths")?,
                rows.iter()
                    .map(|row| row.solid_lengths.clone())
                    .collect::<Vec<_>>(),
            )?),
        ],
    )
    .map_err(Error::from)
}

fn semantics_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[SemanticRow],
    projection: &ProjectionLayout,
) -> Result<RecordBatch> {
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| Some(row.citymodel_id.clone()))
                .collect::<Vec<_>>(),
        )),
        Arc::new(UInt64Array::from(
            rows.iter().map(|row| row.semantic_id).collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            rows.iter()
                .map(|row| Some(row.semantic_type.clone()))
                .collect::<Vec<_>>(),
        )),
    ];
    for column_index in 0..projection.semantic_attributes.len() {
        arrays.push(Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| row.attributes[column_index].clone())
                .collect::<Vec<_>>(),
        )));
    }
    RecordBatch::try_new(schema.clone(), arrays).map_err(Error::from)
}

fn semantic_children_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: Vec<SemanticChildRow>,
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.citymodel_id.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter()
                    .map(|row| row.parent_semantic_id)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter().map(|row| row.child_ordinal).collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.into_iter()
                    .map(|row| row.child_semantic_id)
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn geometry_surface_semantics_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: Vec<GeometrySurfaceSemanticRow>,
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.citymodel_id.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.geometry_id).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter()
                    .map(|row| row.surface_ordinal)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.into_iter()
                    .map(|row| row.semantic_id)
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn materials_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[MaterialRow],
    projection: &ProjectionLayout,
) -> Result<RecordBatch> {
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| Some(row.citymodel_id.clone()))
                .collect::<Vec<_>>(),
        )),
        Arc::new(UInt64Array::from(
            rows.iter().map(|row| row.material_id).collect::<Vec<_>>(),
        )),
    ];
    for spec in &projection.material_payload {
        arrays.push(material_payload_array(spec, rows)?);
    }
    RecordBatch::try_new(schema.clone(), arrays).map_err(Error::from)
}

fn geometry_surface_materials_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: Vec<GeometrySurfaceMaterialRow>,
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.citymodel_id.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.geometry_id).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter()
                    .map(|row| row.surface_ordinal)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.theme.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.into_iter()
                    .map(|row| row.material_id)
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn textures_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: &[TextureRow],
    projection: &ProjectionLayout,
) -> Result<RecordBatch> {
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| Some(row.citymodel_id.clone()))
                .collect::<Vec<_>>(),
        )),
        Arc::new(UInt64Array::from(
            rows.iter().map(|row| row.texture_id).collect::<Vec<_>>(),
        )),
        Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| Some(row.image_uri.clone()))
                .collect::<Vec<_>>(),
        )),
    ];
    for spec in &projection.texture_payload {
        arrays.push(texture_payload_array(spec, rows)?);
    }
    RecordBatch::try_new(schema.clone(), arrays).map_err(Error::from)
}

fn texture_vertices_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: Vec<TextureVertexRow>,
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.citymodel_id.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.uv_id).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.u).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.into_iter().map(|row| row.v).collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(Error::from)
}

fn geometry_ring_textures_batch(
    schema: &Arc<arrow::datatypes::Schema>,
    rows: Vec<GeometryRingTextureRow>,
) -> Result<RecordBatch> {
    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(LargeStringArray::from(
                rows.iter()
                    .map(|row| Some(row.citymodel_id.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.geometry_id).collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter()
                    .map(|row| row.surface_ordinal)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt32Array::from(
                rows.iter().map(|row| row.ring_ordinal).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| Some(row.theme.clone()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.texture_id).collect::<Vec<_>>(),
            )),
            Arc::new(list_u64_array(
                field_from_schema(schema, "uv_indices")?,
                rows.into_iter()
                    .map(|row| Some(row.uv_indices))
                    .collect::<Vec<_>>(),
            )?),
        ],
    )
    .map_err(Error::from)
}

fn material_payload_array(spec: &ProjectedFieldSpec, rows: &[MaterialRow]) -> Result<ArrayRef> {
    Ok(match spec.name.as_str() {
        FIELD_MATERIAL_NAME => Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| Some(row.name.clone()))
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_MATERIAL_AMBIENT_INTENSITY => Arc::new(Float64Array::from(
            rows.iter()
                .map(|row| row.ambient_intensity)
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_MATERIAL_DIFFUSE_COLOR => Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| row.diffuse_color.clone())
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_MATERIAL_EMISSIVE_COLOR => Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| row.emissive_color.clone())
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_MATERIAL_SPECULAR_COLOR => Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| row.specular_color.clone())
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_MATERIAL_SHININESS => Arc::new(Float64Array::from(
            rows.iter().map(|row| row.shininess).collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_MATERIAL_TRANSPARENCY => Arc::new(Float64Array::from(
            rows.iter().map(|row| row.transparency).collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_MATERIAL_IS_SMOOTH => Arc::new(arrow::array::BooleanArray::from(
            rows.iter().map(|row| row.is_smooth).collect::<Vec<_>>(),
        )) as ArrayRef,
        other => {
            return Err(Error::Conversion(format!(
                "unsupported material projection column {other}"
            )));
        }
    })
}

fn texture_payload_array(spec: &ProjectedFieldSpec, rows: &[TextureRow]) -> Result<ArrayRef> {
    Ok(match spec.name.as_str() {
        FIELD_TEXTURE_IMAGE_TYPE => Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| Some(row.image_type.clone()))
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_TEXTURE_WRAP_MODE => Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| row.wrap_mode.clone())
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_TEXTURE_TEXTURE_TYPE => Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| row.texture_type.clone())
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        FIELD_TEXTURE_BORDER_COLOR => Arc::new(LargeStringArray::from(
            rows.iter()
                .map(|row| row.border_color.clone())
                .collect::<Vec<_>>(),
        )) as ArrayRef,
        other => {
            return Err(Error::Conversion(format!(
                "unsupported texture projection column {other}"
            )));
        }
    })
}

fn optional_batch<T, F>(rows: Vec<T>, build: F) -> Result<Option<RecordBatch>>
where
    F: FnOnce(Vec<T>) -> Result<RecordBatch>,
{
    if rows.is_empty() {
        Ok(None)
    } else {
        build(rows).map(Some)
    }
}

fn field_from_schema(schema: &Arc<arrow::datatypes::Schema>, name: &str) -> Result<FieldRef> {
    Ok(Arc::new(schema.field_with_name(name)?.clone()))
}

fn fixed_size_f64_array<const N: usize>(
    field: FieldRef,
    size: i32,
    rows: Vec<Option<[f64; N]>>,
) -> Result<FixedSizeListArray> {
    let mut flat = Vec::with_capacity(rows.len() * N);
    let mut validity = Vec::with_capacity(rows.len());
    for row in rows {
        match row {
            Some(values) => {
                flat.extend(values);
                validity.push(true);
            }
            None => {
                flat.extend(std::iter::repeat_n(0.0, N));
                validity.push(false);
            }
        }
    }
    let values: ArrayRef = Arc::new(Float64Array::from(flat));
    let nulls = if validity.iter().all(|item| *item) {
        None
    } else {
        Some(NullBuffer::from(validity))
    };
    FixedSizeListArray::try_new(fixed_list_child_field(&field)?, size, values, nulls)
        .map_err(Error::from)
}

fn list_u64_array(field: FieldRef, rows: Vec<Option<Vec<u64>>>) -> Result<ListArray> {
    let mut offsets = vec![0_i32];
    let mut flat: Vec<u64> = Vec::new();
    let mut validity = Vec::with_capacity(rows.len());
    for row in rows {
        match row {
            Some(values) => {
                flat.extend(&values);
                offsets.push(flat.len() as i32);
                validity.push(true);
            }
            None => {
                offsets.push(flat.len() as i32);
                validity.push(false);
            }
        }
    }
    let values: ArrayRef = Arc::new(UInt64Array::from(flat));
    let nulls = if validity.iter().all(|item| *item) {
        None
    } else {
        Some(NullBuffer::from(validity))
    };
    ListArray::try_new(
        list_child_field(&field)?,
        OffsetBuffer::new(ScalarBuffer::from(offsets)),
        values,
        nulls,
    )
    .map_err(Error::from)
}

fn list_u32_array(field: FieldRef, rows: Vec<Option<Vec<u32>>>) -> Result<ListArray> {
    let mut offsets = vec![0_i32];
    let mut flat: Vec<u32> = Vec::new();
    let mut validity = Vec::with_capacity(rows.len());
    for row in rows {
        match row {
            Some(values) => {
                flat.extend(&values);
                offsets.push(flat.len() as i32);
                validity.push(true);
            }
            None => {
                offsets.push(flat.len() as i32);
                validity.push(false);
            }
        }
    }
    let values: ArrayRef = Arc::new(UInt32Array::from(flat));
    let nulls = if validity.iter().all(|item| *item) {
        None
    } else {
        Some(NullBuffer::from(validity))
    };
    ListArray::try_new(
        list_child_field(&field)?,
        OffsetBuffer::new(ScalarBuffer::from(offsets)),
        values,
        nulls,
    )
    .map_err(Error::from)
}

fn fixed_list_child_field(field: &FieldRef) -> Result<FieldRef> {
    match field.data_type() {
        DataType::FixedSizeList(child, _) => Ok(child.clone()),
        other => Err(Error::Conversion(format!(
            "expected fixed size list field, found {other:?}"
        ))),
    }
}

fn list_child_field(field: &FieldRef) -> Result<FieldRef> {
    match field.data_type() {
        DataType::List(child) => Ok(child.clone()),
        other => Err(Error::Conversion(format!(
            "expected list field, found {other:?}"
        ))),
    }
}

fn read_metadata_row(batch: &RecordBatch, projection: &ProjectionLayout) -> Result<MetadataRow> {
    Ok(MetadataRow {
        citymodel_id: read_large_string_scalar(batch, "citymodel_id", 0)?,
        cityjson_version: read_string_scalar(batch, "cityjson_version", 0)?,
        citymodel_kind: read_string_scalar(batch, "citymodel_kind", 0)?,
        identifier: read_large_string_optional(batch, "identifier", 0)?,
        title: read_large_string_optional(batch, "title", 0)?,
        reference_system: read_large_string_optional(batch, "reference_system", 0)?,
        geographical_extent: read_fixed_size_f64_optional::<6>(batch, "geographical_extent", 0)?,
        projected: projection
            .metadata_extra
            .iter()
            .map(|spec| read_large_string_optional(batch, &spec.name, 0))
            .collect::<Result<Vec<_>>>()?,
    })
}

fn read_transform_row(batch: &RecordBatch) -> Result<TransformRow> {
    Ok(TransformRow {
        citymodel_id: read_large_string_scalar(batch, "citymodel_id", 0)?,
        scale: read_fixed_size_f64_required::<3>(batch, "scale", 0)?,
        translate: read_fixed_size_f64_required::<3>(batch, "translate", 0)?,
    })
}

fn read_extension_rows(batch: &RecordBatch) -> Result<Vec<ExtensionRow>> {
    (0..batch.num_rows())
        .map(|row| {
            Ok(ExtensionRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                extension_name: read_string_scalar(batch, "extension_name", row)?,
                uri: read_large_string_scalar(batch, "uri", row)?,
                version: read_string_optional(batch, "version", row)?,
            })
        })
        .collect()
}

fn read_vertex_rows(batch: &RecordBatch) -> Result<Vec<VertexRow>> {
    let vertex_ids = downcast_required::<UInt64Array>(batch, "vertex_id")?;
    let xs = downcast_required::<Float64Array>(batch, "x")?;
    let ys = downcast_required::<Float64Array>(batch, "y")?;
    let zs = downcast_required::<Float64Array>(batch, "z")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(VertexRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                vertex_id: vertex_ids.value(row),
                x: xs.value(row),
                y: ys.value(row),
                z: zs.value(row),
            })
        })
        .collect()
}

fn read_cityobject_rows(
    batch: &RecordBatch,
    projection: &ProjectionLayout,
) -> Result<Vec<CityObjectRow>> {
    let object_ixs = downcast_required::<UInt64Array>(batch, "cityobject_ix")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(CityObjectRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                cityobject_id: read_large_string_scalar(batch, "cityobject_id", row)?,
                cityobject_ix: object_ixs.value(row),
                object_type: read_string_scalar(batch, "object_type", row)?,
                geographical_extent: read_fixed_size_f64_optional::<6>(
                    batch,
                    "geographical_extent",
                    row,
                )?,
                attributes: projection
                    .cityobject_attributes
                    .iter()
                    .map(|spec| read_large_string_optional(batch, &spec.name, row))
                    .collect::<Result<Vec<_>>>()?,
                extra: projection
                    .cityobject_extra
                    .iter()
                    .map(|spec| read_large_string_optional(batch, &spec.name, row))
                    .collect::<Result<Vec<_>>>()?,
            })
        })
        .collect()
}

fn read_cityobject_child_rows(batch: &RecordBatch) -> Result<Vec<CityObjectChildRow>> {
    let ordinals = downcast_required::<UInt32Array>(batch, "child_ordinal")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(CityObjectChildRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                parent_cityobject_id: read_large_string_scalar(batch, "parent_cityobject_id", row)?,
                child_ordinal: ordinals.value(row),
                child_cityobject_id: read_large_string_scalar(batch, "child_cityobject_id", row)?,
            })
        })
        .collect()
}

fn read_geometry_rows(batch: &RecordBatch) -> Result<Vec<GeometryRow>> {
    let geometry_ids = downcast_required::<UInt64Array>(batch, "geometry_id")?;
    let ordinals = downcast_required::<UInt32Array>(batch, "geometry_ordinal")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(GeometryRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                geometry_id: geometry_ids.value(row),
                cityobject_id: read_large_string_scalar(batch, "cityobject_id", row)?,
                geometry_ordinal: ordinals.value(row),
                geometry_type: read_string_scalar(batch, "geometry_type", row)?,
                lod: read_string_optional(batch, "lod", row)?,
            })
        })
        .collect()
}

fn read_geometry_boundary_rows(batch: &RecordBatch) -> Result<Vec<GeometryBoundaryRow>> {
    let geometry_ids = downcast_required::<UInt64Array>(batch, "geometry_id")?;
    let vertices = downcast_required::<ListArray>(batch, "vertex_indices")?;
    let lines = downcast_required::<ListArray>(batch, "line_lengths")?;
    let rings = downcast_required::<ListArray>(batch, "ring_lengths")?;
    let surfaces = downcast_required::<ListArray>(batch, "surface_lengths")?;
    let shells = downcast_required::<ListArray>(batch, "shell_lengths")?;
    let solids = downcast_required::<ListArray>(batch, "solid_lengths")?;

    (0..batch.num_rows())
        .map(|row| {
            Ok(GeometryBoundaryRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                geometry_id: geometry_ids.value(row),
                vertex_indices: list_u64_value(vertices, row)?,
                line_lengths: list_u32_optional_value(lines, row)?,
                ring_lengths: list_u32_optional_value(rings, row)?,
                surface_lengths: list_u32_optional_value(surfaces, row)?,
                shell_lengths: list_u32_optional_value(shells, row)?,
                solid_lengths: list_u32_optional_value(solids, row)?,
            })
        })
        .collect()
}

fn read_geometry_instance_rows(batch: &RecordBatch) -> Result<Vec<GeometryInstanceRow>> {
    let geometry_ids = downcast_required::<UInt64Array>(batch, "geometry_id")?;
    let ordinals = downcast_required::<UInt32Array>(batch, "geometry_ordinal")?;
    let template_geometry_ids = downcast_required::<UInt64Array>(batch, "template_geometry_id")?;
    let reference_point_vertex_ids =
        downcast_required::<UInt64Array>(batch, "reference_point_vertex_id")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(GeometryInstanceRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                geometry_id: geometry_ids.value(row),
                cityobject_id: read_large_string_scalar(batch, "cityobject_id", row)?,
                geometry_ordinal: ordinals.value(row),
                lod: read_string_optional(batch, "lod", row)?,
                template_geometry_id: template_geometry_ids.value(row),
                reference_point_vertex_id: reference_point_vertex_ids.value(row),
                transform_matrix: read_fixed_size_f64_optional::<16>(
                    batch,
                    "transform_matrix",
                    row,
                )?,
            })
        })
        .collect()
}

fn read_template_vertex_rows(batch: &RecordBatch) -> Result<Vec<TemplateVertexRow>> {
    let vertex_ids = downcast_required::<UInt64Array>(batch, "template_vertex_id")?;
    let xs = downcast_required::<Float64Array>(batch, "x")?;
    let ys = downcast_required::<Float64Array>(batch, "y")?;
    let zs = downcast_required::<Float64Array>(batch, "z")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(TemplateVertexRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                template_vertex_id: vertex_ids.value(row),
                x: xs.value(row),
                y: ys.value(row),
                z: zs.value(row),
            })
        })
        .collect()
}

fn read_template_geometry_rows(batch: &RecordBatch) -> Result<Vec<TemplateGeometryRow>> {
    let geometry_ids = downcast_required::<UInt64Array>(batch, "template_geometry_id")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(TemplateGeometryRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                template_geometry_id: geometry_ids.value(row),
                geometry_type: read_string_scalar(batch, "geometry_type", row)?,
                lod: read_string_optional(batch, "lod", row)?,
            })
        })
        .collect()
}

fn read_template_geometry_boundary_rows(
    batch: &RecordBatch,
) -> Result<Vec<TemplateGeometryBoundaryRow>> {
    let geometry_ids = downcast_required::<UInt64Array>(batch, "template_geometry_id")?;
    let vertices = downcast_required::<ListArray>(batch, "vertex_indices")?;
    let lines = downcast_required::<ListArray>(batch, "line_lengths")?;
    let rings = downcast_required::<ListArray>(batch, "ring_lengths")?;
    let surfaces = downcast_required::<ListArray>(batch, "surface_lengths")?;
    let shells = downcast_required::<ListArray>(batch, "shell_lengths")?;
    let solids = downcast_required::<ListArray>(batch, "solid_lengths")?;

    (0..batch.num_rows())
        .map(|row| {
            Ok(TemplateGeometryBoundaryRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                template_geometry_id: geometry_ids.value(row),
                vertex_indices: list_u64_value(vertices, row)?,
                line_lengths: list_u32_optional_value(lines, row)?,
                ring_lengths: list_u32_optional_value(rings, row)?,
                surface_lengths: list_u32_optional_value(surfaces, row)?,
                shell_lengths: list_u32_optional_value(shells, row)?,
                solid_lengths: list_u32_optional_value(solids, row)?,
            })
        })
        .collect()
}

fn read_semantic_rows(
    batch: &RecordBatch,
    projection: &ProjectionLayout,
) -> Result<Vec<SemanticRow>> {
    let semantic_ids = downcast_required::<UInt64Array>(batch, "semantic_id")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(SemanticRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                semantic_id: semantic_ids.value(row),
                semantic_type: read_string_scalar(batch, "semantic_type", row)?,
                attributes: projection
                    .semantic_attributes
                    .iter()
                    .map(|spec| read_large_string_optional(batch, &spec.name, row))
                    .collect::<Result<Vec<_>>>()?,
            })
        })
        .collect()
}

fn read_semantic_child_rows(batch: &RecordBatch) -> Result<Vec<SemanticChildRow>> {
    let parents = downcast_required::<UInt64Array>(batch, "parent_semantic_id")?;
    let ordinals = downcast_required::<UInt32Array>(batch, "child_ordinal")?;
    let children = downcast_required::<UInt64Array>(batch, "child_semantic_id")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(SemanticChildRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                parent_semantic_id: parents.value(row),
                child_ordinal: ordinals.value(row),
                child_semantic_id: children.value(row),
            })
        })
        .collect()
}

fn read_geometry_surface_semantic_rows(
    batch: &RecordBatch,
) -> Result<Vec<GeometrySurfaceSemanticRow>> {
    let geometries = downcast_required::<UInt64Array>(batch, "geometry_id")?;
    let ordinals = downcast_required::<UInt32Array>(batch, "surface_ordinal")?;
    let semantics = downcast_required::<UInt64Array>(batch, "semantic_id")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(GeometrySurfaceSemanticRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                geometry_id: geometries.value(row),
                surface_ordinal: ordinals.value(row),
                semantic_id: if semantics.is_null(row) {
                    None
                } else {
                    Some(semantics.value(row))
                },
            })
        })
        .collect()
}

fn read_material_rows(
    batch: &RecordBatch,
    projection: &ProjectionLayout,
) -> Result<Vec<MaterialRow>> {
    let material_ids = downcast_required::<UInt64Array>(batch, "material_id")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(MaterialRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                material_id: material_ids.value(row),
                name: read_required_large_string_projection(
                    batch,
                    &projection.material_payload,
                    FIELD_MATERIAL_NAME,
                    row,
                )?,
                ambient_intensity: read_optional_float_projection(
                    batch,
                    &projection.material_payload,
                    FIELD_MATERIAL_AMBIENT_INTENSITY,
                    row,
                )?,
                diffuse_color: read_optional_large_string_projection(
                    batch,
                    &projection.material_payload,
                    FIELD_MATERIAL_DIFFUSE_COLOR,
                    row,
                )?,
                emissive_color: read_optional_large_string_projection(
                    batch,
                    &projection.material_payload,
                    FIELD_MATERIAL_EMISSIVE_COLOR,
                    row,
                )?,
                specular_color: read_optional_large_string_projection(
                    batch,
                    &projection.material_payload,
                    FIELD_MATERIAL_SPECULAR_COLOR,
                    row,
                )?,
                shininess: read_optional_float_projection(
                    batch,
                    &projection.material_payload,
                    FIELD_MATERIAL_SHININESS,
                    row,
                )?,
                transparency: read_optional_float_projection(
                    batch,
                    &projection.material_payload,
                    FIELD_MATERIAL_TRANSPARENCY,
                    row,
                )?,
                is_smooth: read_optional_bool_projection(
                    batch,
                    &projection.material_payload,
                    FIELD_MATERIAL_IS_SMOOTH,
                    row,
                )?,
            })
        })
        .collect()
}

fn read_geometry_surface_material_rows(
    batch: &RecordBatch,
) -> Result<Vec<GeometrySurfaceMaterialRow>> {
    let geometries = downcast_required::<UInt64Array>(batch, "geometry_id")?;
    let ordinals = downcast_required::<UInt32Array>(batch, "surface_ordinal")?;
    let materials = downcast_required::<UInt64Array>(batch, "material_id")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(GeometrySurfaceMaterialRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                geometry_id: geometries.value(row),
                surface_ordinal: ordinals.value(row),
                theme: read_string_scalar(batch, "theme", row)?,
                material_id: materials.value(row),
            })
        })
        .collect()
}

fn read_texture_rows(
    batch: &RecordBatch,
    projection: &ProjectionLayout,
) -> Result<Vec<TextureRow>> {
    let texture_ids = downcast_required::<UInt64Array>(batch, "texture_id")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(TextureRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                texture_id: texture_ids.value(row),
                image_uri: read_large_string_scalar(batch, "image_uri", row)?,
                image_type: read_required_large_string_projection(
                    batch,
                    &projection.texture_payload,
                    FIELD_TEXTURE_IMAGE_TYPE,
                    row,
                )?,
                wrap_mode: read_optional_large_string_projection(
                    batch,
                    &projection.texture_payload,
                    FIELD_TEXTURE_WRAP_MODE,
                    row,
                )?,
                texture_type: read_optional_large_string_projection(
                    batch,
                    &projection.texture_payload,
                    FIELD_TEXTURE_TEXTURE_TYPE,
                    row,
                )?,
                border_color: read_optional_large_string_projection(
                    batch,
                    &projection.texture_payload,
                    FIELD_TEXTURE_BORDER_COLOR,
                    row,
                )?,
            })
        })
        .collect()
}

fn read_texture_vertex_rows(batch: &RecordBatch) -> Result<Vec<TextureVertexRow>> {
    let uv_ids = downcast_required::<UInt64Array>(batch, "uv_id")?;
    let us = downcast_required::<Float64Array>(batch, "u")?;
    let vs = downcast_required::<Float64Array>(batch, "v")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(TextureVertexRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                uv_id: uv_ids.value(row),
                u: us.value(row),
                v: vs.value(row),
            })
        })
        .collect()
}

fn read_geometry_ring_texture_rows(batch: &RecordBatch) -> Result<Vec<GeometryRingTextureRow>> {
    let geometries = downcast_required::<UInt64Array>(batch, "geometry_id")?;
    let surface_ordinals = downcast_required::<UInt32Array>(batch, "surface_ordinal")?;
    let ring_ordinals = downcast_required::<UInt32Array>(batch, "ring_ordinal")?;
    let textures = downcast_required::<UInt64Array>(batch, "texture_id")?;
    let uv_indices = downcast_required::<ListArray>(batch, "uv_indices")?;
    (0..batch.num_rows())
        .map(|row| {
            Ok(GeometryRingTextureRow {
                citymodel_id: read_large_string_scalar(batch, "citymodel_id", row)?,
                geometry_id: geometries.value(row),
                surface_ordinal: surface_ordinals.value(row),
                ring_ordinal: ring_ordinals.value(row),
                theme: read_string_scalar(batch, "theme", row)?,
                texture_id: textures.value(row),
                uv_indices: list_u64_value(uv_indices, row)?,
            })
        })
        .collect()
}

fn apply_metadata_row(
    model: &mut OwnedCityModel,
    row: &MetadataRow,
    projection: &ProjectionLayout,
    geometry_handles: &HashMap<u64, cityjson::prelude::GeometryHandle>,
) -> Result<()> {
    if let Some(identifier) = &row.identifier {
        model
            .metadata_mut()
            .set_identifier(CityModelIdentifier::new(identifier.clone()));
    }
    if let Some(title) = &row.title {
        model.metadata_mut().set_title(title.clone());
    }
    if let Some(reference_system) = &row.reference_system {
        model
            .metadata_mut()
            .set_reference_system(CRS::new(reference_system.clone()));
    }
    if let Some(extent) = row.geographical_extent {
        model
            .metadata_mut()
            .set_geographical_extent(BBox::from(extent));
    }

    for (spec, value) in projection.metadata_extra.iter().zip(&row.projected) {
        let Some(value) = value else {
            continue;
        };
        let json: JsonValue = serde_json::from_str(value)?;
        if spec.name == FIELD_METADATA_REFERENCE_DATE {
            let date = json.as_str().ok_or_else(|| {
                Error::Conversion("metadata referenceDate must be a JSON string".to_string())
            })?;
            model
                .metadata_mut()
                .set_reference_date(cityjson::v2_0::Date::new(date.to_string()));
            continue;
        }
        if spec.name == FIELD_METADATA_POINT_OF_CONTACT {
            let contact = contact_from_json(&json, geometry_handles)?;
            model.metadata_mut().set_point_of_contact(Some(contact));
            continue;
        }
        if spec.name == FIELD_METADATA_DEFAULT_MATERIAL_THEME {
            let theme = json.as_str().ok_or_else(|| {
                Error::Conversion("default material theme must be a JSON string".to_string())
            })?;
            model.set_default_material_theme(Some(ThemeName::new(theme.to_string())));
            continue;
        }
        if spec.name == FIELD_METADATA_DEFAULT_TEXTURE_THEME {
            let theme = json.as_str().ok_or_else(|| {
                Error::Conversion("default texture theme must be a JSON string".to_string())
            })?;
            model.set_default_texture_theme(Some(ThemeName::new(theme.to_string())));
            continue;
        }
        if let Some(key) = decode_projection_name(&spec.name, FIELD_ROOT_EXTRA_PREFIX) {
            model
                .extra_mut()
                .insert(key, json_to_attribute(&json, geometry_handles)?);
            continue;
        }
        if let Some(key) = decode_projection_name(&spec.name, FIELD_METADATA_EXTRA_PREFIX) {
            model
                .metadata_mut()
                .extra_mut()
                .insert(key, json_to_attribute(&json, geometry_handles)?);
            continue;
        }
        return Err(Error::Conversion(format!(
            "unrecognized metadata projection column {}",
            spec.name
        )));
    }

    Ok(())
}

fn apply_projected_attributes(
    target: &mut cityjson::v2_0::OwnedAttributes,
    specs: &[ProjectedFieldSpec],
    values: &[Option<String>],
    prefix: &str,
    geometry_handles: &HashMap<u64, cityjson::prelude::GeometryHandle>,
) -> Result<()> {
    for (spec, value) in specs.iter().zip(values) {
        if let Some(value) = value {
            let key = decode_projection_name(&spec.name, prefix).ok_or_else(|| {
                Error::Conversion(format!("invalid projection column {}", spec.name))
            })?;
            let json: JsonValue = serde_json::from_str(value)?;
            target.insert(key, json_to_attribute(&json, geometry_handles)?);
        }
    }
    Ok(())
}

fn build_semantic_map(
    geometry_type: &str,
    boundary: &GeometryBoundaryRow,
    rows: Option<&Vec<GeometrySurfaceSemanticRow>>,
    handles: &HashMap<u64, cityjson::prelude::SemanticHandle>,
) -> Result<Option<SemanticMap<u32>>> {
    let Some(rows) = rows else {
        return Ok(None);
    };
    if rows.is_empty() {
        return Ok(None);
    }
    match parse_geometry_type(geometry_type)? {
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => {
            let surface_count = surface_count(boundary)?;
            let mut rows = rows.clone();
            rows.sort_by_key(|row| row.surface_ordinal);
            if rows.len() != surface_count {
                return Err(Error::Conversion(format!(
                    "surface semantic row count {} does not match surface count {}",
                    rows.len(),
                    surface_count
                )));
            }
            let mut map = SemanticMap::new();
            for row in rows {
                map.add_surface(row.semantic_id.and_then(|id| handles.get(&id).copied()));
            }
            Ok(Some(map))
        }
        GeometryType::MultiPoint | GeometryType::MultiLineString => Err(Error::Unsupported(
            "point and linestring semantic mappings".to_string(),
        )),
        GeometryType::GeometryInstance => Err(Error::Unsupported("geometry instances".to_string())),
        _ => Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
}

fn build_material_maps(
    geometry_type: &str,
    boundary: &GeometryBoundaryRow,
    rows: Option<&Vec<GeometrySurfaceMaterialRow>>,
    handles: &HashMap<u64, cityjson::prelude::MaterialHandle>,
) -> Result<
    Option<
        Vec<(
            ThemeName<cityjson::prelude::OwnedStringStorage>,
            MaterialMap<u32>,
        )>,
    >,
> {
    let Some(rows) = rows else {
        return Ok(None);
    };
    if rows.is_empty() {
        return Ok(None);
    }
    match parse_geometry_type(geometry_type)? {
        GeometryType::MultiSurface
        | GeometryType::CompositeSurface
        | GeometryType::Solid
        | GeometryType::MultiSolid
        | GeometryType::CompositeSolid => {
            let surface_count = surface_count(boundary)?;
            let mut grouped =
                BTreeMap::<String, Vec<Option<cityjson::prelude::MaterialHandle>>>::new();
            for row in rows {
                if row.surface_ordinal as usize >= surface_count {
                    return Err(Error::Conversion(format!(
                        "material assignment surface ordinal {} exceeds surface count {}",
                        row.surface_ordinal, surface_count
                    )));
                }
                let material = *handles.get(&row.material_id).ok_or_else(|| {
                    Error::Conversion(format!("missing material {}", row.material_id))
                })?;
                let surfaces = grouped
                    .entry(row.theme.clone())
                    .or_insert_with(|| vec![None; surface_count]);
                if surfaces[row.surface_ordinal as usize].is_some() {
                    return Err(Error::Conversion(format!(
                        "duplicate material assignment for theme {} surface {}",
                        row.theme, row.surface_ordinal
                    )));
                }
                surfaces[row.surface_ordinal as usize] = Some(material);
            }
            Ok(Some(
                grouped
                    .into_iter()
                    .map(|(theme, surfaces)| {
                        let mut map = MaterialMap::new();
                        for surface in surfaces {
                            map.add_surface(surface);
                        }
                        (ThemeName::new(theme), map)
                    })
                    .collect(),
            ))
        }
        GeometryType::MultiPoint | GeometryType::MultiLineString => Err(Error::Unsupported(
            "point and linestring material mappings".to_string(),
        )),
        GeometryType::GeometryInstance => Err(Error::Unsupported("geometry materials".to_string())),
        _ => Err(Error::Unsupported("unsupported geometry type".to_string())),
    }
}

fn build_texture_maps(
    geometry_type: &str,
    boundary: &GeometryBoundaryRow,
    rows: Option<&Vec<GeometryRingTextureRow>>,
    handles: &HashMap<u64, cityjson::prelude::TextureHandle>,
) -> Result<
    Option<
        Vec<(
            ThemeName<cityjson::prelude::OwnedStringStorage>,
            TextureMap<u32>,
        )>,
    >,
> {
    let Some(rows) = rows else {
        return Ok(None);
    };
    if rows.is_empty() {
        return Ok(None);
    }
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

            for row in rows {
                let layout_index = *ring_lookup
                    .get(&(row.surface_ordinal, row.ring_ordinal))
                    .ok_or_else(|| {
                        Error::Conversion(format!(
                            "missing ring layout for surface {} ring {}",
                            row.surface_ordinal, row.ring_ordinal
                        ))
                    })?;
                let layout = ring_layouts[layout_index];
                if row.uv_indices.len() != layout.len {
                    return Err(Error::Conversion(format!(
                        "texture assignment for theme {} surface {} ring {} has {} uv indices, expected {}",
                        row.theme,
                        row.surface_ordinal,
                        row.ring_ordinal,
                        row.uv_indices.len(),
                        layout.len
                    )));
                }
                let texture = *handles.get(&row.texture_id).ok_or_else(|| {
                    Error::Conversion(format!("missing texture {}", row.texture_id))
                })?;
                let map = maps.entry(row.theme.clone()).or_insert_with(|| {
                    let mut map = TextureMap::new();
                    for layout in &ring_layouts {
                        map.add_ring(cityjson::v2_0::VertexIndex::new(layout.start as u32));
                        map.add_ring_texture(None);
                        for _ in 0..layout.len {
                            map.add_vertex(None);
                        }
                    }
                    map
                });
                if map.ring_textures()[layout_index].is_some() {
                    return Err(Error::Conversion(format!(
                        "duplicate texture assignment for theme {} surface {} ring {}",
                        row.theme, row.surface_ordinal, row.ring_ordinal
                    )));
                }
                if !map.set_ring_texture(layout_index, Some(texture)) {
                    return Err(Error::Conversion("missing texture ring slot".to_string()));
                }
                let slice = map.vertices_mut()[layout.start..layout.start + layout.len].iter_mut();
                for (slot, uv_id) in slice.zip(&row.uv_indices) {
                    let uv_id = u32::try_from(*uv_id).map_err(|_| {
                        Error::Conversion(format!("uv index {} does not fit into u32", uv_id))
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

fn boundary_from_row(row: &GeometryBoundaryRow, geometry_type: &str) -> Result<Boundary<u32>> {
    boundary_from_parts(
        &row.vertex_indices,
        &row.line_lengths,
        &row.ring_lengths,
        &row.surface_lengths,
        &row.shell_lengths,
        &row.solid_lengths,
        geometry_type,
    )
}

fn template_boundary_from_row(
    row: &TemplateGeometryBoundaryRow,
    geometry_type: &str,
) -> Result<Boundary<u32>> {
    boundary_from_parts(
        &row.vertex_indices,
        &row.line_lengths,
        &row.ring_lengths,
        &row.surface_lengths,
        &row.shell_lengths,
        &row.solid_lengths,
        geometry_type,
    )
}

fn boundary_from_parts(
    vertex_indices: &[u64],
    line_lengths: &Option<Vec<u32>>,
    ring_lengths: &Option<Vec<u32>>,
    surface_lengths: &Option<Vec<u32>>,
    shell_lengths: &Option<Vec<u32>>,
    solid_lengths: &Option<Vec<u32>>,
    geometry_type: &str,
) -> Result<Boundary<u32>> {
    let vertices = vertex_indices
        .iter()
        .map(|value| {
            u32::try_from(*value).map_err(|_| {
                Error::Conversion(format!("vertex index {} does not fit into u32", value))
            })
        })
        .collect::<Result<Vec<_>>>()?
        .to_vertex_indices();

    let boundary = match parse_geometry_type(geometry_type)? {
        GeometryType::MultiPoint => Boundary::from_parts(vertices, vec![], vec![], vec![], vec![])?,
        GeometryType::MultiLineString => Boundary::from_parts(
            vertices,
            lengths_to_offsets(required_lengths(line_lengths, "line_lengths")?)?,
            vec![],
            vec![],
            vec![],
        )?,
        GeometryType::MultiSurface | GeometryType::CompositeSurface => Boundary::from_parts(
            vertices,
            lengths_to_offsets(required_lengths(ring_lengths, "ring_lengths")?)?,
            lengths_to_offsets(required_lengths(surface_lengths, "surface_lengths")?)?,
            vec![],
            vec![],
        )?,
        GeometryType::Solid => Boundary::from_parts(
            vertices,
            lengths_to_offsets(required_lengths(ring_lengths, "ring_lengths")?)?,
            lengths_to_offsets(required_lengths(surface_lengths, "surface_lengths")?)?,
            lengths_to_offsets(required_lengths(shell_lengths, "shell_lengths")?)?,
            vec![],
        )?,
        GeometryType::MultiSolid | GeometryType::CompositeSolid => Boundary::from_parts(
            vertices,
            lengths_to_offsets(required_lengths(ring_lengths, "ring_lengths")?)?,
            lengths_to_offsets(required_lengths(surface_lengths, "surface_lengths")?)?,
            lengths_to_offsets(required_lengths(shell_lengths, "shell_lengths")?)?,
            lengths_to_offsets(required_lengths(solid_lengths, "solid_lengths")?)?,
        )?,
        GeometryType::GeometryInstance => {
            return Err(Error::Unsupported("geometry instances".to_string()));
        }
        _ => {
            return Err(Error::Unsupported("unsupported geometry type".to_string()));
        }
    };
    Ok(boundary)
}

fn required_lengths<'a>(value: &'a Option<Vec<u32>>, name: &str) -> Result<&'a [u32]> {
    value
        .as_deref()
        .ok_or_else(|| Error::Conversion(format!("missing required {name}")))
}

fn lengths_to_offsets(lengths: &[u32]) -> Result<Vec<cityjson::v2_0::VertexIndex<u32>>> {
    if lengths.is_empty() {
        return Ok(Vec::<u32>::new().to_vertex_indices());
    }
    let mut offsets = Vec::with_capacity(lengths.len());
    let mut total = 0_u32;
    offsets.push(0);
    for length in &lengths[..lengths.len() - 1] {
        total = total
            .checked_add(*length)
            .ok_or_else(|| Error::Conversion("length offsets overflow u32".to_string()))?;
        offsets.push(total);
    }
    Ok(offsets.to_vertex_indices())
}

fn surface_count(row: &GeometryBoundaryRow) -> Result<usize> {
    Ok(match row.surface_lengths.as_ref() {
        Some(lengths) => lengths.len(),
        None => 0,
    })
}

fn encode_key(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || byte == b'_' {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("_x{:02X}_", byte));
        }
    }
    encoded
}

fn decode_key(value: &str) -> Result<String> {
    let mut decoded = String::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'_'
            && index + 5 < bytes.len()
            && bytes[index + 1] == b'x'
            && bytes[index + 4] == b'_'
        {
            let hex = &value[index + 2..index + 4];
            let byte = u8::from_str_radix(hex, 16)
                .map_err(|_| Error::Conversion(format!("invalid encoded key segment {hex}")))?;
            decoded.push(byte as char);
            index += 5;
        } else {
            decoded.push(bytes[index] as char);
            index += 1;
        }
    }
    Ok(decoded)
}

fn decode_projection_name(name: &str, prefix: &str) -> Option<String> {
    name.strip_prefix(prefix)
        .and_then(|value| value.strip_suffix(FIELD_JSON_SUFFIX))
        .and_then(|value| decode_key(value).ok())
}

fn has_projection_field(specs: &[ProjectedFieldSpec], name: &str) -> bool {
    specs.iter().any(|spec| spec.name == name)
}

fn read_required_large_string_projection(
    batch: &RecordBatch,
    specs: &[ProjectedFieldSpec],
    name: &str,
    row: usize,
) -> Result<String> {
    if !has_projection_field(specs, name) {
        return Err(Error::Conversion(format!(
            "missing required projection column {name}"
        )));
    }
    read_large_string_scalar(batch, name, row)
}

fn read_optional_large_string_projection(
    batch: &RecordBatch,
    specs: &[ProjectedFieldSpec],
    name: &str,
    row: usize,
) -> Result<Option<String>> {
    if !has_projection_field(specs, name) {
        return Ok(None);
    }
    read_large_string_optional(batch, name, row)
}

fn read_optional_float_projection(
    batch: &RecordBatch,
    specs: &[ProjectedFieldSpec],
    name: &str,
    row: usize,
) -> Result<Option<f64>> {
    if !has_projection_field(specs, name) {
        return Ok(None);
    }
    read_f64_optional(batch, name, row)
}

fn read_optional_bool_projection(
    batch: &RecordBatch,
    specs: &[ProjectedFieldSpec],
    name: &str,
    row: usize,
) -> Result<Option<bool>> {
    if !has_projection_field(specs, name) {
        return Ok(None);
    }
    read_bool_optional(batch, name, row)
}

fn parse_geometry_type(value: &str) -> Result<GeometryType> {
    value.parse().map_err(Error::from)
}

fn parse_lod(value: &str) -> Result<LoD> {
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

fn parse_image_type(value: &str) -> Result<ImageType> {
    Ok(match value {
        "PNG" => ImageType::Png,
        "JPG" => ImageType::Jpg,
        other => return Err(Error::Conversion(format!("unsupported image type {other}"))),
    })
}

fn parse_wrap_mode(value: &str) -> Result<WrapMode> {
    Ok(match value {
        "wrap" => WrapMode::Wrap,
        "mirror" => WrapMode::Mirror,
        "clamp" => WrapMode::Clamp,
        "border" => WrapMode::Border,
        "none" => WrapMode::None,
        other => return Err(Error::Conversion(format!("unsupported wrap mode {other}"))),
    })
}

fn parse_texture_mapping_type(value: &str) -> Result<TextureType> {
    Ok(match value {
        "unknown" => TextureType::Unknown,
        "specific" => TextureType::Specific,
        "typical" => TextureType::Typical,
        other => {
            return Err(Error::Conversion(format!(
                "unsupported texture type {other}"
            )));
        }
    })
}

fn parse_rgb_json(value: &str) -> Result<RGB> {
    let json: JsonValue = serde_json::from_str(value)?;
    let array = json
        .as_array()
        .ok_or_else(|| Error::Conversion("rgb payload must be a JSON array".to_string()))?;
    if array.len() != 3 {
        return Err(Error::Conversion(format!(
            "rgb payload must contain 3 elements, found {}",
            array.len()
        )));
    }
    let mut values = [0.0_f32; 3];
    for (index, component) in array.iter().enumerate() {
        values[index] = component
            .as_f64()
            .ok_or_else(|| Error::Conversion("rgb component must be numeric".to_string()))?
            as f32;
    }
    Ok(RGB::from(values))
}

fn parse_rgba_json(value: &str) -> Result<RGBA> {
    let json: JsonValue = serde_json::from_str(value)?;
    let array = json
        .as_array()
        .ok_or_else(|| Error::Conversion("rgba payload must be a JSON array".to_string()))?;
    if array.len() != 4 {
        return Err(Error::Conversion(format!(
            "rgba payload must contain 4 elements, found {}",
            array.len()
        )));
    }
    let mut values = [0.0_f32; 4];
    for (index, component) in array.iter().enumerate() {
        values[index] = component
            .as_f64()
            .ok_or_else(|| Error::Conversion("rgba component must be numeric".to_string()))?
            as f32;
    }
    Ok(RGBA::from(values))
}

fn parse_semantic_type(value: &str) -> SemanticType<cityjson::prelude::OwnedStringStorage> {
    match value {
        "Default" => SemanticType::Default,
        "RoofSurface" => SemanticType::RoofSurface,
        "GroundSurface" => SemanticType::GroundSurface,
        "WallSurface" => SemanticType::WallSurface,
        "ClosureSurface" => SemanticType::ClosureSurface,
        "OuterCeilingSurface" => SemanticType::OuterCeilingSurface,
        "OuterFloorSurface" => SemanticType::OuterFloorSurface,
        "Window" => SemanticType::Window,
        "Door" => SemanticType::Door,
        "InteriorWallSurface" => SemanticType::InteriorWallSurface,
        "CeilingSurface" => SemanticType::CeilingSurface,
        "FloorSurface" => SemanticType::FloorSurface,
        "WaterSurface" => SemanticType::WaterSurface,
        "WaterGroundSurface" => SemanticType::WaterGroundSurface,
        "WaterClosureSurface" => SemanticType::WaterClosureSurface,
        "TrafficArea" => SemanticType::TrafficArea,
        "AuxiliaryTrafficArea" => SemanticType::AuxiliaryTrafficArea,
        "TransportationMarking" => SemanticType::TransportationMarking,
        "TransportationHole" => SemanticType::TransportationHole,
        other if other.starts_with('+') => SemanticType::Extension(other.to_string()),
        other => SemanticType::Extension(other.to_string()),
    }
}

fn contact_from_json(
    value: &JsonValue,
    geometry_handles: &HashMap<u64, cityjson::prelude::GeometryHandle>,
) -> Result<Contact<cityjson::prelude::OwnedStringStorage>> {
    let object = value.as_object().ok_or_else(|| {
        Error::Conversion("pointOfContact must be encoded as a JSON object".to_string())
    })?;
    let mut contact = Contact::new();
    if let Some(value) = object.get("contactName").and_then(JsonValue::as_str) {
        contact.set_contact_name(value.to_string());
    }
    if let Some(value) = object.get("emailAddress").and_then(JsonValue::as_str) {
        contact.set_email_address(value.to_string());
    }
    if let Some(value) = object.get("role").and_then(JsonValue::as_str) {
        contact.set_role(Some(parse_contact_role(value)?));
    }
    if let Some(value) = object.get("website").and_then(JsonValue::as_str) {
        contact.set_website(Some(value.to_string()));
    }
    if let Some(value) = object.get("type").and_then(JsonValue::as_str) {
        contact.set_contact_type(Some(parse_contact_type(value)?));
    }
    if let Some(value) = object.get("phone").and_then(JsonValue::as_str) {
        contact.set_phone(Some(value.to_string()));
    }
    if let Some(value) = object.get("organization").and_then(JsonValue::as_str) {
        contact.set_organization(Some(value.to_string()));
    }
    if let Some(address) = object.get("address") {
        match json_to_attribute(address, geometry_handles)? {
            AttributeValue::Map(map) => {
                contact.set_address(Some(map.into()));
            }
            other => {
                return Err(Error::Conversion(format!(
                    "pointOfContact address must decode to an attribute map, found {other}"
                )));
            }
        }
    }
    Ok(contact)
}

fn parse_contact_role(value: &str) -> Result<ContactRole> {
    Ok(match value {
        "Author" => ContactRole::Author,
        "CoAuthor" => ContactRole::CoAuthor,
        "Processor" => ContactRole::Processor,
        "PointOfContact" => ContactRole::PointOfContact,
        "Owner" => ContactRole::Owner,
        "User" => ContactRole::User,
        "Distributor" => ContactRole::Distributor,
        "Originator" => ContactRole::Originator,
        "Custodian" => ContactRole::Custodian,
        "ResourceProvider" => ContactRole::ResourceProvider,
        "RightsHolder" => ContactRole::RightsHolder,
        "Sponsor" => ContactRole::Sponsor,
        "PrincipalInvestigator" => ContactRole::PrincipalInvestigator,
        "Stakeholder" => ContactRole::Stakeholder,
        "Publisher" => ContactRole::Publisher,
        other => {
            return Err(Error::Conversion(format!(
                "unsupported metadata contact role {other}"
            )));
        }
    })
}

fn parse_contact_type(value: &str) -> Result<ContactType> {
    Ok(match value {
        "Individual" => ContactType::Individual,
        "Organization" => ContactType::Organization,
        other => {
            return Err(Error::Conversion(format!(
                "unsupported metadata contact type {other}"
            )));
        }
    })
}

fn contact_to_json(contact: &Contact<cityjson::prelude::OwnedStringStorage>) -> Result<String> {
    let mut object = JsonMap::new();
    object.insert(
        "contactName".to_string(),
        JsonValue::String(contact.contact_name().to_string()),
    );
    object.insert(
        "emailAddress".to_string(),
        JsonValue::String(contact.email_address().to_string()),
    );
    if let Some(role) = contact.role() {
        object.insert("role".to_string(), JsonValue::String(role.to_string()));
    }
    if let Some(value) = contact.website().as_ref() {
        object.insert("website".to_string(), JsonValue::String(value.clone()));
    }
    if let Some(kind) = contact.contact_type() {
        object.insert("type".to_string(), JsonValue::String(kind.to_string()));
    }
    if let Some(value) = contact.phone().as_ref() {
        object.insert("phone".to_string(), JsonValue::String(value.clone()));
    }
    if let Some(value) = contact.organization().as_ref() {
        object.insert("organization".to_string(), JsonValue::String(value.clone()));
    }
    if let Some(address) = contact.address() {
        object.insert(
            "address".to_string(),
            attribute_to_json(
                &AttributeValue::Map(
                    address
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect::<HashMap<_, _>>(),
                ),
                &HashMap::new(),
            )?,
        );
    }
    Ok(JsonValue::Object(object).to_string())
}

fn read_large_string_scalar(batch: &RecordBatch, name: &str, row: usize) -> Result<String> {
    let array = downcast_required::<LargeStringArray>(batch, name)?;
    Ok(array.value(row).to_string())
}

fn read_large_string_optional(
    batch: &RecordBatch,
    name: &str,
    row: usize,
) -> Result<Option<String>> {
    let array = downcast_required::<LargeStringArray>(batch, name)?;
    Ok((!array.is_null(row)).then(|| array.value(row).to_string()))
}

fn read_string_scalar(batch: &RecordBatch, name: &str, row: usize) -> Result<String> {
    let array = downcast_required::<StringArray>(batch, name)?;
    Ok(array.value(row).to_string())
}

fn read_string_optional(batch: &RecordBatch, name: &str, row: usize) -> Result<Option<String>> {
    let array = downcast_required::<StringArray>(batch, name)?;
    Ok((!array.is_null(row)).then(|| array.value(row).to_string()))
}

fn read_f64_optional(batch: &RecordBatch, name: &str, row: usize) -> Result<Option<f64>> {
    let array = downcast_required::<Float64Array>(batch, name)?;
    Ok((!array.is_null(row)).then(|| array.value(row)))
}

fn read_bool_optional(batch: &RecordBatch, name: &str, row: usize) -> Result<Option<bool>> {
    let array = downcast_required::<arrow::array::BooleanArray>(batch, name)?;
    Ok((!array.is_null(row)).then(|| array.value(row)))
}

fn read_fixed_size_f64_required<const N: usize>(
    batch: &RecordBatch,
    name: &str,
    row: usize,
) -> Result<[f64; N]> {
    read_fixed_size_f64_optional::<N>(batch, name, row)?
        .ok_or_else(|| Error::Conversion(format!("missing required fixed-size list {name}")))
}

fn read_fixed_size_f64_optional<const N: usize>(
    batch: &RecordBatch,
    name: &str,
    row: usize,
) -> Result<Option<[f64; N]>> {
    let array = downcast_required::<FixedSizeListArray>(batch, name)?;
    if array.is_null(row) {
        return Ok(None);
    }
    let values = array.value(row);
    let values = values
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| Error::Conversion(format!("fixed-size list {name} does not contain f64")))?;
    let slice = values.values().as_ref();
    Ok(Some(slice.try_into().map_err(|_| {
        Error::Conversion(format!("fixed-size list {name} does not have length {N}"))
    })?))
}

fn list_u64_value(array: &ListArray, row: usize) -> Result<Vec<u64>> {
    let values = array.value(row);
    let values = values
        .as_any()
        .downcast_ref::<UInt64Array>()
        .ok_or_else(|| Error::Conversion("list child is not u64".to_string()))?;
    Ok(values.values().to_vec())
}

fn list_u32_optional_value(array: &ListArray, row: usize) -> Result<Option<Vec<u32>>> {
    if array.is_null(row) {
        return Ok(None);
    }
    let values = array.value(row);
    let values = values
        .as_any()
        .downcast_ref::<UInt32Array>()
        .ok_or_else(|| Error::Conversion("list child is not u32".to_string()))?;
    Ok(Some(values.values().to_vec()))
}

fn downcast_required<'a, T: Array + 'static>(batch: &'a RecordBatch, name: &str) -> Result<&'a T> {
    batch
        .column_by_name(name)
        .ok_or_else(|| Error::MissingField(name.to_string()))?
        .as_any()
        .downcast_ref::<T>()
        .ok_or_else(|| Error::Conversion(format!("field {name} has unexpected array type")))
}
