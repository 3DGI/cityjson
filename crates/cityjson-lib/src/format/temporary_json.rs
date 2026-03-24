//! Temporary JSON-to-model bridge.
//!
//! This module is intentionally not part of `cjlib`'s core facade design. It exists only to keep
//! the crate functional until `serde_cityjson <--> cityjson-rs` conversion is ready, at which
//! point this module should be deleted and replaced by a thin boundary call into `serde_cityjson`.

use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use cityjson::resources::handles::{
    CityObjectHandle, GeometryHandle, GeometryTemplateHandle, MaterialHandle, SemanticHandle,
    TextureHandle,
};
use cityjson::resources::storage::OwnedStringStorage;
use cityjson::v2_0::{
    AffineTransform3D, BBox, CRS, CityModelIdentifier, CityObject, CityObjectIdentifier,
    CityObjectType, Contact, ContactRole, ContactType, Date, Extension, GeometryDraft,
    GeometryType, ImageType, LineStringDraft, LoD, Material, Metadata, OwnedAttributeValue,
    OwnedAttributes, PointDraft, RGB, RGBA, RingDraft, Semantic, SemanticType, ShellDraft,
    SurfaceDraft, Texture, TextureType, Transform, WrapMode,
};
use serde::Deserialize;
use serde_json::{Number, Value as JsonValue};

use crate::{CityModel, Error, Result};

#[derive(Debug, Default, Deserialize)]
struct DocumentJson {
    #[serde(rename = "type")]
    type_model: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    transform: Option<TransformJson>,
    #[serde(default)]
    metadata: Option<MetadataJson>,
    #[serde(default)]
    extensions: BTreeMap<String, ExtensionJson>,
    #[serde(rename = "CityObjects", default)]
    city_objects: BTreeMap<String, CityObjectJson>,
    #[serde(default)]
    vertices: Vec<[f64; 3]>,
    #[serde(default)]
    appearance: Option<AppearanceJson>,
    #[serde(rename = "geometry-templates", default)]
    geometry_templates: Option<GeometryTemplatesJson>,
    #[serde(flatten, default)]
    extra: BTreeMap<String, JsonValue>,
}

#[derive(Debug, Default, Deserialize)]
struct FeatureJson {
    #[serde(rename = "type")]
    type_model: String,
    #[serde(rename = "CityObjects", default)]
    city_objects: BTreeMap<String, CityObjectJson>,
    #[serde(default)]
    vertices: Vec<[f64; 3]>,
    #[serde(default)]
    appearance: Option<AppearanceJson>,
    #[serde(flatten, default)]
    extra: BTreeMap<String, JsonValue>,
}

#[derive(Debug, Deserialize)]
struct TransformJson {
    scale: [f64; 3],
    translate: [f64; 3],
}

#[derive(Debug, Default, Deserialize)]
struct MetadataJson {
    #[serde(rename = "geographicalExtent", default)]
    geographical_extent: Option<[f64; 6]>,
    #[serde(default)]
    identifier: Option<String>,
    #[serde(rename = "referenceDate", default)]
    reference_date: Option<String>,
    #[serde(rename = "referenceSystem", default)]
    reference_system: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(rename = "pointOfContact", default)]
    point_of_contact: Option<ContactJson>,
    #[serde(flatten, default)]
    extra: BTreeMap<String, JsonValue>,
}

#[derive(Debug, Default, Deserialize)]
struct ContactJson {
    #[serde(rename = "contactName", default)]
    contact_name: Option<String>,
    #[serde(rename = "emailAddress", default)]
    email_address: Option<String>,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    website: Option<String>,
    #[serde(rename = "contactType", default)]
    contact_type: Option<String>,
    #[serde(default)]
    address: Option<JsonValue>,
    #[serde(default)]
    phone: Option<String>,
    #[serde(default)]
    organization: Option<String>,
    #[serde(flatten, default)]
    extra: BTreeMap<String, JsonValue>,
}

#[derive(Debug, Deserialize)]
struct ExtensionJson {
    url: String,
    version: String,
}

#[derive(Debug, Default, Deserialize)]
struct AppearanceJson {
    #[serde(default)]
    materials: Vec<MaterialJson>,
    #[serde(default)]
    textures: Vec<TextureJson>,
    #[serde(rename = "vertices-texture", default)]
    vertices_texture: Vec<[f32; 2]>,
    #[serde(rename = "default-theme-material", default)]
    default_theme_material: Option<String>,
    #[serde(rename = "default-theme-texture", default)]
    default_theme_texture: Option<String>,
    #[serde(flatten, default)]
    extra: BTreeMap<String, JsonValue>,
}

#[derive(Debug, Deserialize)]
struct MaterialJson {
    name: String,
    #[serde(rename = "ambientIntensity", default)]
    ambient_intensity: Option<f32>,
    #[serde(rename = "diffuseColor", default)]
    diffuse_color: Option<[f32; 3]>,
    #[serde(rename = "emissiveColor", default)]
    emissive_color: Option<[f32; 3]>,
    #[serde(rename = "specularColor", default)]
    specular_color: Option<[f32; 3]>,
    #[serde(default)]
    shininess: Option<f32>,
    #[serde(default)]
    transparency: Option<f32>,
    #[serde(rename = "isSmooth", default)]
    is_smooth: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct TextureJson {
    image: String,
    #[serde(rename = "type")]
    image_type: String,
    #[serde(rename = "wrapMode", default)]
    wrap_mode: Option<String>,
    #[serde(rename = "textureType", default)]
    texture_type: Option<String>,
    #[serde(rename = "borderColor", default)]
    border_color: Option<[f32; 4]>,
}

#[derive(Debug, Default, Deserialize)]
struct GeometryTemplatesJson {
    #[serde(default)]
    templates: Vec<GeometryJson>,
    #[serde(rename = "vertices-templates", default)]
    vertices_templates: Vec<[f64; 3]>,
}

#[derive(Debug, Default, Clone, Deserialize)]
struct CityObjectJson {
    #[serde(rename = "type")]
    type_name: String,
    #[serde(default)]
    geometry: Vec<GeometryJson>,
    #[serde(default)]
    attributes: Option<BTreeMap<String, JsonValue>>,
    #[serde(default)]
    children: Option<Vec<String>>,
    #[serde(default)]
    parents: Option<Vec<String>>,
    #[serde(rename = "geographicalExtent", default)]
    geographical_extent: Option<[f64; 6]>,
    #[serde(flatten, default)]
    extra: BTreeMap<String, JsonValue>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeometryJson {
    #[serde(rename = "type")]
    type_name: String,
    #[serde(default)]
    lod: Option<LodJson>,
    #[serde(default)]
    boundaries: Option<JsonValue>,
    #[serde(default)]
    semantics: Option<SemanticsJson>,
    #[serde(default)]
    material: BTreeMap<String, MaterialReferenceJson>,
    #[serde(default)]
    texture: BTreeMap<String, TextureReferenceJson>,
    #[serde(default)]
    template: Option<usize>,
    #[serde(rename = "transformationMatrix", default)]
    transformation_matrix: Option<[f64; 16]>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum LodJson {
    String(String),
    Number(Number),
}

#[derive(Debug, Clone, Deserialize)]
struct SemanticsJson {
    #[serde(default)]
    surfaces: Vec<SemanticJson>,
    #[serde(default)]
    values: Option<JsonValue>,
}

#[derive(Debug, Default, Clone, Deserialize)]
struct SemanticJson {
    #[serde(rename = "type", default)]
    type_name: Option<String>,
    #[serde(default)]
    parent: Option<usize>,
    #[serde(default)]
    children: Option<Vec<usize>>,
    #[serde(flatten, default)]
    attributes: BTreeMap<String, JsonValue>,
}

#[derive(Debug, Default, Clone, Deserialize)]
struct MaterialReferenceJson {
    #[serde(default)]
    value: Option<Option<usize>>,
    #[serde(default)]
    values: Option<JsonValue>,
}

#[derive(Debug, Default, Clone, Deserialize)]
struct TextureReferenceJson {
    #[serde(default)]
    values: Option<JsonValue>,
}

#[derive(Debug, Clone, Copy)]
struct CoordinateTransform {
    scale: [f64; 3],
    translate: [f64; 3],
}

impl CoordinateTransform {
    fn from_json(value: &TransformJson) -> Self {
        Self {
            scale: value.scale,
            translate: value.translate,
        }
    }

    fn from_model(value: &Transform) -> Self {
        Self {
            scale: value.scale(),
            translate: value.translate(),
        }
    }

    fn apply(self, coordinate: [f64; 3]) -> cityjson::v2_0::RealWorldCoordinate {
        cityjson::v2_0::RealWorldCoordinate::new(
            coordinate[0] * self.scale[0] + self.translate[0],
            coordinate[1] * self.scale[1] + self.translate[1],
            coordinate[2] * self.scale[2] + self.translate[2],
        )
    }
}

impl Default for CoordinateTransform {
    fn default() -> Self {
        Self {
            scale: [1.0, 1.0, 1.0],
            translate: [0.0, 0.0, 0.0],
        }
    }
}

#[derive(Debug, Default)]
struct AppearanceState {
    materials: Vec<MaterialHandle>,
    textures: Vec<TextureHandle>,
    uv_coordinates: Vec<cityjson::v2_0::VertexIndex<u32>>,
}

#[derive(Debug)]
struct ImportedSemantics {
    handles: Vec<SemanticHandle>,
    values: Option<JsonValue>,
}

#[derive(Debug, Clone)]
struct RingTextureRef {
    texture_index: usize,
    uv_indices: Vec<usize>,
}

pub(crate) fn import_document(bytes: &[u8]) -> Result<CityModel> {
    let document: DocumentJson = serde_json::from_slice(bytes)?;
    if document.type_model != "CityJSON" {
        return Err(Error::ExpectedCityJSON(document.type_model));
    }

    if let Some(id) = document.id {
        return Err(Error::UnsupportedFeature(format!(
            "root id {id:?} is not representable in cityjson-rs yet"
        )));
    }

    let mut model = cityjson::v2_0::OwnedCityModel::new(cityjson::CityModelType::CityJSON);
    let transform = document
        .transform
        .as_ref()
        .map(CoordinateTransform::from_json)
        .unwrap_or_default();

    if let Some(transform_json) = document.transform {
        let transform_value = model.transform_mut();
        transform_value.set_scale(transform_json.scale);
        transform_value.set_translate(transform_json.translate);
    }

    if let Some(metadata) = document.metadata {
        import_metadata(model.metadata_mut(), metadata)?;
    }

    for (name, extension) in document.extensions {
        model
            .extensions_mut()
            .add(Extension::new(name, extension.url, extension.version));
    }

    if !document.extra.is_empty() {
        *model.extra_mut() = convert_attributes_map(document.extra)?;
    }

    let appearance = import_appearance(&mut model, document.appearance)?;
    let vertices = import_vertices(&mut model, &document.vertices, transform)?;

    let template_vertices = import_template_vertices(
        &mut model,
        document
            .geometry_templates
            .as_ref()
            .map(|templates| templates.vertices_templates.as_slice())
            .unwrap_or(&[]),
        transform,
    )?;

    let template_handles = import_templates(
        &mut model,
        document
            .geometry_templates
            .map(|templates| templates.templates)
            .unwrap_or_default(),
        &template_vertices,
        &appearance,
    )?;

    let mut cityobject_handles = HashMap::new();
    import_city_objects(
        &mut model,
        document.city_objects,
        &vertices,
        &appearance,
        &template_handles,
        &mut cityobject_handles,
    )?;

    Ok(CityModel::from(model))
}

pub(crate) fn merge_feature(model: &mut CityModel, bytes: &[u8]) -> Result<()> {
    let feature: FeatureJson = serde_json::from_slice(bytes)?;
    if feature.type_model != "CityJSONFeature" {
        return Err(Error::ExpectedCityJSONFeature(feature.type_model));
    }

    if !feature.extra.is_empty() {
        return Err(Error::Streaming(
            "feature root-level extra members are not supported".into(),
        ));
    }

    let transform = model
        .transform()
        .map(CoordinateTransform::from_model)
        .unwrap_or_default();
    let appearance = import_appearance(&mut model.0, feature.appearance)?;
    let vertices = import_vertices(&mut model.0, &feature.vertices, transform)?;
    let template_handles = model
        .iter_geometry_templates()
        .map(|(handle, _)| handle)
        .collect::<Vec<_>>();
    let mut cityobject_handles = collect_cityobject_handles(&model.0);

    import_city_objects(
        &mut model.0,
        feature.city_objects,
        &vertices,
        &appearance,
        &template_handles,
        &mut cityobject_handles,
    )
}

fn collect_cityobject_handles(
    model: &cityjson::v2_0::OwnedCityModel,
) -> HashMap<String, CityObjectHandle> {
    model
        .cityobjects()
        .iter()
        .map(|(handle, cityobject)| (cityobject.id().to_string(), handle))
        .collect()
}

fn import_vertices(
    model: &mut cityjson::v2_0::OwnedCityModel,
    vertices: &[[f64; 3]],
    transform: CoordinateTransform,
) -> Result<Vec<cityjson::v2_0::VertexIndex<u32>>> {
    vertices
        .iter()
        .map(|vertex| {
            model
                .add_vertex(transform.apply(*vertex))
                .map_err(Into::into)
        })
        .collect()
}

fn import_template_vertices(
    model: &mut cityjson::v2_0::OwnedCityModel,
    vertices: &[[f64; 3]],
    transform: CoordinateTransform,
) -> Result<Vec<cityjson::v2_0::VertexIndex<u32>>> {
    vertices
        .iter()
        .map(|vertex| {
            model
                .add_template_vertex(transform.apply(*vertex))
                .map_err(Into::into)
        })
        .collect()
}

fn import_appearance(
    model: &mut cityjson::v2_0::OwnedCityModel,
    appearance: Option<AppearanceJson>,
) -> Result<AppearanceState> {
    let Some(appearance) = appearance else {
        return Ok(AppearanceState::default());
    };

    if appearance.default_theme_material.is_some() || appearance.default_theme_texture.is_some() {
        return Err(Error::UnsupportedFeature(
            "default appearance themes are not representable in cityjson-rs yet".into(),
        ));
    }

    if !appearance.extra.is_empty() {
        return Err(Error::UnsupportedFeature(
            "appearance extra members are not supported".into(),
        ));
    }

    let materials = appearance
        .materials
        .into_iter()
        .map(|material| {
            let mut value = Material::new(material.name);
            value.set_ambient_intensity(material.ambient_intensity);
            value.set_diffuse_color(material.diffuse_color.map(RGB::from));
            value.set_emissive_color(material.emissive_color.map(RGB::from));
            value.set_specular_color(material.specular_color.map(RGB::from));
            value.set_shininess(material.shininess);
            value.set_transparency(material.transparency);
            value.set_is_smooth(material.is_smooth);
            model.add_material(value).map_err(Into::into)
        })
        .collect::<Result<Vec<_>>>()?;

    let textures = appearance
        .textures
        .into_iter()
        .map(|texture| {
            let mut value = Texture::new(texture.image, parse_image_type(&texture.image_type)?);
            value.set_wrap_mode(
                texture
                    .wrap_mode
                    .as_deref()
                    .map(parse_wrap_mode)
                    .transpose()?,
            );
            value.set_texture_type(
                texture
                    .texture_type
                    .as_deref()
                    .map(parse_texture_type)
                    .transpose()?,
            );
            value.set_border_color(texture.border_color.map(RGBA::from));
            model.add_texture(value).map_err(Into::into)
        })
        .collect::<Result<Vec<_>>>()?;

    let uv_coordinates = appearance
        .vertices_texture
        .into_iter()
        .map(|coordinate| {
            model
                .add_uv_coordinate(cityjson::v2_0::UVCoordinate::new(
                    coordinate[0],
                    coordinate[1],
                ))
                .map_err(Into::into)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(AppearanceState {
        materials,
        textures,
        uv_coordinates,
    })
}

fn import_templates(
    _model: &mut cityjson::v2_0::OwnedCityModel,
    templates: Vec<GeometryJson>,
    _template_vertices: &[cityjson::v2_0::VertexIndex<u32>],
    _appearance: &AppearanceState,
) -> Result<Vec<GeometryTemplateHandle>> {
    if templates.is_empty() {
        Ok(Vec::new())
    } else {
        Err(Error::UnsupportedFeature(
            "geometry template import is not implemented yet".into(),
        ))
    }
}

fn import_city_objects(
    model: &mut cityjson::v2_0::OwnedCityModel,
    city_objects: BTreeMap<String, CityObjectJson>,
    vertices: &[cityjson::v2_0::VertexIndex<u32>],
    appearance: &AppearanceState,
    template_handles: &[GeometryTemplateHandle],
    cityobject_handles: &mut HashMap<String, CityObjectHandle>,
) -> Result<()> {
    for (id, city_object) in &city_objects {
        if cityobject_handles.contains_key(id) {
            return Err(Error::Streaming(format!(
                "duplicate city object id in stream: {id}"
            )));
        }

        let handle = model.cityobjects_mut().add(CityObject::new(
            CityObjectIdentifier::new(id.clone()),
            CityObjectType::from_str(&city_object.type_name)
                .map_err(|error| Error::Import(error.to_string()))?,
        ))?;
        cityobject_handles.insert(id.clone(), handle);
    }

    for (id, city_object) in &city_objects {
        let geometry_handles = city_object
            .geometry
            .iter()
            .cloned()
            .map(|geometry| {
                import_geometry(model, geometry, vertices, appearance, template_handles)
            })
            .collect::<Result<Vec<_>>>()?;

        let handle = *cityobject_handles
            .get(id)
            .ok_or_else(|| Error::Import(format!("missing handle for city object {id}")))?;
        let cityobject = model
            .cityobjects_mut()
            .get_mut(handle)
            .ok_or_else(|| Error::Import(format!("invalid handle for city object {id}")))?;

        if let Some(attributes) = city_object.attributes.clone() {
            *cityobject.attributes_mut() = convert_attributes_map(attributes)?;
        }
        if let Some(extra) = (!city_object.extra.is_empty())
            .then(|| convert_attributes_map(city_object.extra.clone()))
            .transpose()?
        {
            *cityobject.extra_mut() = extra;
        }
        if let Some(geographical_extent) = city_object.geographical_extent {
            cityobject.set_geographical_extent(Some(BBox::from(geographical_extent)));
        }
        for geometry_handle in geometry_handles {
            cityobject.add_geometry(geometry_handle);
        }
    }

    for (id, city_object) in &city_objects {
        let handle = *cityobject_handles
            .get(id)
            .ok_or_else(|| Error::Import(format!("missing handle for city object {id}")))?;
        let cityobject_ref = model
            .cityobjects_mut()
            .get_mut(handle)
            .ok_or_else(|| Error::Import(format!("invalid handle for city object {id}")))?;

        if let Some(parents) = &city_object.parents {
            for parent_id in parents {
                let parent = *cityobject_handles.get(parent_id).ok_or_else(|| {
                    Error::Import(format!(
                        "city object {id} references unknown parent {parent_id}"
                    ))
                })?;
                cityobject_ref.add_parent(parent);
            }
        }

        if let Some(children) = &city_object.children {
            for child_id in children {
                let child = *cityobject_handles.get(child_id).ok_or_else(|| {
                    Error::Import(format!(
                        "city object {id} references unknown child {child_id}"
                    ))
                })?;
                cityobject_ref.add_child(child);
            }
        }
    }

    Ok(())
}

fn import_geometry(
    model: &mut cityjson::v2_0::OwnedCityModel,
    geometry: GeometryJson,
    vertices: &[cityjson::v2_0::VertexIndex<u32>],
    appearance: &AppearanceState,
    template_handles: &[GeometryTemplateHandle],
) -> Result<GeometryHandle> {
    let geometry_type = GeometryType::from_str(&geometry.type_name)
        .map_err(|error| Error::Import(error.to_string()))?;
    let lod = geometry.lod.as_ref().map(parse_lod).transpose()?;
    let semantics = import_semantics(model, geometry.semantics.clone())?;

    match geometry_type {
        GeometryType::MultiPoint => {
            ensure_no_surface_style(&geometry)?;
            let boundaries = parse_ring(geometry.boundaries.as_ref().ok_or_else(|| {
                Error::Import("MultiPoint geometry is missing boundaries".into())
            })?)?;
            let assignments = match semantics
                .as_ref()
                .and_then(|semantics| semantics.values.as_ref())
            {
                Some(values) => parse_optional_index_vec(values)?,
                None => vec![None; boundaries.len()],
            };
            if assignments.len() != boundaries.len() {
                return Err(Error::Import(
                    "MultiPoint semantics count does not match point count".into(),
                ));
            }
            let points = boundaries
                .into_iter()
                .zip(assignments)
                .map(|(index, semantic)| {
                    let mut point = PointDraft::new(vertex_at(vertices, index)?);
                    if let Some(semantic) = semantic {
                        point = point.with_semantic(semantic_handle(semantics.as_ref(), semantic)?);
                    }
                    Ok(point)
                })
                .collect::<Result<Vec<_>>>()?;

            GeometryDraft::<u32, OwnedStringStorage>::multi_point(lod, points)
                .insert_into(model)
                .map_err(Into::into)
        }
        GeometryType::MultiLineString => {
            ensure_no_surface_style(&geometry)?;
            let boundaries = parse_linestrings(geometry.boundaries.as_ref().ok_or_else(|| {
                Error::Import("MultiLineString geometry is missing boundaries".into())
            })?)?;
            let assignments = match semantics
                .as_ref()
                .and_then(|semantics| semantics.values.as_ref())
            {
                Some(values) => parse_optional_index_vec(values)?,
                None => vec![None; boundaries.len()],
            };
            if assignments.len() != boundaries.len() {
                return Err(Error::Import(
                    "MultiLineString semantics count does not match linestring count".into(),
                ));
            }
            let lines = boundaries
                .into_iter()
                .zip(assignments)
                .map(|(linestring, semantic)| {
                    let mut draft = LineStringDraft::new(
                        linestring
                            .into_iter()
                            .map(|index| vertex_at(vertices, index))
                            .collect::<Result<Vec<_>>>()?,
                    );
                    if let Some(semantic) = semantic {
                        draft = draft.with_semantic(semantic_handle(semantics.as_ref(), semantic)?);
                    }
                    Ok(draft)
                })
                .collect::<Result<Vec<_>>>()?;

            GeometryDraft::<u32, OwnedStringStorage>::multi_line_string(lod, lines)
                .insert_into(model)
                .map_err(Into::into)
        }
        GeometryType::MultiSurface | GeometryType::CompositeSurface => {
            let boundaries =
                parse_surfaces(geometry.boundaries.as_ref().ok_or_else(|| {
                    Error::Import("surface geometry is missing boundaries".into())
                })?)?;
            let semantic_assignments = match semantics
                .as_ref()
                .and_then(|semantics| semantics.values.as_ref())
            {
                Some(values) => parse_optional_index_vec(values)?,
                None => vec![None; boundaries.len()],
            };
            if semantic_assignments.len() != boundaries.len() {
                return Err(Error::Import(
                    "surface semantics count does not match surface count".into(),
                ));
            }
            let material_assignments =
                parse_surface_materials(&geometry.material, appearance, boundaries.len())?;
            let texture_assignments =
                parse_surface_textures(&geometry.texture, appearance, &boundaries)?;

            let surfaces = boundaries
                .into_iter()
                .enumerate()
                .map(|(surface_index, rings)| {
                    let surface_textures =
                        texture_assignments.get(surface_index).ok_or_else(|| {
                            Error::Import(
                                "surface texture topology does not match boundaries".into(),
                            )
                        })?;
                    let mut surface = SurfaceDraft::new(
                        build_ring(&rings[0], &surface_textures[0], vertices, appearance)?,
                        rings
                            .iter()
                            .enumerate()
                            .skip(1)
                            .map(|(ring_index, ring)| {
                                build_ring(
                                    ring,
                                    surface_textures.get(ring_index).ok_or_else(|| {
                                        Error::Import(
                                            "surface texture topology does not match boundaries"
                                                .into(),
                                        )
                                    })?,
                                    vertices,
                                    appearance,
                                )
                            })
                            .collect::<Result<Vec<_>>>()?,
                    );
                    if let Some(semantic) = semantic_assignments[surface_index] {
                        surface =
                            surface.with_semantic(semantic_handle(semantics.as_ref(), semantic)?);
                    }
                    for (theme, material) in &material_assignments[surface_index] {
                        surface = surface.with_material(theme.clone(), *material);
                    }
                    Ok(surface)
                })
                .collect::<Result<Vec<_>>>()?;

            match geometry_type {
                GeometryType::MultiSurface => {
                    GeometryDraft::<u32, OwnedStringStorage>::multi_surface(lod, surfaces)
                }
                GeometryType::CompositeSurface => {
                    GeometryDraft::<u32, OwnedStringStorage>::composite_surface(lod, surfaces)
                }
                _ => unreachable!(),
            }
            .insert_into(model)
            .map_err(Into::into)
        }
        GeometryType::Solid => {
            let boundaries =
                parse_shells(geometry.boundaries.as_ref().ok_or_else(|| {
                    Error::Import("Solid geometry is missing boundaries".into())
                })?)?;
            let semantic_assignments = match semantics
                .as_ref()
                .and_then(|semantics| semantics.values.as_ref())
            {
                Some(values) => parse_optional_index_shells(values)?,
                None => boundaries
                    .iter()
                    .map(|shell| vec![None; shell.len()])
                    .collect::<Vec<_>>(),
            };
            if semantic_assignments.len() != boundaries.len() {
                return Err(Error::Import(
                    "Solid semantics count does not match shell count".into(),
                ));
            }
            let material_assignments =
                parse_solid_materials(&geometry.material, appearance, &boundaries)?;
            let texture_assignments =
                parse_solid_textures(&geometry.texture, appearance, &boundaries)?;

            let outer = build_shell(
                &boundaries[0],
                semantic_assignments.first().ok_or_else(|| {
                    Error::Import("Solid geometry is missing the exterior shell".into())
                })?,
                material_assignments.first().ok_or_else(|| {
                    Error::Import("Solid material topology does not match boundaries".into())
                })?,
                texture_assignments.first().ok_or_else(|| {
                    Error::Import("Solid texture topology does not match boundaries".into())
                })?,
                vertices,
                appearance,
                semantics.as_ref(),
            )?;

            let inner_shells = boundaries
                .iter()
                .enumerate()
                .skip(1)
                .map(|(shell_index, shell)| {
                    build_shell(
                        shell,
                        semantic_assignments.get(shell_index).ok_or_else(|| {
                            Error::Import(
                                "Solid semantics topology does not match boundaries".into(),
                            )
                        })?,
                        material_assignments.get(shell_index).ok_or_else(|| {
                            Error::Import(
                                "Solid material topology does not match boundaries".into(),
                            )
                        })?,
                        texture_assignments.get(shell_index).ok_or_else(|| {
                            Error::Import("Solid texture topology does not match boundaries".into())
                        })?,
                        vertices,
                        appearance,
                        semantics.as_ref(),
                    )
                })
                .collect::<Result<Vec<_>>>()?;

            GeometryDraft::<u32, OwnedStringStorage>::solid(lod, outer, inner_shells)
                .insert_into(model)
                .map_err(Into::into)
        }
        GeometryType::GeometryInstance => {
            let _ = (
                model,
                vertices,
                template_handles,
                AffineTransform3D::identity(),
            );
            let _ = (
                geometry.boundaries,
                geometry.template,
                geometry.transformation_matrix,
                lod,
                semantics,
            );
            Err(Error::UnsupportedFeature(
                "GeometryInstance import is not implemented yet".into(),
            ))
        }
        GeometryType::MultiSolid | GeometryType::CompositeSolid => Err(Error::UnsupportedFeature(
            format!("{geometry_type} import is not implemented yet"),
        )),
        _ => Err(Error::UnsupportedFeature(format!(
            "unsupported geometry type {geometry_type}"
        ))),
    }
}

fn build_shell(
    shell: &[Vec<Vec<usize>>],
    semantic_assignments: &[Option<usize>],
    material_assignments: &[Vec<(String, MaterialHandle)>],
    texture_assignments: &[Vec<Vec<(String, RingTextureRef)>>],
    vertices: &[cityjson::v2_0::VertexIndex<u32>],
    appearance: &AppearanceState,
    semantics: Option<&ImportedSemantics>,
) -> Result<ShellDraft<u32, OwnedStringStorage>> {
    if semantic_assignments.len() != shell.len()
        || material_assignments.len() != shell.len()
        || texture_assignments.len() != shell.len()
    {
        return Err(Error::Import(
            "shell topology does not match semantics/materials/textures".into(),
        ));
    }

    let surfaces = shell
        .iter()
        .enumerate()
        .map(|(surface_index, rings)| {
            let mut surface = SurfaceDraft::new(
                build_ring(
                    &rings[0],
                    texture_assignments
                        .get(surface_index)
                        .and_then(|surface| surface.first())
                        .ok_or_else(|| {
                            Error::Import("shell texture topology does not match boundaries".into())
                        })?,
                    vertices,
                    appearance,
                )?,
                rings
                    .iter()
                    .enumerate()
                    .skip(1)
                    .map(|(ring_index, ring)| {
                        build_ring(
                            ring,
                            texture_assignments
                                .get(surface_index)
                                .and_then(|surface| surface.get(ring_index))
                                .ok_or_else(|| {
                                    Error::Import(
                                        "shell texture topology does not match boundaries".into(),
                                    )
                                })?,
                            vertices,
                            appearance,
                        )
                    })
                    .collect::<Result<Vec<_>>>()?,
            );

            if let Some(semantic) = semantic_assignments[surface_index] {
                surface = surface.with_semantic(semantic_handle(semantics, semantic)?);
            }
            for (theme, material) in &material_assignments[surface_index] {
                surface = surface.with_material(theme.clone(), *material);
            }

            Ok(surface)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(ShellDraft::new(surfaces))
}

fn build_ring(
    ring: &[usize],
    texture_assignments: &[(String, RingTextureRef)],
    vertices: &[cityjson::v2_0::VertexIndex<u32>],
    appearance: &AppearanceState,
) -> Result<RingDraft<u32, OwnedStringStorage>> {
    let mut draft = RingDraft::new(
        ring.iter()
            .copied()
            .map(|index| vertex_at(vertices, index))
            .collect::<Result<Vec<_>>>()?,
    );

    for (theme, texture) in texture_assignments {
        let texture_handle = appearance
            .textures
            .get(texture.texture_index)
            .copied()
            .ok_or_else(|| {
                Error::Import(format!(
                    "texture index {} is out of bounds",
                    texture.texture_index
                ))
            })?;
        let uvs = texture
            .uv_indices
            .iter()
            .copied()
            .map(|index| {
                appearance
                    .uv_coordinates
                    .get(index)
                    .copied()
                    .ok_or_else(|| {
                        Error::Import(format!("texture vertex index {index} is out of bounds"))
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        draft = draft.with_texture(theme.clone(), texture_handle, uvs);
    }

    Ok(draft)
}

fn import_semantics(
    model: &mut cityjson::v2_0::OwnedCityModel,
    semantics: Option<SemanticsJson>,
) -> Result<Option<ImportedSemantics>> {
    let Some(semantics) = semantics else {
        return Ok(None);
    };

    let handles = semantics
        .surfaces
        .iter()
        .map(|semantic| {
            let mut value = Semantic::new(parse_semantic_type(
                semantic.type_name.as_deref().unwrap_or("Default"),
            )?);
            if !semantic.attributes.is_empty() {
                *value.attributes_mut() = convert_attributes_map(semantic.attributes.clone())?;
            }
            model.add_semantic(value).map_err(Into::into)
        })
        .collect::<Result<Vec<_>>>()?;

    for (semantic, handle) in semantics.surfaces.iter().zip(handles.iter().copied()) {
        let value = model
            .get_semantic_mut(handle)
            .ok_or_else(|| Error::Import("invalid semantic handle".into()))?;
        if let Some(parent) = semantic.parent {
            let parent_handle = *handles.get(parent).ok_or_else(|| {
                Error::Import(format!("semantic parent index {parent} is out of bounds"))
            })?;
            value.set_parent(parent_handle);
        }
        if let Some(children) = &semantic.children {
            for child in children {
                let child_handle = *handles.get(*child).ok_or_else(|| {
                    Error::Import(format!("semantic child index {child} is out of bounds"))
                })?;
                value.children_mut().push(child_handle);
            }
        }
    }

    Ok(Some(ImportedSemantics {
        handles,
        values: semantics.values,
    }))
}

fn ensure_no_surface_style(geometry: &GeometryJson) -> Result<()> {
    if !geometry.material.is_empty() || !geometry.texture.is_empty() {
        return Err(Error::UnsupportedFeature(
            "materials and textures are only supported on surface-based geometry".into(),
        ));
    }
    Ok(())
}

fn parse_surface_materials(
    material: &BTreeMap<String, MaterialReferenceJson>,
    appearance: &AppearanceState,
    surface_count: usize,
) -> Result<Vec<Vec<(String, MaterialHandle)>>> {
    let mut assignments = vec![Vec::new(); surface_count];
    for (theme, reference) in material {
        let per_surface = if let Some(value) = reference.value {
            vec![value; surface_count]
        } else if let Some(values) = reference.values.as_ref() {
            parse_optional_index_vec(values)?
        } else {
            return Err(Error::Import(format!(
                "material theme {theme} must contain value or values"
            )));
        };

        if per_surface.len() != surface_count {
            return Err(Error::Import(format!(
                "material theme {theme} does not match the number of surfaces"
            )));
        }

        for (surface_index, material_index) in per_surface.into_iter().enumerate() {
            if let Some(material_index) = material_index {
                assignments[surface_index].push((
                    theme.clone(),
                    *appearance.materials.get(material_index).ok_or_else(|| {
                        Error::Import(format!("material index {material_index} is out of bounds"))
                    })?,
                ));
            }
        }
    }
    Ok(assignments)
}

fn parse_solid_materials(
    material: &BTreeMap<String, MaterialReferenceJson>,
    appearance: &AppearanceState,
    shells: &[Vec<Vec<Vec<usize>>>],
) -> Result<Vec<Vec<Vec<(String, MaterialHandle)>>>> {
    let mut assignments = shells
        .iter()
        .map(|shell| vec![Vec::new(); shell.len()])
        .collect::<Vec<_>>();

    for (theme, reference) in material {
        let per_surface = if let Some(value) = reference.value {
            shells
                .iter()
                .map(|shell| vec![value; shell.len()])
                .collect::<Vec<_>>()
        } else if let Some(values) = reference.values.as_ref() {
            parse_optional_index_shells(values)?
        } else {
            return Err(Error::Import(format!(
                "material theme {theme} must contain value or values"
            )));
        };

        if per_surface.len() != shells.len() {
            return Err(Error::Import(format!(
                "material theme {theme} does not match the number of shells"
            )));
        }

        for (shell_index, surfaces) in per_surface.into_iter().enumerate() {
            if surfaces.len() != shells[shell_index].len() {
                return Err(Error::Import(format!(
                    "material theme {theme} does not match the number of surfaces"
                )));
            }
            for (surface_index, material_index) in surfaces.into_iter().enumerate() {
                if let Some(material_index) = material_index {
                    assignments[shell_index][surface_index].push((
                        theme.clone(),
                        *appearance.materials.get(material_index).ok_or_else(|| {
                            Error::Import(format!(
                                "material index {material_index} is out of bounds"
                            ))
                        })?,
                    ));
                }
            }
        }
    }

    Ok(assignments)
}

fn parse_surface_textures(
    texture: &BTreeMap<String, TextureReferenceJson>,
    appearance: &AppearanceState,
    boundaries: &[Vec<Vec<usize>>],
) -> Result<Vec<Vec<Vec<(String, RingTextureRef)>>>> {
    let mut assignments = boundaries
        .iter()
        .map(|surface| vec![Vec::new(); surface.len()])
        .collect::<Vec<_>>();

    for (theme, reference) in texture {
        let values = reference
            .values
            .as_ref()
            .ok_or_else(|| Error::Import(format!("texture theme {theme} must contain values")))?;
        let per_surface = parse_surface_ring_textures(values)?;
        if per_surface.len() != boundaries.len() {
            return Err(Error::Import(format!(
                "texture theme {theme} does not match the number of surfaces"
            )));
        }

        for (surface_index, rings) in per_surface.into_iter().enumerate() {
            if rings.len() != boundaries[surface_index].len() {
                return Err(Error::Import(format!(
                    "texture theme {theme} does not match the number of rings"
                )));
            }

            for (ring_index, ring) in rings.into_iter().enumerate() {
                if let Some(texture_ref) = ring {
                    if texture_ref.uv_indices.len() != boundaries[surface_index][ring_index].len() {
                        return Err(Error::Import(format!(
                            "texture theme {theme} ring has {} uv indices but boundary has {} vertices",
                            texture_ref.uv_indices.len(),
                            boundaries[surface_index][ring_index].len()
                        )));
                    }
                    if texture_ref.texture_index >= appearance.textures.len() {
                        return Err(Error::Import(format!(
                            "texture index {} is out of bounds",
                            texture_ref.texture_index
                        )));
                    }
                    assignments[surface_index][ring_index].push((theme.clone(), texture_ref));
                }
            }
        }
    }

    Ok(assignments)
}

fn parse_solid_textures(
    texture: &BTreeMap<String, TextureReferenceJson>,
    appearance: &AppearanceState,
    boundaries: &[Vec<Vec<Vec<usize>>>],
) -> Result<Vec<Vec<Vec<Vec<(String, RingTextureRef)>>>>> {
    let mut assignments = boundaries
        .iter()
        .map(|shell| {
            shell
                .iter()
                .map(|surface| vec![Vec::new(); surface.len()])
                .collect()
        })
        .collect::<Vec<Vec<Vec<Vec<(String, RingTextureRef)>>>>>();

    for (theme, reference) in texture {
        let values = reference
            .values
            .as_ref()
            .ok_or_else(|| Error::Import(format!("texture theme {theme} must contain values")))?;
        let per_shell = parse_shell_ring_textures(values)?;
        if per_shell.len() != boundaries.len() {
            return Err(Error::Import(format!(
                "texture theme {theme} does not match the number of shells"
            )));
        }

        for (shell_index, surfaces) in per_shell.into_iter().enumerate() {
            if surfaces.len() != boundaries[shell_index].len() {
                return Err(Error::Import(format!(
                    "texture theme {theme} does not match the number of surfaces"
                )));
            }
            for (surface_index, rings) in surfaces.into_iter().enumerate() {
                if rings.len() != boundaries[shell_index][surface_index].len() {
                    return Err(Error::Import(format!(
                        "texture theme {theme} does not match the number of rings"
                    )));
                }
                for (ring_index, ring) in rings.into_iter().enumerate() {
                    if let Some(texture_ref) = ring {
                        if texture_ref.uv_indices.len()
                            != boundaries[shell_index][surface_index][ring_index].len()
                        {
                            return Err(Error::Import(format!(
                                "texture theme {theme} ring has {} uv indices but boundary has {} vertices",
                                texture_ref.uv_indices.len(),
                                boundaries[shell_index][surface_index][ring_index].len()
                            )));
                        }
                        if texture_ref.texture_index >= appearance.textures.len() {
                            return Err(Error::Import(format!(
                                "texture index {} is out of bounds",
                                texture_ref.texture_index
                            )));
                        }
                        assignments[shell_index][surface_index][ring_index]
                            .push((theme.clone(), texture_ref));
                    }
                }
            }
        }
    }

    Ok(assignments)
}

fn convert_attributes_map(values: BTreeMap<String, JsonValue>) -> Result<OwnedAttributes> {
    let mut attributes = OwnedAttributes::new();
    for (key, value) in values {
        attributes.insert(key, convert_attribute_value(value)?);
    }
    Ok(attributes)
}

fn convert_attribute_value(value: JsonValue) -> Result<OwnedAttributeValue> {
    Ok(match value {
        JsonValue::Null => OwnedAttributeValue::Null,
        JsonValue::Bool(value) => OwnedAttributeValue::Bool(value),
        JsonValue::Number(value) => {
            if let Some(integer) = value.as_i64() {
                OwnedAttributeValue::Integer(integer)
            } else if let Some(unsigned) = value.as_u64() {
                OwnedAttributeValue::Unsigned(unsigned)
            } else {
                OwnedAttributeValue::Float(
                    value
                        .as_f64()
                        .ok_or_else(|| Error::Import("unsupported JSON number".into()))?,
                )
            }
        }
        JsonValue::String(value) => OwnedAttributeValue::String(value),
        JsonValue::Array(values) => OwnedAttributeValue::Vec(
            values
                .into_iter()
                .map(convert_attribute_value)
                .map(|value| value.map(Box::new))
                .collect::<Result<Vec<_>>>()?,
        ),
        JsonValue::Object(values) => OwnedAttributeValue::Map(
            values
                .into_iter()
                .map(|(key, value)| Ok((key, Box::new(convert_attribute_value(value)?))))
                .collect::<Result<HashMap<_, _>>>()?,
        ),
    })
}

fn import_metadata(metadata: &mut Metadata<OwnedStringStorage>, value: MetadataJson) -> Result<()> {
    if let Some(bbox) = value.geographical_extent {
        metadata.set_geographical_extent(BBox::from(bbox));
    }
    if let Some(identifier) = value.identifier {
        metadata.set_identifier(CityModelIdentifier::new(identifier));
    }
    if let Some(reference_date) = value.reference_date {
        metadata.set_reference_date(Date::new(reference_date));
    }
    if let Some(reference_system) = value.reference_system {
        metadata.set_reference_system(CRS::new(reference_system));
    }
    if let Some(title) = value.title {
        metadata.set_title(title);
    }
    if let Some(contact) = value.point_of_contact {
        metadata.set_point_of_contact(Some(import_contact(contact)?));
    }
    if !value.extra.is_empty() {
        metadata.set_extra(Some(convert_attributes_map(value.extra)?));
    }
    Ok(())
}

fn import_contact(value: ContactJson) -> Result<Contact<OwnedStringStorage>> {
    if !value.extra.is_empty() {
        return Err(Error::UnsupportedFeature(
            "pointOfContact extra members are not supported".into(),
        ));
    }

    let mut contact = Contact::new();
    if let Some(name) = value.contact_name {
        contact.set_contact_name(name);
    }
    if let Some(email) = value.email_address {
        contact.set_email_address(email);
    }
    if let Some(role) = value.role.as_deref() {
        contact.set_role(Some(parse_contact_role(role)?));
    }
    if let Some(website) = value.website {
        contact.set_website(Some(website));
    }
    if let Some(contact_type) = value.contact_type.as_deref() {
        contact.set_contact_type(Some(parse_contact_type(contact_type)?));
    }
    if let Some(phone) = value.phone {
        contact.set_phone(Some(phone));
    }
    if let Some(organization) = value.organization {
        contact.set_organization(Some(organization));
    }
    if let Some(address) = value.address {
        let JsonValue::Object(address) = address else {
            return Err(Error::UnsupportedFeature(
                "pointOfContact.address must be a JSON object".into(),
            ));
        };
        contact.set_address(Some(convert_attributes_map(
            address.into_iter().collect::<BTreeMap<_, _>>(),
        )?));
    }
    Ok(contact)
}

fn parse_image_type(value: &str) -> Result<ImageType> {
    match value {
        "PNG" | "png" => Ok(ImageType::Png),
        "JPG" | "jpg" | "JPEG" | "jpeg" => Ok(ImageType::Jpg),
        other => Err(Error::UnsupportedFeature(format!(
            "unsupported texture image type {other}"
        ))),
    }
}

fn parse_wrap_mode(value: &str) -> Result<WrapMode> {
    match value {
        "wrap" => Ok(WrapMode::Wrap),
        "mirror" => Ok(WrapMode::Mirror),
        "clamp" => Ok(WrapMode::Clamp),
        "border" => Ok(WrapMode::Border),
        other => Err(Error::UnsupportedFeature(format!(
            "unsupported texture wrap mode {other}"
        ))),
    }
}

fn parse_texture_type(value: &str) -> Result<TextureType> {
    match value {
        "unknown" => Ok(TextureType::Unknown),
        "specific" => Ok(TextureType::Specific),
        "typical" => Ok(TextureType::Typical),
        other => Err(Error::UnsupportedFeature(format!(
            "unsupported texture type {other}"
        ))),
    }
}

fn parse_contact_role(value: &str) -> Result<ContactRole> {
    match value {
        "author" => Ok(ContactRole::Author),
        "processor" => Ok(ContactRole::Processor),
        "pointOfContact" => Ok(ContactRole::PointOfContact),
        "owner" => Ok(ContactRole::Owner),
        "user" => Ok(ContactRole::User),
        "distributor" => Ok(ContactRole::Distributor),
        "originator" => Ok(ContactRole::Originator),
        "custodian" => Ok(ContactRole::Custodian),
        "resourceProvider" => Ok(ContactRole::ResourceProvider),
        "rightsHolder" => Ok(ContactRole::RightsHolder),
        "sponsor" => Ok(ContactRole::Sponsor),
        "principalInvestigator" => Ok(ContactRole::PrincipalInvestigator),
        "stakeholder" => Ok(ContactRole::Stakeholder),
        "publisher" => Ok(ContactRole::Publisher),
        other => Err(Error::UnsupportedFeature(format!(
            "unsupported contact role {other}"
        ))),
    }
}

fn parse_contact_type(value: &str) -> Result<ContactType> {
    match value {
        "individual" => Ok(ContactType::Individual),
        "organization" => Ok(ContactType::Organization),
        other => Err(Error::UnsupportedFeature(format!(
            "unsupported contact type {other}"
        ))),
    }
}

fn parse_semantic_type(value: &str) -> Result<SemanticType<OwnedStringStorage>> {
    Ok(match value {
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
        other => {
            return Err(Error::UnsupportedFeature(format!(
                "unsupported semantic type {other}"
            )));
        }
    })
}

fn parse_lod(value: &LodJson) -> Result<LoD> {
    let value = match value {
        LodJson::String(value) => value.clone(),
        LodJson::Number(value) => value.to_string(),
    };
    match value.as_str() {
        "0" => Ok(LoD::LoD0),
        "0.0" => Ok(LoD::LoD0_0),
        "0.1" => Ok(LoD::LoD0_1),
        "0.2" => Ok(LoD::LoD0_2),
        "0.3" => Ok(LoD::LoD0_3),
        "1" => Ok(LoD::LoD1),
        "1.0" => Ok(LoD::LoD1_0),
        "1.1" => Ok(LoD::LoD1_1),
        "1.2" => Ok(LoD::LoD1_2),
        "1.3" => Ok(LoD::LoD1_3),
        "2" => Ok(LoD::LoD2),
        "2.0" => Ok(LoD::LoD2_0),
        "2.1" => Ok(LoD::LoD2_1),
        "2.2" => Ok(LoD::LoD2_2),
        "2.3" => Ok(LoD::LoD2_3),
        "3" => Ok(LoD::LoD3),
        "3.0" => Ok(LoD::LoD3_0),
        "3.1" => Ok(LoD::LoD3_1),
        "3.2" => Ok(LoD::LoD3_2),
        "3.3" => Ok(LoD::LoD3_3),
        other => Err(Error::UnsupportedFeature(format!(
            "unsupported LoD value {other}"
        ))),
    }
}

fn semantic_handle(semantics: Option<&ImportedSemantics>, index: usize) -> Result<SemanticHandle> {
    semantics
        .and_then(|semantics| semantics.handles.get(index).copied())
        .ok_or_else(|| Error::Import(format!("semantic index {index} is out of bounds")))
}

fn vertex_at(
    vertices: &[cityjson::v2_0::VertexIndex<u32>],
    index: usize,
) -> Result<cityjson::v2_0::VertexIndex<u32>> {
    vertices
        .get(index)
        .copied()
        .ok_or_else(|| Error::Import(format!("vertex index {index} is out of bounds")))
}

fn parse_linestrings(value: &JsonValue) -> Result<Vec<Vec<usize>>> {
    parse_array(value, parse_ring)
}

fn parse_surfaces(value: &JsonValue) -> Result<Vec<Vec<Vec<usize>>>> {
    parse_array(value, parse_linestrings)
}

fn parse_shells(value: &JsonValue) -> Result<Vec<Vec<Vec<Vec<usize>>>>> {
    parse_array(value, parse_surfaces)
}

fn parse_ring(value: &JsonValue) -> Result<Vec<usize>> {
    parse_array(value, parse_index)
}

fn parse_optional_index_vec(value: &JsonValue) -> Result<Vec<Option<usize>>> {
    parse_array(value, parse_optional_index)
}

fn parse_optional_index_shells(value: &JsonValue) -> Result<Vec<Vec<Option<usize>>>> {
    parse_array(value, parse_optional_index_vec)
}

fn parse_surface_ring_textures(value: &JsonValue) -> Result<Vec<Vec<Option<RingTextureRef>>>> {
    parse_array(value, |surface| {
        parse_array(surface, parse_ring_texture_ref)
    })
}

fn parse_shell_ring_textures(value: &JsonValue) -> Result<Vec<Vec<Vec<Option<RingTextureRef>>>>> {
    parse_array(value, parse_surface_ring_textures)
}

fn parse_ring_texture_ref(value: &JsonValue) -> Result<Option<RingTextureRef>> {
    match value {
        JsonValue::Null => Ok(None),
        JsonValue::Array(values) => {
            if values.is_empty() {
                return Err(Error::Import(
                    "texture ring reference must contain a texture index".into(),
                ));
            }
            let texture_index = parse_index(&values[0])?;
            let uv_indices = values[1..]
                .iter()
                .map(parse_index)
                .collect::<Result<Vec<_>>>()?;
            Ok(Some(RingTextureRef {
                texture_index,
                uv_indices,
            }))
        }
        _ => Err(Error::Import(
            "texture ring reference must be null or an array".into(),
        )),
    }
}

fn parse_index(value: &JsonValue) -> Result<usize> {
    value
        .as_u64()
        .map(|value| value as usize)
        .ok_or_else(|| Error::Import(format!("expected unsigned integer index, got {value}")))
}

fn parse_optional_index(value: &JsonValue) -> Result<Option<usize>> {
    match value {
        JsonValue::Null => Ok(None),
        _ => parse_index(value).map(Some),
    }
}

fn parse_array<T>(
    value: &JsonValue,
    parse_item: impl Fn(&JsonValue) -> Result<T>,
) -> Result<Vec<T>> {
    let JsonValue::Array(values) = value else {
        return Err(Error::Import(format!("expected JSON array, got {value}")));
    };
    values.iter().map(parse_item).collect()
}
