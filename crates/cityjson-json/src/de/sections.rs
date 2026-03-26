use std::collections::HashMap;

use serde::Deserialize;

use crate::de::attributes::RawAttribute;

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
pub(crate) struct RawMetadataSection<'a> {
    #[serde(rename = "geographicalExtent", default)]
    pub(crate) geographical_extent: Option<[f64; 6]>,
    #[serde(default, borrow)]
    pub(crate) identifier: Option<&'a str>,
    #[serde(rename = "pointOfContact", default, borrow)]
    pub(crate) point_of_contact: Option<RawContact<'a>>,
    #[serde(rename = "referenceDate", default, borrow)]
    pub(crate) reference_date: Option<&'a str>,
    #[serde(rename = "referenceSystem", default, borrow)]
    pub(crate) reference_system: Option<&'a str>,
    #[serde(default, borrow)]
    pub(crate) title: Option<&'a str>,
    #[serde(flatten, borrow)]
    pub(crate) extra: HashMap<&'a str, RawAttribute<'a>>,
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
pub(crate) struct RawContact<'a> {
    #[serde(rename = "contactName", default, borrow)]
    pub(crate) contact_name: Option<&'a str>,
    #[serde(rename = "emailAddress", default, borrow)]
    pub(crate) email_address: Option<&'a str>,
    #[serde(default, borrow)]
    pub(crate) role: Option<&'a str>,
    #[serde(default, borrow)]
    pub(crate) website: Option<&'a str>,
    #[serde(rename = "contactType", default, borrow)]
    pub(crate) contact_type: Option<&'a str>,
    #[serde(default, borrow)]
    pub(crate) address: Option<HashMap<&'a str, RawAttribute<'a>>>,
    #[serde(default, borrow)]
    pub(crate) phone: Option<&'a str>,
    #[serde(default, borrow)]
    pub(crate) organization: Option<&'a str>,
}

// ---------------------------------------------------------------------------
// Extensions
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
pub(crate) struct RawExtension<'a> {
    #[serde(borrow)]
    pub(crate) url: &'a str,
    #[serde(borrow)]
    pub(crate) version: &'a str,
}

// ---------------------------------------------------------------------------
// Appearance
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
pub(crate) struct RawAppearanceSection<'a> {
    #[serde(default, borrow)]
    pub(crate) materials: Vec<RawMaterial<'a>>,
    #[serde(default, borrow)]
    pub(crate) textures: Vec<RawTexture<'a>>,
    #[serde(rename = "vertices-texture", default)]
    pub(crate) vertices_texture: Vec<[f32; 2]>,
    #[serde(rename = "default-theme-material", default, borrow)]
    pub(crate) default_theme_material: Option<&'a str>,
    #[serde(rename = "default-theme-texture", default, borrow)]
    pub(crate) default_theme_texture: Option<&'a str>,
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
pub(crate) struct RawMaterial<'a> {
    #[serde(borrow)]
    pub(crate) name: &'a str,
    #[serde(rename = "ambientIntensity", default)]
    pub(crate) ambient_intensity: Option<f32>,
    #[serde(rename = "diffuseColor", default)]
    pub(crate) diffuse_color: Option<[f32; 3]>,
    #[serde(rename = "emissiveColor", default)]
    pub(crate) emissive_color: Option<[f32; 3]>,
    #[serde(rename = "specularColor", default)]
    pub(crate) specular_color: Option<[f32; 3]>,
    #[serde(default)]
    pub(crate) shininess: Option<f32>,
    #[serde(default)]
    pub(crate) transparency: Option<f32>,
    #[serde(rename = "isSmooth", default)]
    pub(crate) is_smooth: Option<bool>,
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
pub(crate) struct RawTexture<'a> {
    #[serde(rename = "type", borrow)]
    pub(crate) image_type: &'a str,
    #[serde(borrow)]
    pub(crate) image: &'a str,
    #[serde(rename = "wrapMode", default, borrow)]
    pub(crate) wrap_mode: Option<&'a str>,
    #[serde(rename = "textureType", default, borrow)]
    pub(crate) texture_type: Option<&'a str>,
    #[serde(rename = "borderColor", default)]
    pub(crate) border_color: Option<[f32; 4]>,
}

// ---------------------------------------------------------------------------
// Geometry templates
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
pub(crate) struct RawGeometryTemplatesSection<'a> {
    #[serde(default, borrow)]
    pub(crate) templates: Vec<RawGeometry<'a>>,
    #[serde(rename = "vertices-templates", default)]
    pub(crate) vertices_templates: Vec<[f64; 3]>,
}

// ---------------------------------------------------------------------------
// City objects
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
pub(crate) struct RawCityObject<'a> {
    #[serde(rename = "type", borrow)]
    pub(crate) type_name: &'a str,
    #[serde(rename = "geographicalExtent", default)]
    pub(crate) geographical_extent: Option<[f64; 6]>,
    #[serde(default, borrow)]
    pub(crate) attributes: Option<HashMap<&'a str, RawAttribute<'a>>>,
    #[serde(default, borrow)]
    pub(crate) parents: Vec<&'a str>,
    #[serde(default, borrow)]
    pub(crate) children: Vec<&'a str>,
    #[serde(default, borrow)]
    pub(crate) geometry: Option<Vec<RawGeometry<'a>>>,
    #[serde(flatten, borrow)]
    pub(crate) extra: HashMap<&'a str, RawAttribute<'a>>,
}

// ---------------------------------------------------------------------------
// Geometry
// ---------------------------------------------------------------------------

pub(crate) type MultiPointBoundary = Vec<u32>;
pub(crate) type MultiLineStringBoundary = Vec<MultiPointBoundary>;
pub(crate) type MultiSurfaceBoundary = Vec<MultiLineStringBoundary>;
pub(crate) type SolidBoundary = Vec<MultiSurfaceBoundary>;
pub(crate) type MultiSolidBoundary = Vec<SolidBoundary>;

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
#[serde(tag = "type")]
pub(crate) enum RawGeometry<'a> {
    MultiPoint {
        #[serde(default, borrow)]
        lod: Option<&'a str>,
        boundaries: MultiPointBoundary,
        #[serde(default, borrow)]
        semantics: Option<RawSemantics<'a>>,
        #[serde(default, borrow)]
        material: Option<HashMap<&'a str, RawMaterialTheme>>,
        #[serde(default, borrow)]
        texture: Option<HashMap<&'a str, RawTextureTheme>>,
    },
    MultiLineString {
        #[serde(default, borrow)]
        lod: Option<&'a str>,
        boundaries: MultiLineStringBoundary,
        #[serde(default, borrow)]
        semantics: Option<RawSemantics<'a>>,
        #[serde(default, borrow)]
        material: Option<HashMap<&'a str, RawMaterialTheme>>,
        #[serde(default, borrow)]
        texture: Option<HashMap<&'a str, RawTextureTheme>>,
    },
    MultiSurface {
        #[serde(default, borrow)]
        lod: Option<&'a str>,
        boundaries: MultiSurfaceBoundary,
        #[serde(default, borrow)]
        semantics: Option<RawSemantics<'a>>,
        #[serde(default, borrow)]
        material: Option<HashMap<&'a str, RawMaterialTheme>>,
        #[serde(default, borrow)]
        texture: Option<HashMap<&'a str, RawTextureTheme>>,
    },
    CompositeSurface {
        #[serde(default, borrow)]
        lod: Option<&'a str>,
        boundaries: MultiSurfaceBoundary,
        #[serde(default, borrow)]
        semantics: Option<RawSemantics<'a>>,
        #[serde(default, borrow)]
        material: Option<HashMap<&'a str, RawMaterialTheme>>,
        #[serde(default, borrow)]
        texture: Option<HashMap<&'a str, RawTextureTheme>>,
    },
    Solid {
        #[serde(default, borrow)]
        lod: Option<&'a str>,
        boundaries: SolidBoundary,
        #[serde(default, borrow)]
        semantics: Option<RawSemantics<'a>>,
        #[serde(default, borrow)]
        material: Option<HashMap<&'a str, RawMaterialTheme>>,
        #[serde(default, borrow)]
        texture: Option<HashMap<&'a str, RawTextureTheme>>,
    },
    MultiSolid {
        #[serde(default, borrow)]
        lod: Option<&'a str>,
        boundaries: MultiSolidBoundary,
        #[serde(default, borrow)]
        semantics: Option<RawSemantics<'a>>,
        #[serde(default, borrow)]
        material: Option<HashMap<&'a str, RawMaterialTheme>>,
        #[serde(default, borrow)]
        texture: Option<HashMap<&'a str, RawTextureTheme>>,
    },
    CompositeSolid {
        #[serde(default, borrow)]
        lod: Option<&'a str>,
        boundaries: MultiSolidBoundary,
        #[serde(default, borrow)]
        semantics: Option<RawSemantics<'a>>,
        #[serde(default, borrow)]
        material: Option<HashMap<&'a str, RawMaterialTheme>>,
        #[serde(default, borrow)]
        texture: Option<HashMap<&'a str, RawTextureTheme>>,
    },
    GeometryInstance {
        #[serde(default)]
        template: Option<u32>,
        #[serde(default)]
        boundaries: Option<Vec<u32>>,
        #[serde(rename = "transformationMatrix", default)]
        transformation_matrix: Option<[f64; 16]>,
    },
}

// ---------------------------------------------------------------------------
// Semantics
// ---------------------------------------------------------------------------

/// Typed semantics section for a geometry.
#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
pub(crate) struct RawSemantics<'a> {
    #[serde(borrow)]
    pub(crate) surfaces: Vec<RawSemanticSurface<'a>>,
    pub(crate) values: RawAssignment,
}

/// One semantic surface definition.
#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
pub(crate) struct RawSemanticSurface<'a> {
    #[serde(rename = "type", borrow)]
    pub(crate) type_name: &'a str,
    #[serde(default)]
    pub(crate) parent: Option<u64>,
    #[serde(default)]
    pub(crate) children: Vec<u64>,
    /// Extra per-surface attributes (everything except type/parent/children).
    #[serde(flatten, borrow)]
    pub(crate) attributes: HashMap<&'a str, RawAttribute<'a>>,
}

/// Recursive typed assignment value: null, integer index, or nested array.
///
/// Used for semantics.values, material.values, and material.value.
#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum RawAssignment {
    Null,
    Index(u64),
    Nested(Vec<RawAssignment>),
}

// ---------------------------------------------------------------------------
// Material mapping
// ---------------------------------------------------------------------------

/// One material theme entry: either a single `value` or an array `values`.
#[derive(Deserialize)]
pub(crate) struct RawMaterialTheme {
    pub(crate) value: Option<RawAssignment>,
    pub(crate) values: Option<RawAssignment>,
}

// ---------------------------------------------------------------------------
// Texture mapping
// ---------------------------------------------------------------------------

/// One texture theme entry: always has a `values` array (ring texture assignments).
#[derive(Deserialize)]
pub(crate) struct RawTextureTheme {
    /// Complex nested array of ring texture assignments.
    /// Stored as an owned JSON Value because the structure requires custom parsing
    /// and cannot be zero-copy borrowed inside an internally-tagged enum context.
    pub(crate) values: serde_json::Value,
}
