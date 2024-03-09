//! CityJSON version 1.1
//!
//! Specs: <https://www.cityjson.org/specs/1.1.3/>.
//!
//! The main struct is [CityModel], which represents a CityJSON or CityJSONFeature object.
//! See the examples of usage by the various members.

use crate::errors::{Error, Result};

use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[cfg(feature = "datasize")]
use datasize::{data_size, DataSize};
use derive_more::Display;
use serde::de::Visitor;
use serde::{Deserialize, Serialize};
#[cfg(feature = "datasize")]
use std::io::Write;

/// Represents the city model that is stored in a CityJSON object.
/// The conceptual equivalent of a CityJSON object, but the `CityModel` is also used for
/// `CityJSONFeature`s.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#cityjson-object>.
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let cm: CityModel = serde_json::from_str(r#"{
///   "type": "CityJSON",
///   "version": "1.1",
///   "extensions": {},
///   "transform": {
///     "scale": [1.0, 1.0, 1.0],
///     "translate": [0.0, 0.0, 0.0]
///   },
///   "metadata": {},
///   "CityObjects": {},
///   "vertices": [],
///   "appearance": {},
///   "geometry-templates": {}
/// }"#)?;
/// println!("{:?}", &cm);
/// let cm_json = serde_json::to_string(&cm)?;
///
/// let cfeature: CityModel = serde_json::from_str(r#"{
///   "type": "CityJSONFeature",
///   "id": "myid",
///   "CityObjects": {},
///   "vertices": [],
///   "appearance": {}
/// }"#)?;
/// println!("{:?}", &cfeature);
/// let cfeature_json = serde_json::to_string(&cfeature)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct CityModel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_cm: CityModelType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<CityJSONVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<Transform>,
    #[serde(rename = "CityObjects")]
    pub cityobjects: CityObjects,
    pub vertices: Vertices,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appearance: Option<Appearance>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "kebab-case")]
    pub geometry_templates: Option<GeometryTemplates>,
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_attributes"
    )]
    #[cfg_attr(feature = "datasize", data_size(with = sizeof_attributes_option))]
    pub extra: Option<Attributes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// Version of the CityJSON specifications used for this city model. This module is only for
/// version `1.1`, thus there is only one version available.
#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Hash, Deserialize, Serialize)]
#[serde(tag = "version", try_from = "String", into = "String")]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub enum CityJSONVersion {
    #[default]
    V1_1,
}

/// CityModel type.
///
/// Marks if the [CityModel] represents a CityJSON object or a CityJSONFeature object.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub enum CityModelType {
    #[default]
    CityJSON,
    CityJSONFeature,
}

/// Transform.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#transform-object>.
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let transform: Transform = serde_json::from_str(r#"{
///   "scale": [0.001, 0.001, 0.001],
///   "translate": [442464.879, 5482614.692, 310.19]
/// }"#)?;
/// println!("{}", &transform);
/// let transform_json = serde_json::to_string(&transform)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct Transform {
    pub scale: [f64; 3],
    pub translate: [f64; 3],
}

/// The `CityObjects` member of CityJSON.
pub type CityObjects = HashMap<String, CityObject>;

/// CityObject.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#the-different-city-objects>.
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let co: CityObject = serde_json::from_str(r#"{
///   "type": "BuildingPart",
///   "geographicalExtent": [ 84710.1, 446846.0, -5.3, 84757.1, 446944.0, 40.9 ],
///   "attributes": {
///     "measuredHeight": 22.3,
///     "roofType": "gable",
///     "owner": "Elvis Presley"
///   },
///   "children": ["id-2"],
///   "parents": ["id-3"],
///   "geometry": []
/// }"#)?;
/// println!("{}", &co);
/// let co_json = serde_json::to_string(&co)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Display, Clone, Deserialize, Serialize)]
#[display(
    fmt = "type: {}, geometry: {:?}, attributes: {:?}, geographical_extent: {:?}, children: {:?}, parents: {:?}",
    type_co,
    geometry,
    attributes,
    geographical_extent,
    children,
    parents
)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct CityObject {
    #[serde(rename = "type")]
    pub type_co: CityObjectType,
    pub geometry: Vec<Geometry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "datasize", data_size(with = sizeof_attributes_option))]
    pub attributes: Option<Attributes>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "camelCase")]
    pub geographical_extent: Option<BBox>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parents: Option<Vec<String>>,
}

/// CityObject type.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#the-different-city-objects>
///
/// CityObject types from an Extension are stored in the `Extension(String)` variant, which stores
/// the type as a string, e.g. `Extension("+NoiseBuilding")`.
/// It contains a special variant `Default`, which is only used as the default variant and it is not
/// a valid CityObject type.
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let cotype: CityObjectType = serde_json::from_str(r#""+NoiseBuilding""#)?;
/// println!("{}", &cotype);
/// let cotype_json = serde_json::to_string(&cotype)?;
///
/// let cotype: CityObjectType = serde_json::from_str(r#""BridgeRoom""#)?;
/// println!("{}", &cotype);
/// let cotype_json = serde_json::to_string(&cotype)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default, Display, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub enum CityObjectType {
    Bridge,
    BridgePart,
    BridgeInstallation,
    BridgeConstructiveElement,
    BridgeRoom,
    BridgeFurniture,
    Building,
    BuildingPart,
    BuildingInstallation,
    BuildingConstructiveElement,
    BuildingFurniture,
    BuildingStorey,
    BuildingRoom,
    BuildingUnit,
    CityFurniture,
    CityObjectGroup,
    #[default]
    Default,
    LandUse,
    OtherConstruction,
    PlantCover,
    SolitaryVegetationObject,
    TINRelief,
    WaterBody,
    Road,
    Railway,
    Waterway,
    TransportSquare,
    Tunnel,
    TunnelPart,
    TunnelInstallation,
    TunnelConstructiveElement,
    TunnelHollowSpace,
    TunnelFurniture,
    Extension(String),
}

/// Attributes of CityModel, CityObjects, Semantics.
pub type Attributes = HashMap<String, serde_json::Value>;

/// Geometry.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#geometry-objects>.
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let geom: Geometry = serde_json::from_str(r#"{
///   "type": "Solid",
///   "lod": "2.1",
///   "boundaries": [
///     [ [[0, 3, 2, 1]], [[4, 5, 6, 7]], [[0, 1, 5, 4]], [[1, 2, 6, 5]] ]
///   ],
///   "semantics": {
///     "surfaces": [
///       { "type": "RoofSurface" },
///       { "type": "+PatioDoor"}
///     ],
///     "values": [[0, 0, null, 1]]
///   },
///   "material": {
///     "irradiation": {
///       "values": [[0, 0, 1, null]]
///     },
///     "red": {
///       "value": 3
///     }
///   }
/// }"#)?;
/// println!("{:?}", &geom);
/// let geom_json = serde_json::to_string(&geom)?;
///
/// let geom: Geometry = serde_json::from_str(r#"{
///   "type": "GeometryInstance",
///   "template": 0,
///   "boundaries": [372],
///   "transformationMatrix": [
///     2.0, 0.0, 0.0, 0.0,
///     0.0, 2.0, 0.0, 0.0,
///     0.0, 0.0, 2.0, 0.0,
///     0.0, 0.0, 0.0, 1.0
///   ]
/// }"#)?;
/// println!("{:?}", &geom);
/// let geom_json = serde_json::to_string(&geom)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub enum Geometry {
    MultiPoint {
        lod: LoD,
        boundaries: MultiPointBoundary,
        semantics: Option<MultiPointSemantics>,
    },
    MultiLineString {
        lod: LoD,
        boundaries: SurfaceBoundary,
        semantics: Option<MultiLineStringSemantics>,
    },
    MultiSurface {
        lod: LoD,
        boundaries: AggregateSurfaceBoundary,
        semantics: Option<MultiSurfaceSemantics>,
        material: Option<HashMap<String, MultiSurfaceAppearanceValues>>,
        texture: Option<HashMap<String, MultiSurfaceAppearanceValues>>,
    },
    CompositeSurface {
        lod: LoD,
        boundaries: AggregateSurfaceBoundary,
        semantics: Option<CompositeSurfaceSemantics>,
        material: Option<HashMap<String, CompositeSurfaceAppearanceValues>>,
        texture: Option<HashMap<String, CompositeSurfaceAppearanceValues>>,
    },
    Solid {
        lod: LoD,
        boundaries: SolidBoundary,
        semantics: Option<SolidSemantics>,
        material: Option<HashMap<String, SolidAppearanceValues>>,
        texture: Option<HashMap<String, SolidAppearanceValues>>,
    },
    MultiSolid {
        lod: LoD,
        boundaries: AggregateSolidBoundary,
        semantics: Option<MultiSolidSemantics>,
        material: Option<HashMap<String, MultiSolidAppearanceValues>>,
        texture: Option<HashMap<String, MultiSolidAppearanceValues>>,
    },
    CompositeSolid {
        lod: LoD,
        boundaries: AggregateSolidBoundary,
        semantics: Option<CompositeSolidSemantics>,
        material: Option<HashMap<String, CompositeSolidAppearanceValues>>,
        texture: Option<HashMap<String, CompositeSolidAppearanceValues>>,
    },
    #[serde(rename_all = "camelCase")]
    GeometryInstance {
        template: usize,
        boundaries: [usize; 1],
        transformation_matrix: [f64; 16],
    },
}

/// The Level of Detail of a Geometry.
///
/// The `LoD` forms an order, such as `LoD0 < LoD0_0 < LoD0_1 < LoD0_2 < LoD0_3 < LoD1 < ...`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub enum LoD {
    LoD0,
    LoD0_0,
    LoD0_1,
    LoD0_2,
    LoD0_3,
    LoD1,
    LoD1_0,
    LoD1_1,
    LoD1_2,
    LoD1_3,
    LoD2,
    LoD2_0,
    LoD2_1,
    LoD2_2,
    LoD2_3,
    LoD3,
    LoD3_0,
    LoD3_1,
    LoD3_2,
    LoD3_3,
}

/// Appearance.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#appearance-object>.
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let appearance: Appearance = serde_json::from_str(r#"{
///   "materials": [],
///   "textures":[],
///   "vertices-texture": [],
///   "default-theme-texture": "myDefaultTheme1",
///   "default-theme-material": "myDefaultTheme2"
/// }"#)?;
/// println!("{}", &appearance);
/// let appearance_json = serde_json::to_string(&appearance)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Default, Debug, Display, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[display(
    fmt = "materials: {:?}, textures: {:?}, vertices-texture: {:?}, default-theme-texture: {:?}, default-theme-material: {:?}",
    materials,
    textures,
    vertices_texture,
    default_theme_texture,
    default_theme_material
)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct Appearance {
    #[serde(skip_serializing_if = "Option::is_none")]
    materials: Option<Vec<Material>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    textures: Option<Vec<Texture>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vertices_texture: Option<VerticesTexture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_theme_texture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_theme_material: Option<String>,
}

/// Material.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#material-object>.
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let material: Material = serde_json::from_str(r#"{
///     "name": "roofandground",
///     "ambientIntensity":  0.2000,
///     "diffuseColor":  [0.9000, 0.1000, 0.7500],
///     "emissiveColor": [0.9000, 0.1000, 0.7500],
///     "specularColor": [0.9000, 0.1000, 0.7500],
///     "shininess": 0.2,
///     "transparency": 0.5,
///     "isSmooth": false
/// }"#)?;
/// println!("{}", &material);
/// let material_json = serde_json::to_string(&material)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Default, Debug, Display, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[display(
    fmt = "name: {}, ambient_intensity: {:?}, diffuse_color: {:?}, emissive_color: {:?}, specular_color: {:?}, shininess: {:?}, transparency: {:?}, is_smooth: {:?}",
    name,
    ambient_intensity,
    diffuse_color,
    emissive_color,
    specular_color,
    shininess,
    transparency,
    is_smooth
)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct Material {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ambient_intensity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diffuse_color: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emissive_color: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    specular_color: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shininess: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transparency: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_smooth: Option<bool>,
}

/// Texture.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#texture-object>.
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let texture: Texture = serde_json::from_str(r#"{
///     "type": "JPG",
///     "image": "appearances/myroof.jpg",
///     "wrapMode": "wrap",
///     "textureType": "unknown",
///     "borderColor": [0.0, 0.1, 0.2, 1.0]
/// }"#)?;
/// println!("{}", &texture);
/// let texture_json = serde_json::to_string(&texture)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Default, Debug, Display, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[display(
    fmt = "type: {:?}, image: {:?}, wrap_mode: {:?}, texture_type: {:?}, border_color: {:?}",
    image_type,
    image,
    wrap_mode,
    texture_type,
    border_color
)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct Texture {
    #[serde(rename = "type")]
    image_type: ImageType,
    image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    wrap_mode: Option<WrapMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    texture_type: Option<TextureType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_color: Option<[f32; 4]>,
}

/// Texture image type.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#texture-object>.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub enum ImageType {
    Png,
    Jpg,
}

/// Texture wrap mode.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#texture-object>.
#[derive(Clone, Copy, Debug, Default, Display, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub enum WrapMode {
    Wrap,
    Mirror,
    Clamp,
    Border,
    #[default]
    None,
}

/// Texture type.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#texture-object>.
#[derive(Clone, Copy, Debug, Default, Display, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub enum TextureType {
    #[default]
    Unknown,
    Specific,
    Typical,
}

/// Vertices-texture of an Appearance.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#vertices-texture-object>.
///
///
pub type VerticesTexture = Vec<[f32; 2]>;

/// The Material or Texture index of a MultiSurface geometry. This is the `value` or `values` member
/// of a Material or Texture that is assigned to the Geometry object.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#geometry-object-having-material-s> and
/// <https://www.cityjson.org/specs/1.1.3/#geometry-object-having-texture-s>.
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let ms_app_val: MultiSurfaceAppearanceValues = serde_json::from_str(r#"{
///     "values": [0, 0, 1, null]
/// }"#)?;
/// println!("{}", &ms_app_val);
/// let ms_app_val_json = serde_json::to_string(&ms_app_val)?;
/// let ms_app_val: MultiSurfaceAppearanceValues = serde_json::from_str(r#"{
///     "value": 0
/// }"#)?;
/// println!("{}", &ms_app_val);
/// let ms_app_val_json = serde_json::to_string(&ms_app_val)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default, Display, PartialEq, Deserialize, Serialize)]
#[display(fmt = "value: {:?}, values: {:?}", value, values)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct MultiSurfaceAppearanceValues {
    #[serde(skip_serializing_if = "Option::is_none")]
    value: OptionalIndex,
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<Vec<OptionalIndex>>,
}

/// The Material or Texture index of a CompositeSurface geometry. This is the `value` or `values`
/// member of a Material or Texture that is assigned to the Geometry object.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#geometry-object-having-material-s> and
/// <https://www.cityjson.org/specs/1.1.3/#geometry-object-having-texture-s>.
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let cs_app_val: CompositeSurfaceAppearanceValues = serde_json::from_str(r#"{
///     "values": [0, 0, 1, null]
/// }"#)?;
/// println!("{}", &cs_app_val);
/// let cs_app_val_json = serde_json::to_string(&cs_app_val)?;
/// let cs_app_val: CompositeSurfaceAppearanceValues = serde_json::from_str(r#"{
///     "value": 0
/// }"#)?;
/// println!("{}", &cs_app_val);
/// let ms_app_val_json = serde_json::to_string(&cs_app_val)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default, Display, PartialEq, Deserialize, Serialize)]
#[display(fmt = "value: {:?}, values: {:?}", value, values)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct CompositeSurfaceAppearanceValues {
    #[serde(skip_serializing_if = "Option::is_none")]
    value: OptionalIndex,
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<Vec<OptionalIndex>>,
}

/// The Material or Texture index of a Solid geometry. This is the `value` or `values`
/// member of a Material or Texture that is assigned to the Geometry object.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#geometry-object-having-material-s> and
/// <https://www.cityjson.org/specs/1.1.3/#geometry-object-having-texture-s>.
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let s_app_val: SolidAppearanceValues = serde_json::from_str(r#"{
///     "values": [[0, 0, 1, null]]
/// }"#)?;
/// println!("{}", &s_app_val);
/// let s_app_val_json = serde_json::to_string(&s_app_val)?;
/// let s_app_val: SolidAppearanceValues = serde_json::from_str(r#"{
///     "value": 0
/// }"#)?;
/// println!("{}", &s_app_val);
/// let s_app_val_json = serde_json::to_string(&s_app_val)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default, Display, PartialEq, Deserialize, Serialize)]
#[display(fmt = "value: {:?}, values: {:?}", value, values)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct SolidAppearanceValues {
    #[serde(skip_serializing_if = "Option::is_none")]
    value: OptionalIndex,
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<Vec<Vec<OptionalIndex>>>,
}

/// The Material or Texture index of a MultiSolid geometry. This is the `value` or `values`
/// member of a Material or Texture that is assigned to the Geometry object.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#geometry-object-having-material-s> and
/// <https://www.cityjson.org/specs/1.1.3/#geometry-object-having-texture-s>.
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let msol_app_val: MultiSolidAppearanceValues = serde_json::from_str(r#"{
///     "values": [[[0, 0, 1, null]]]
/// }"#)?;
/// println!("{}", &msol_app_val);
/// let msol_app_val_json = serde_json::to_string(&msol_app_val)?;
/// let msol_app_val: MultiSolidAppearanceValues = serde_json::from_str(r#"{
///     "value": 0
/// }"#)?;
/// println!("{}", &msol_app_val);
/// let s_app_val_json = serde_json::to_string(&msol_app_val)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default, Display, PartialEq, Deserialize, Serialize)]
#[display(fmt = "value: {:?}, values: {:?}", value, values)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct MultiSolidAppearanceValues {
    #[serde(skip_serializing_if = "Option::is_none")]
    value: OptionalIndex,
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<Vec<Vec<Vec<OptionalIndex>>>>,
}

/// The Material or Texture index of a CompositeSolid geometry. This is the `value` or `values`
/// member of a Material or Texture that is assigned to the Geometry object.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#geometry-object-having-material-s> and
/// <https://www.cityjson.org/specs/1.1.3/#geometry-object-having-texture-s>.
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let csol_app_val: CompositeSolidAppearanceValues = serde_json::from_str(r#"{
///     "values": [[[0, 0, 1, null]]]
/// }"#)?;
/// println!("{}", &csol_app_val);
/// let csol_app_val_json = serde_json::to_string(&csol_app_val)?;
/// let csol_app_val: CompositeSolidAppearanceValues = serde_json::from_str(r#"{
///     "value": 0
/// }"#)?;
/// println!("{}", &csol_app_val);
/// let s_app_val_json = serde_json::to_string(&csol_app_val)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default, Display, PartialEq, Deserialize, Serialize)]
#[display(fmt = "value: {:?}, values: {:?}", value, values)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct CompositeSolidAppearanceValues {
    #[serde(skip_serializing_if = "Option::is_none")]
    value: OptionalIndex,
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<Vec<Vec<Vec<OptionalIndex>>>>,
}

/// Geometry Templates.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#geometry-templates>.
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let geometry_templates: GeometryTemplates = serde_json::from_str(r#"{
///   "templates": [
///     {
///       "type": "MultiSurface",
///       "lod": "2.1",
///       "boundaries": [
///          [[0, 3, 2, 1]], [[4, 5, 6, 7]], [[0, 1, 5, 4]]
///       ]
///     },
///     {
///       "type": "MultiSurface",
///       "lod": "1.3",
///       "boundaries": [
///          [[1, 2, 6, 5]], [[2, 3, 7, 6]], [[3, 0, 4, 7]]
///       ]
///     }
///   ],
///   "vertices-templates": [
///     [0.0, 0.5, 0.0],
///     [1.0, 1.0, 0.0],
///     [0.0, 1.0, 0.0]
///   ]
/// }"#)?;
/// println!("{}", &geometry_templates);
/// let geometry_templates_json = serde_json::to_string(&geometry_templates)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Default, Debug, Display, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[display(
    fmt = "templates: {:?}, vertices-templates: {:?}",
    templates,
    vertices_templates
)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct GeometryTemplates {
    templates: Vec<Geometry>,
    vertices_templates: VerticesTemplates,
}

/// The `vertices_templates` member of `geometry-templates` of CityJSON.
pub type VerticesTemplates = Vec<[f64; 3]>;

/// The `semantics` of a `CompositeSolid` geometry.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#semantics-of-geometric-primitives>.
///
/// # Examples
/// ```rust
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let sem: CompositeSolidSemantics = serde_json::from_str(r#"{
///     "surfaces" : [
///       {
///         "type": "WallSurface",
///         "slope": 33.4,
///         "children": [2]
///       },
///       {
///         "type": "RoofSurface",
///         "slope": 66.6
///       },
///       {
///         "type": "+PatioDoor",
///         "parent": 0,
///         "colour": "blue"
///       }
///     ],
///     "values": [[[0, 1, 1, null]], [[null, null, null]]]
/// }"#)?;
/// println!("{}", &sem);
/// let sem_json = serde_json::to_string(&sem)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Display, PartialEq, Eq, Deserialize, Serialize)]
#[display(fmt = "surfaces: {:?}, values: {:?}", surfaces, values)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct CompositeSolidSemantics {
    pub surfaces: Vec<Semantic>,
    pub values: CompositeSolidSemanticsValues,
}

/// The `semantics` of a `MultiSolid` geometry.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#semantics-of-geometric-primitives>.
///
/// # Examples
/// ```rust
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let sem: MultiSolidSemantics = serde_json::from_str(r#"{
///     "surfaces" : [
///       {
///         "type": "WallSurface",
///         "slope": 33.4,
///         "children": [2]
///       },
///       {
///         "type": "RoofSurface",
///         "slope": 66.6
///       },
///       {
///         "type": "+PatioDoor",
///         "parent": 0,
///         "colour": "blue"
///       }
///     ],
///     "values": [[[0, 1, 1, null]], [[null, null, null]]]
/// }"#)?;
/// println!("{}", &sem);
/// let sem_json = serde_json::to_string(&sem)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Display, PartialEq, Eq, Deserialize, Serialize)]
#[display(fmt = "surfaces: {:?}, values: {:?}", surfaces, values)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct MultiSolidSemantics {
    pub surfaces: Vec<Semantic>,
    pub values: MultiSolidSemanticsValues,
}

/// The `semantics` of a `Solid` geometry.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#semantics-of-geometric-primitives>.
///
/// # Examples
/// ```rust
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let sem: SolidSemantics = serde_json::from_str(r#"{
///     "surfaces" : [
///       {
///         "type": "WallSurface",
///         "slope": 33.4,
///         "children": [2]
///       },
///       {
///         "type": "RoofSurface",
///         "slope": 66.6
///       },
///       {
///         "type": "+PatioDoor",
///         "parent": 0,
///         "colour": "blue"
///       }
///     ],
///     "values": [[0, 0, null, 1, 2]]
/// }"#)?;
/// println!("{}", &sem);
/// let sem_json = serde_json::to_string(&sem)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Display, PartialEq, Eq, Deserialize, Serialize)]
#[display(fmt = "surfaces: {:?}, values: {:?}", surfaces, values)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct SolidSemantics {
    pub surfaces: Vec<Semantic>,
    pub values: SolidSemanticsValues,
}

/// The `semantics` of a `CompositeSurface` geometry.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#semantics-of-geometric-primitives>.
///
/// # Examples
/// ```rust
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let sem: CompositeSurfaceSemantics = serde_json::from_str(r#"{
///     "surfaces" : [
///       {
///         "type": "WallSurface",
///         "slope": 33.4,
///         "children": [2]
///       },
///       {
///         "type": "RoofSurface",
///         "slope": 66.6
///       },
///       {
///         "type": "+PatioDoor",
///         "parent": 0,
///         "colour": "blue"
///       }
///     ],
///     "values": [0, 0, null, 1, 2]
/// }"#)?;
/// println!("{}", &sem);
/// let sem_json = serde_json::to_string(&sem)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Display, PartialEq, Eq, Deserialize, Serialize)]
#[display(fmt = "surfaces: {:?}, values: {:?}", surfaces, values)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct CompositeSurfaceSemantics {
    pub surfaces: Vec<Semantic>,
    pub values: CompositeSurfaceSemanticsValues,
}

/// The `semantics` of a `MultiSurface` geometry.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#semantics-of-geometric-primitives>.
///
/// # Examples
/// ```rust
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let sem: MultiSurfaceSemantics = serde_json::from_str(r#"{
///     "surfaces" : [
///       {
///         "type": "WallSurface",
///         "slope": 33.4,
///         "children": [2]
///       },
///       {
///         "type": "RoofSurface",
///         "slope": 66.6
///       },
///       {
///         "type": "+PatioDoor",
///         "parent": 0,
///         "colour": "blue"
///       }
///     ],
///     "values": [0, 0, null, 1, 2]
/// }"#)?;
/// println!("{}", &sem);
/// let sem_json = serde_json::to_string(&sem)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Display, PartialEq, Eq, Deserialize, Serialize)]
#[display(fmt = "surfaces: {:?}, values: {:?}", surfaces, values)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct MultiSurfaceSemantics {
    pub surfaces: Vec<Semantic>,
    pub values: MultiSurfaceSemanticsValues,
}

/// The `semantics` of a `MultiLineString` geometry.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#semantics-of-geometric-primitives>.
///
/// # Examples
/// ```rust
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let sem: MultiLineStringSemantics = serde_json::from_str(r#"{
///     "surfaces" : [
///       {
///         "type": "TransportationMarking"
///       }
///     ],
///     "values": [0, 0, null, 1, 2]
/// }"#)?;
/// println!("{}", &sem);
/// let sem_json = serde_json::to_string(&sem)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Display, PartialEq, Eq, Deserialize, Serialize)]
#[display(fmt = "surfaces: {:?}, values: {:?}", surfaces, values)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct MultiLineStringSemantics {
    pub surfaces: Vec<Semantic>,
    pub values: MultiLineStringSemanticsValues,
}

/// The `semantics` of a `MultiPoint` geometry.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#semantics-of-geometric-primitives>.
///
/// # Examples
/// ```rust
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let sem: MultiPointSemantics = serde_json::from_str(r#"{
///     "surfaces" : [
///       {
///         "type": "TransportationMarking"
///       }
///     ],
///     "values": [0, 0, null, 1, 2]
/// }"#)?;
/// println!("{}", &sem);
/// let sem_json = serde_json::to_string(&sem)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Display, PartialEq, Eq, Deserialize, Serialize)]
#[display(fmt = "surfaces: {:?}, values: {:?}", surfaces, values)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct MultiPointSemantics {
    pub surfaces: Vec<Semantic>,
    pub values: MultiPointSemanticsValues,
}

/// Semantic Object.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#semantics-of-geometric-primitives>.
///
/// # Examples
/// ```rust
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let sem: Semantic = serde_json::from_str(r#"{ "type": "RoofSurface" }"#)?;
/// let sem_json = serde_json::to_string(&sem)?;
/// let sem: Semantic = serde_json::from_str(r#"{
///     "type": "+MySemantic",
///     "my_attribute": 42,
///     "children": [2, 37],
///     "parent": 0
/// }"#)?;
/// let sem_json = serde_json::to_string(&sem)?;
/// println!("{}", &sem);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Display, PartialEq, Eq, Deserialize, Serialize)]
#[display(
    fmt = "type: {:?}, children: {:?}, parent: {:?}, attributes: {:?}",
    type_sem,
    children,
    parent,
    attributes
)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct Semantic {
    #[serde(rename = "type")]
    pub type_sem: SemanticType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<usize>,
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_attributes"
    )]
    #[cfg_attr(feature = "datasize", data_size(with = sizeof_attributes_option))]
    pub attributes: Option<Attributes>,
}

/// Semantic surface type.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#semantics-of-geometric-primitives>.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub enum SemanticType {
    RoofSurface,
    GroundSurface,
    WallSurface,
    ClosureSurface,
    OuterCeilingSurface,
    OuterFloorSurface,
    Window,
    Door,
    InteriorWallSurface,
    CeilingSurface,
    FloorSurface,
    WaterSurface,
    WaterGroundSurface,
    WaterClosureSurface,
    TrafficArea,
    AuxiliaryTrafficArea,
    TransportationMarking,
    TransportationHole,
    Extension(String),
}

/// The `values` array of geometry indices of a Semantic object.
///
/// # Examples
/// ```
/// use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let csolsemval: CompositeSolidSemanticsValues = serde_json::from_str(r#"[
///  [ [0, 1, 1, null] ],
///  [ [null, null, null] ]
/// ]"#)?;
/// # Ok(())
/// # }
/// ```
pub type CompositeSolidSemanticsValues = Vec<Vec<Vec<OptionalIndex>>>;

/// The `values` array of geometry indices of a Semantic object.
///
/// # Examples
/// ```
/// use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let msolsemval: MultiSolidSemanticsValues = serde_json::from_str(r#"[
///  [ [0, 1, 1, null] ],
///  [ [null, null, null] ]
/// ]"#)?;
/// # Ok(())
/// # }
/// ```
pub type MultiSolidSemanticsValues = Vec<Vec<Vec<OptionalIndex>>>;

/// The `values` array of geometry indices of a Semantic object.
///
/// # Examples
/// ```
/// use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let solsemval: SolidSemanticsValues = serde_json::from_str(r#"[ [0, 1, 1, null] ]"#)?;
/// # Ok(())
/// # }
/// ```
pub type SolidSemanticsValues = Vec<Vec<OptionalIndex>>;

/// The `values` array of geometry indices of a Semantic object.
pub type CompositeSurfaceSemanticsValues = Vec<OptionalIndex>;

/// The `values` array of geometry indices of a Semantic object.
///
/// # Examples
/// ```
/// use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let mptsemval: MultiSurfaceSemanticsValues = serde_json::from_str(r#"[0, 0, null, 1, 2]"#)?;
/// # Ok(())
/// # }
/// ```
pub type MultiSurfaceSemanticsValues = Vec<OptionalIndex>;

/// The `values` array of geometry indices of a Semantic object.
pub type MultiLineStringSemanticsValues = Vec<OptionalIndex>;

/// The `values` array of geometry indices of a Semantic object.
///
/// # Examples
/// ```
/// use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let mptsemval: MultiPointSemanticsValues = serde_json::from_str(r#"[0, 0, null, 1, 2]"#)?;
/// # Ok(())
/// # }
/// ```
pub type MultiPointSemanticsValues = Vec<OptionalIndex>;

/// Index value that can be `null` to indicate the absence of semantic, or appearance on a
/// geometric primitive.
pub type OptionalIndex = Option<usize>;

/// The Boundary representation of a `MultiSolid` or `CompositeSolid`.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#arrays-to-represent-boundaries>
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let ring: MultiPointBoundary = vec![0, 1, 2, 3];
/// let surface: SurfaceBoundary = vec![ring];
/// let shell: AggregateSurfaceBoundary = vec![surface];
/// let solid: SolidBoundary = vec![shell];
/// let asolid: AggregateSolidBoundary = vec![solid];
/// let asol_json = serde_json::to_string(&asolid)?;
/// println!("{:?}", asol_json);
/// # Ok(())
/// # }
pub type AggregateSolidBoundary = Vec<SolidBoundary>;

/// The Boundary representation of a `Solid`.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#arrays-to-represent-boundaries>
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let ring: MultiPointBoundary = vec![0, 1, 2, 3];
/// let surface: SurfaceBoundary = vec![ring];
/// let shell: AggregateSurfaceBoundary = vec![surface];
/// let solid: SolidBoundary = vec![shell];
/// let solid_json = serde_json::to_string(&solid)?;
/// println!("{:?}", solid_json);
/// # Ok(())
/// # }
pub type SolidBoundary = Vec<AggregateSurfaceBoundary>;

/// The Boundary representation of a `MultiSurface`, `CompositeSurface` or `Shell`.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#arrays-to-represent-boundaries>
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let ring: MultiPointBoundary = vec![0, 1, 2, 3];
/// let surface: SurfaceBoundary = vec![ring];
/// // MultiSurface, CompositeSurface or Shell
/// let multisurface: AggregateSurfaceBoundary = vec![surface];
/// let msrf_json = serde_json::to_string(&multisurface)?;
/// println!("{:?}", msrf_json);
/// # Ok(())
/// # }
pub type AggregateSurfaceBoundary = Vec<SurfaceBoundary>;

/// The Boundary representation of a `Surface`.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#arrays-to-represent-boundaries>
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let ring: MultiPointBoundary = vec![0, 1, 2, 3];
/// let surface: SurfaceBoundary = vec![ring];
/// let srf_json = serde_json::to_string(&surface)?;
/// println!("{:?}", srf_json);
/// # Ok(())
/// # }
pub type SurfaceBoundary = Vec<MultiPointBoundary>;

/// The Boundary representation of a `MultiLineString`.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#arrays-to-represent-boundaries>
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let linestring: MultiPointBoundary = vec![0, 1, 2, 3];
/// let multilinestring: SurfaceBoundary = vec![linestring];
/// let mls_json = serde_json::to_string(&multilinestring)?;
/// println!("{:?}", mls_json);
/// # Ok(())
/// # }
pub type MultiLineStringBoundary = Vec<MultiPointBoundary>;

/// The Boundary representation of a `MultiPoint`.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#arrays-to-represent-boundaries>
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let nr_points = 4;
/// let mut mp = MultiPointBoundary::with_capacity(nr_points);
/// for v in 0..nr_points {
///     let p: PointBoundary = v;
///     mp.push(p);
/// }
///
/// let mp: MultiPointBoundary = vec![0, 1, 2, 3];
/// let mp_json = serde_json::to_string(&mp)?;
/// println!("{:?}", mp_json);
/// # Ok(())
/// # }
pub type MultiPointBoundary = Vec<PointBoundary>;

/// The Boundary representation of a `Point`.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#arrays-to-represent-boundaries>
pub type PointBoundary = usize;

/// Vertex coordinates, deserialized from a CityJSON document.
///
/// Uses i64, because when we work with CityJSONFeatures of a very large (national)
/// area, and there is a single, national transformation parameters, then the quantized
/// coordinates can easily go beyond the max i32.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#coordinates-of-the-vertices>
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let vertices_json = r#"[
///   [102, 103, 1],
///   [11, 910, 43],
///   [25, 744, 22],
///   [23, 88, 5],
///   [8523, 487, 22]
/// ]"#;
/// let vertices: Vertices = serde_json::from_str(&vertices_json)?;
/// println!("{:?}", vertices);
/// # Ok(())
/// # }
pub type Vertices = Vec<[i64; 3]>;

/// Metadata for a city model.
///
/// There is only structural validation for the metadata items, the metadata values are not
/// validated. For instance, a contact website must be a string, but it is not
/// checked whether the string is a valid URL or not.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#metadata>
///
/// # Examples
/// You can instantiate the Metadata struct directly, however that requires that you set each field.
/// ```
/// # use serde_cityjson::v1_1::*;
/// let metadata = Metadata {
///     geographical_extent: Some([1.0, 1.0, 1.0, 1.0, 1.0, 1.0]),
///     identifier: Some("123-456-789".to_string()),
///     point_of_contact: Some(Contact {
///         contact_name: "My name".to_string(),
///         email_address: "my@email.com".to_string(),
///         role: Some(ContactRole::Author),
///         organization: Some("Big Org".to_string()),
///         ..Default::default()
///     }),
///     ..Default::default()
/// };
/// println!("{}", &metadata);
/// ```
///
/// It may be more convenient to use the setter methods, which allows you to set only those members
/// that you need.
/// ```
/// # use serde_cityjson::v1_1::*;
/// let mut metadata = Metadata::new();
/// metadata.set_organization("BigOrg");
/// metadata.set_role(ContactRole::Author);
/// metadata.set_contact_name("My Name");
/// metadata.set_email_address("my@email.com");
/// metadata.set_geographical_extent([1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
/// metadata.set_identifier("123-456-789");
/// println!("{}", &metadata);
/// ```
#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct Metadata {
    pub geographical_extent: Option<BBox>,
    pub identifier: Option<CityModelIdentifier>,
    pub point_of_contact: Option<Contact>,
    pub reference_date: Option<Date>,
    pub reference_system: Option<CRS>,
    pub title: Option<String>,
}

/// Bounding Box.
///
/// An array of 6 values: `[minx, miny, minz, maxx, maxy, maxz]`.
/// Specs: <https://www.cityjson.org/specs/1.1.3/#geographicalextent-bbox>
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// let bbox: BBox = [ 84710.1, 446846.0, -5.3, 84757.1, 446944.0, 40.9 ];
/// ```
pub type BBox = [f32; 6];

/// An identifier for the dataset.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#identifier>
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// let city_id = CityModelIdentifier::from("44574905-d2d2-4f40-8e96-d39e1ae45f70");
/// ```
pub type CityModelIdentifier = String;

/// The point of contact for the city model.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#pointofcontact>
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let poc_json = r#"{
///     "contactName": "One Person",
///     "emailAddress": "one.person@parl.gc.ca",
///     "phone": "+1-613-992-4211",
///     "address": "24 Sussex Drive, Ottawa, Canada",
///     "contactType": "individual",
///     "website": "https://www.website.gc.ca",
///     "role": "pointOfContact",
///     "organization": "Big Org"
/// }"#;
/// let contact: Contact = serde_json::from_str(&poc_json)?;
/// println!("{}", &contact);
/// let poc_json = serde_json::to_string(&contact)?;
/// println!("{}", &poc_json);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Default, Debug, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct Contact {
    pub contact_name: String,
    pub email_address: String,
    pub role: Option<ContactRole>,
    pub website: Option<String>,
    pub contact_type: Option<ContactType>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub organization: Option<String>,
}

/// Metadata contact role.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#pointofcontact>
#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub enum ContactRole {
    Author,
    CoAuthor,
    Collaborator,
    Contributor,
    Custodian,
    Distributor,
    Editor,
    Funder,
    Mediator,
    Originator,
    Owner,
    PointOfContact,
    PrincipalInvestigator,
    Processor,
    Publisher,
    ResourceProvider,
    RightsHolder,
    Sponsor,
    Stakeholder,
    User,
}

/// Metadata contact type.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#pointofcontact>
#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub enum ContactType {
    Individual,
    Organization,
}

/// The date when the dataset was compiled.
///
/// The format is a `"full-date"` per the
/// [RFC 3339, Section 5.6](https://tools.ietf.org/html/rfc3339#section-5.6).
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#referencedate>
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// let date = Date::from("1977-02-28");
/// ```
pub type Date = String;

// Note: Could also have a CRS struct with named members but that's too much complication, because
// it brings a lot of implementation with it (Display, FromStr, Into<String>, ...), incl.
// validation. And the philosophy with the other Metadata members is that we accept almost any
// value, because too pedantic validation might actually get in the way of building city
// models. And then it is better to push the validation down to specialized libraries, such as
// cjval.
// #[derive(Clone, Default, Debug, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
// pub struct CRS {
//     authority: String,
//     version: i8,
//     code: i16,
// }
//
// impl Display for CRS {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "http://www.opengis.net/def/crs/{authority}/{version}/{code}",
//             authority = self.authority,
//             version = self.version,
//             code = self.code
//         )
//     }
// }
/// The coordinate reference system (CRS) of the city model.
///
/// Must be formatted as a URL, according to the
/// [OGC Name Type Specification](https://docs.opengeospatial.org/pol/09-048r5.html#_production_rule_for_specification_element_names).
/// Specs: <https://www.cityjson.org/specs/1.1.3/#referencesystem-crs>
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// let crs = CRS::from("https://www.opengis.net/def/crs/EPSG/0/7415");
/// ```
pub type CRS = String;

/// The `extensions` member of a CityJSON.
pub type Extensions = HashMap<String, Extension>;

/// An Extension that is used in a city model.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#using-an-extension-in-a-cityjson-file>
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let extension_json = r#"{
///     "url" : "https://someurl.org/noise.json",
///     "version": "2.0"
/// }"#;
/// let extension: Extension = serde_json::from_str(&extension_json)?;
/// println!("{}", &extension);
/// let extension_json = serde_json::to_string(&extension)?;
/// println!("{}", &extension_json);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Default, Debug, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct Extension {
    pub url: String,
    pub version: String,
}

// --- Implementations

impl CityModel {
    pub fn new(
        id: Option<String>,
        type_cm: Option<CityModelType>,
        version: Option<CityJSONVersion>,
        transform: Option<Transform>,
        cityobjects: Option<CityObjects>,
        vertices: Option<Vertices>,
        metadata: Option<Metadata>,
        appearance: Option<Appearance>,
        geometry_templates: Option<GeometryTemplates>,
        extra: Option<HashMap<String, serde_json::Value>>,
        extensions: Option<Extensions>,
    ) -> Self {
        Self {
            id,
            type_cm: type_cm.unwrap_or_default(),
            version,
            transform,
            cityobjects: cityobjects.unwrap_or_default(),
            vertices: vertices.unwrap_or_default(),
            metadata,
            appearance,
            geometry_templates,
            extra,
            extensions,
        }
    }

    // Prints a hierarchy of members, including the amount of heap memory used by them.
    #[cfg(feature = "datasize")]
    fn print_datasize(&self) -> Vec<u8> {
        let mut w: Vec<u8> = Vec::new();
        writeln!(
            &mut w,
            "| {0: <10} | {1: <15} |",
            data_size(&self),
            "CityModel"
        ).unwrap();
        writeln!(
            &mut w,
            "| {0: <10} | {1: <15} |",
            data_size(&self.cityobjects),
            "CityObjects"
        ).unwrap();
        return w;
    }
}

fn sizeof_attributes_option(a: &Option<Attributes>) -> usize {
    if let Some(ref attributes) = a {
        attributes
            .iter()
            .map(|(k, v)| {
                std::mem::size_of::<String>()
                    + k.capacity()
                    + sizeof_serde_value(v)
                    + std::mem::size_of::<usize>() * 3
            })
            .sum()
    } else {
        0
    }
}

// From https://stackoverflow.com/a/76456111
fn sizeof_serde_value(v: &serde_json::Value) -> usize {
    std::mem::size_of::<serde_json::Value>()
        + match v {
            serde_json::Value::Null => 0,
            serde_json::Value::Bool(_) => 0,
            serde_json::Value::Number(_) => 0, // Incorrect if arbitrary_precision is enabled. oh well
            serde_json::Value::String(s) => s.capacity(),
            serde_json::Value::Array(a) => a.iter().map(sizeof_serde_value).sum(),
            serde_json::Value::Object(o) => o
                .iter()
                .map(|(k, v)| {
                    std::mem::size_of::<String>()
                        + k.capacity()
                        + sizeof_serde_value(v)
                        + std::mem::size_of::<usize>() * 3 // As a crude approximation, I pretend each map entry has 3 words of overhead
                })
                .sum(),
        }
}

#[derive(Debug, Default)]
struct CityModelDataSize {
    size_id: usize,
    size_type_cm: usize,
    size_version: usize,
    size_transform: usize,
    count_co: usize,
    size_total_coid: usize,
    count_geometry: usize,
    size_total_geometry: usize,
    geometries: Vec<GeometryDataSize>,
    count_attributes: usize,
    size_total_attributes: usize,
    count_geographical_extent: usize,
    size_total_geographical_extent: usize,
    count_children: usize,
    size_total_children_id: usize,
    count_parents: usize,
    size_total_parents_id: usize,
    size_vertices: usize,
    size_metadata: usize,
    size_appearance: usize,
    size_geometry_templates: usize,
    size_extra: usize,
    size_extensions: usize
}

#[derive(Debug)]
struct GeometryDataSize {
    count: usize,
    total: usize,
    lod: LoD,
    boundaries: usize,
    semantics: usize,
    material: usize,
    texture: usize,
}

impl Default for GeometryDataSize {
    fn default() -> Self {
        Self {
            count: 0,
            total: 0,
            lod: LoD::LoD0,
            boundaries: 0,
            semantics: 0,
            material: 0,
            texture: 0,
        }
    }
}

impl GeometryDataSize {
    fn add_geometry(&mut self, geom: &Geometry) {
        match &geom {
            Geometry::MultiSurface {
                boundaries,
                semantics,
                material,
                texture,
                ..
            } => {
                self.boundaries += total_heap_stack_size(boundaries);
                self.semantics += total_heap_stack_size(semantics);
                self.texture += total_heap_stack_size(texture);
                self.material += total_heap_stack_size(material);
            }
            Geometry::Solid {
                boundaries,
                semantics,
                material,
                texture,
                ..
            } => {
                self.boundaries += total_heap_stack_size(boundaries);
                self.semantics += total_heap_stack_size(semantics);
                self.texture += total_heap_stack_size(texture);
                self.material += total_heap_stack_size(material);
            }
            _ => {}
        }
    }
}

fn add_to_geometrydatasize_lod(
    geom: &Geometry,
    lod: &LoD,
    geom_lod0_size: &mut GeometryDataSize,
    geom_lod12_size: &mut GeometryDataSize,
    geom_lod13_size: &mut GeometryDataSize,
    geom_lod22_size: &mut GeometryDataSize,
) {
    if *lod == LoD::LoD0 {
        geom_lod0_size.count += 1;
        geom_lod0_size.total += total_heap_stack_size(geom);
        geom_lod0_size.add_geometry(geom);
    } else if *lod == LoD::LoD1_2 {
        geom_lod12_size.count += 1;
        geom_lod12_size.total += total_heap_stack_size(geom);
        geom_lod12_size.add_geometry(geom);
    } else if *lod == LoD::LoD1_3 {
        geom_lod13_size.count += 1;
        geom_lod13_size.total += total_heap_stack_size(geom);
        geom_lod13_size.add_geometry(geom);
    } else if *lod == LoD::LoD2_2 {
        geom_lod22_size.count += 1;
        geom_lod22_size.total += total_heap_stack_size(geom);
        geom_lod22_size.add_geometry(geom);
    }
}

/// Calculate the total heap and stack size of a variable.
fn total_heap_stack_size<T: DataSize>(data: &T) -> usize {
    data_size(data) + std::mem::size_of_val(data)
}

#[derive(DataSize)]
struct CityModelSerdeValue {
    #[cfg_attr(feature = "datasize", data_size(with = sizeof_serde_value))]
    inner: serde_json::Value
}

mod test_datasize {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    #[test]
    fn bag3d() {
        let dummy_complete = PathBuf::from("resources")
            .join("data")
            .join("downloaded")
            .join("10-356-724_one.city.json");
        let mut file = File::open(dummy_complete).unwrap();
        let mut cityjson_json = String::new();
        file.read_to_string(&mut cityjson_json).unwrap();

        let mut cm_size = CityModelDataSize {
            ..Default::default()
        };
        let mut geom_lod0_size = GeometryDataSize {
            lod: LoD::LoD0,
            ..Default::default()
        };
        let mut geom_lod12_size = GeometryDataSize {
            lod: LoD::LoD1_2,
            ..Default::default()
        };
        let mut geom_lod13_size = GeometryDataSize {
            lod: LoD::LoD1_3,
            ..Default::default()
        };
        let mut geom_lod22_size = GeometryDataSize {
            lod: LoD::LoD2_2,
            ..Default::default()
        };
        let cm: CityModel = serde_json::from_str(&cityjson_json).unwrap();
        let cm_serde_value = CityModelSerdeValue {
            inner: serde_json::from_str(&cityjson_json).unwrap()
        };
        cm_size.size_id = total_heap_stack_size(&cm.id);
        cm_size.size_type_cm = total_heap_stack_size(&cm.type_cm);
        cm_size.size_version = total_heap_stack_size(&cm.version);
        cm_size.size_transform = total_heap_stack_size(&cm.transform);
        cm_size.size_vertices = total_heap_stack_size(&cm.vertices);
        cm_size.size_metadata = total_heap_stack_size(&cm.metadata);
        cm_size.size_appearance = total_heap_stack_size(&cm.appearance);
        cm_size.size_geometry_templates = total_heap_stack_size(&cm.geometry_templates);
        cm_size.size_extra = total_heap_stack_size(&cm.extra.as_ref());
        cm_size.size_extensions = total_heap_stack_size(&cm.extensions);

        for (coid, co) in cm.cityobjects.iter() {
            cm_size.count_co += 1;
            cm_size.size_total_coid += total_heap_stack_size(coid);
            for geom in co.geometry.iter() {
                cm_size.count_geometry += 1;
                cm_size.size_total_geometry += total_heap_stack_size(geom);
                match geom {
                    Geometry::MultiSurface { lod, .. } => {
                        add_to_geometrydatasize_lod(
                            geom,
                            lod,
                            &mut geom_lod0_size,
                            &mut geom_lod12_size,
                            &mut geom_lod13_size,
                            &mut geom_lod22_size,
                        );
                    }
                    Geometry::Solid { lod, .. } => {
                        add_to_geometrydatasize_lod(
                            geom,
                            lod,
                            &mut geom_lod0_size,
                            &mut geom_lod12_size,
                            &mut geom_lod13_size,
                            &mut geom_lod22_size,
                        );
                    }
                    _ => {}
                }
            }
            if co.attributes.is_some() {
                cm_size.count_attributes += 1;
            }
            cm_size.size_total_attributes += total_heap_stack_size(&co.attributes.as_ref());
            if co.geographical_extent.is_some() {
                cm_size.count_geographical_extent += 1;
            }
            cm_size.size_total_geographical_extent += total_heap_stack_size(&co.geographical_extent);
            if let Some(ref children) = co.children {
                cm_size.count_children += children.len();
            }
            cm_size.size_total_children_id += total_heap_stack_size(&co.children);
            if let Some(ref parents) = co.parents {
                cm_size.count_parents += parents.len();
            }
            cm_size.size_total_parents_id += total_heap_stack_size(&co.parents);
        }
        cm_size.geometries = vec![geom_lod0_size, geom_lod12_size, geom_lod13_size, geom_lod22_size];
        println!("CityJSON string: {}", total_heap_stack_size(&cityjson_json));
        println!("CityModel serde_json::Value : {}", total_heap_stack_size(&cm_serde_value));
        println!("CityModel total: {}", total_heap_stack_size(&cm));
        println!("{:#?}", &cm_size);
    }
}

impl Default for CityModel {
    fn default() -> Self {
        Self {
            id: None,
            type_cm: CityModelType::CityJSON,
            version: Some(CityJSONVersion::V1_1),
            transform: Some(Transform::default()),
            cityobjects: Default::default(),
            vertices: vec![],
            metadata: None,
            appearance: None,
            geometry_templates: None,
            extra: None,
            extensions: None,
        }
    }
}

impl CityJSONVersion {
    fn _from_str(value: &str) -> Result<CityJSONVersion> {
        match value {
            "1.1" | "1.1.1" | "1.1.2" | "1.1.3" => Ok(CityJSONVersion::V1_1),
            _ => Err(Error::UnsupportedVersion(
                value.to_string(),
                "1.1, 1.1.1, 1.1.2, 1.1.3".to_string(),
            )),
        }
    }
}

impl Display for CityJSONVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            CityJSONVersion::V1_1 => {
                write!(f, "1.1")
            }
        }
    }
}

impl TryFrom<&str> for CityJSONVersion {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        CityJSONVersion::_from_str(value)
    }
}

impl TryFrom<String> for CityJSONVersion {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        CityJSONVersion::_from_str(value.as_ref())
    }
}

/// This implementation is only used for serializing the CityJSON version, because serde cannot
/// serialize from 'try_into' (which is provided by the 'try_from' implementations).
/// So we need this Into, even though [std says that one should avoid implementing Into](https://doc.rust-lang.org/std/convert/trait.Into.html).
impl Into<String> for CityJSONVersion {
    fn into(self) -> String {
        match self {
            CityJSONVersion::V1_1 => String::from("1.1"),
        }
    }
}

impl Display for CityModelType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            CityModelType::CityJSON => {
                write!(f, "CityJSON")
            }
            CityModelType::CityJSONFeature => {
                write!(f, "CityJSONFeature")
            }
        }
    }
}

impl Transform {
    pub fn new() -> Self {
        Self {
            scale: [1.0, 1.0, 1.0],
            translate: [0.0, 0.0, 0.0],
        }
    }
}

impl Display for Transform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(scale: [{}, {}, {}], translate:[{}, {}, {}])",
            self.scale[0],
            self.scale[1],
            self.scale[2],
            self.translate[0],
            self.translate[1],
            self.translate[2]
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl CityObject {
    pub fn new(
        cotype: CityObjectType,
        geometry: Vec<Geometry>,
        attributes: Option<Attributes>,
        geographical_extent: Option<BBox>,
        children: Option<Vec<String>>,
        parents: Option<Vec<String>>,
    ) -> Self {
        Self {
            type_co: cotype,
            geometry,
            attributes,
            geographical_extent,
            children,
            parents,
        }
    }
}

impl<'de> Deserialize<'de> for CityObjectType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(CityObjectTypeVisitor)
    }
}

struct CityObjectTypeVisitor;

impl<'de> Visitor<'de> for CityObjectTypeVisitor {
    type Value = CityObjectType;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string of a valid CityObject type")
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        // It would be nice to be case-insensitive, however the &str.to_lowercase()
        // method allocates a new String for the output, which would mean an extra allocation for
        // each semantic type in the data.
        match value {
            "Bridge" => Ok(CityObjectType::Bridge),
            "BridgePart" => Ok(CityObjectType::BridgePart),
            "BridgeInstallation" => Ok(CityObjectType::BridgeInstallation),
            "BridgeConstructiveElement" => Ok(CityObjectType::BridgeConstructiveElement),
            "BridgeRoom" => Ok(CityObjectType::BridgeRoom),
            "BridgeFurniture" => Ok(CityObjectType::BridgeFurniture),
            "Building" => Ok(CityObjectType::Building),
            "BuildingPart" => Ok(CityObjectType::BuildingPart),
            "BuildingInstallation" => Ok(CityObjectType::BuildingInstallation),
            "BuildingConstructiveElement" => Ok(CityObjectType::BuildingConstructiveElement),
            "BuildingFurniture" => Ok(CityObjectType::BuildingFurniture),
            "BuildingStorey" => Ok(CityObjectType::BuildingStorey),
            "BuildingRoom" => Ok(CityObjectType::BuildingRoom),
            "BuildingUnit" => Ok(CityObjectType::BuildingUnit),
            "CityFurniture" => Ok(CityObjectType::CityFurniture),
            "CityObjectGroup" => Ok(CityObjectType::CityObjectGroup),
            "LandUse" => Ok(CityObjectType::LandUse),
            "OtherConstruction" => Ok(CityObjectType::OtherConstruction),
            "PlantCover" => Ok(CityObjectType::PlantCover),
            "SolitaryVegetationObject" => Ok(CityObjectType::SolitaryVegetationObject),
            "TINRelief" => Ok(CityObjectType::TINRelief),
            "WaterBody" => Ok(CityObjectType::WaterBody),
            "Road" => Ok(CityObjectType::Road),
            "Railway" => Ok(CityObjectType::Railway),
            "Waterway" => Ok(CityObjectType::Waterway),
            "TransportSquare" => Ok(CityObjectType::TransportSquare),
            "Tunnel" => Ok(CityObjectType::Tunnel),
            "TunnelPart" => Ok(CityObjectType::TunnelPart),
            "TunnelInstallation" => Ok(CityObjectType::TunnelInstallation),
            "TunnelConstructiveElement" => Ok(CityObjectType::TunnelConstructiveElement),
            "TunnelHollowSpace" => Ok(CityObjectType::TunnelHollowSpace),
            "TunnelFurniture" => Ok(CityObjectType::TunnelFurniture),
            &_ => {
                if value
                    .chars()
                    .nth(0)
                    .is_some_and(|first_char| first_char == '+')
                {
                    Ok(CityObjectType::Extension(value.to_string()))
                } else {
                    Err(serde::de::Error::custom(format!(
                        "invalid CityObject type: {}",
                        value
                    )))
                }
            }
        }
    }
}

impl Serialize for CityObjectType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match *self {
            CityObjectType::Bridge => {
                serializer.serialize_unit_variant("CityObjectType", 0, "Bridge")
            }
            CityObjectType::BridgePart => {
                serializer.serialize_unit_variant("CityObjectType", 1, "BridgePart")
            }
            CityObjectType::BridgeInstallation => {
                serializer.serialize_unit_variant("CityObjectType", 2, "BridgeInstallation")
            }
            CityObjectType::BridgeConstructiveElement => {
                serializer.serialize_unit_variant("CityObjectType", 3, "BridgeConstructiveElement")
            }
            CityObjectType::BridgeRoom => {
                serializer.serialize_unit_variant("CityObjectType", 4, "BridgeRoom")
            }
            CityObjectType::BridgeFurniture => {
                serializer.serialize_unit_variant("CityObjectType", 5, "BridgeFurniture")
            }
            CityObjectType::Building => {
                serializer.serialize_unit_variant("CityObjectType", 6, "Building")
            }
            CityObjectType::BuildingPart => {
                serializer.serialize_unit_variant("CityObjectType", 7, "BuildingPart")
            }
            CityObjectType::BuildingInstallation => {
                serializer.serialize_unit_variant("CityObjectType", 8, "BuildingInstallation")
            }
            CityObjectType::BuildingConstructiveElement => serializer.serialize_unit_variant(
                "CityObjectType",
                9,
                "BuildingConstructiveElement",
            ),
            CityObjectType::BuildingFurniture => {
                serializer.serialize_unit_variant("CityObjectType", 10, "BuildingFurniture")
            }
            CityObjectType::BuildingStorey => {
                serializer.serialize_unit_variant("CityObjectType", 11, "BuildingStorey")
            }
            CityObjectType::BuildingRoom => {
                serializer.serialize_unit_variant("CityObjectType", 12, "BuildingRoom")
            }
            CityObjectType::BuildingUnit => {
                serializer.serialize_unit_variant("CityObjectType", 13, "BuildingUnit")
            }
            CityObjectType::CityFurniture => {
                serializer.serialize_unit_variant("CityObjectType", 14, "CityFurniture")
            }
            CityObjectType::CityObjectGroup => {
                serializer.serialize_unit_variant("CityObjectType", 15, "CityObjectGroup")
            }
            CityObjectType::Default => {
                serializer.serialize_unit_variant("CityObjectType", 16, "Default")
            }
            CityObjectType::LandUse => {
                serializer.serialize_unit_variant("CityObjectType", 17, "LandUse")
            }
            CityObjectType::OtherConstruction => {
                serializer.serialize_unit_variant("CityObjectType", 18, "OtherConstruction")
            }
            CityObjectType::PlantCover => {
                serializer.serialize_unit_variant("CityObjectType", 19, "PlantCover")
            }
            CityObjectType::SolitaryVegetationObject => {
                serializer.serialize_unit_variant("CityObjectType", 20, "SolitaryVegetationObject")
            }
            CityObjectType::TINRelief => {
                serializer.serialize_unit_variant("CityObjectType", 21, "TINRelief")
            }
            CityObjectType::WaterBody => {
                serializer.serialize_unit_variant("CityObjectType", 22, "WaterBody")
            }
            CityObjectType::Road => serializer.serialize_unit_variant("CityObjectType", 23, "Road"),
            CityObjectType::Railway => {
                serializer.serialize_unit_variant("CityObjectType", 24, "Railway")
            }
            CityObjectType::Waterway => {
                serializer.serialize_unit_variant("CityObjectType", 25, "Waterway")
            }
            CityObjectType::TransportSquare => {
                serializer.serialize_unit_variant("CityObjectType", 26, "TransportSquare")
            }
            CityObjectType::Tunnel => {
                serializer.serialize_unit_variant("CityObjectType", 27, "Tunnel")
            }
            CityObjectType::TunnelPart => {
                serializer.serialize_unit_variant("CityObjectType", 28, "TunnelPart")
            }
            CityObjectType::TunnelInstallation => {
                serializer.serialize_unit_variant("CityObjectType", 29, "TunnelInstallation")
            }
            CityObjectType::TunnelConstructiveElement => {
                serializer.serialize_unit_variant("CityObjectType", 30, "TunnelConstructiveElement")
            }
            CityObjectType::TunnelHollowSpace => {
                serializer.serialize_unit_variant("CityObjectType", 31, "TunnelHollowSpace")
            }
            CityObjectType::TunnelFurniture => {
                serializer.serialize_unit_variant("CityObjectType", 32, "TunnelFurniture")
            }
            CityObjectType::Extension(ref s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for LoD {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(LoDVisitor)
    }
}

struct LoDVisitor;

impl<'de> Visitor<'de> for LoDVisitor {
    type Value = LoD;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string with a valid Level of Detail value")
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match value {
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
            &_ => Err(serde::de::Error::custom(format!(
                "invalid Level of Detail value: {}",
                value
            ))),
        }
    }
}

impl Serialize for LoD {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match *self {
            LoD::LoD0 => serializer.serialize_str("0"),
            LoD::LoD0_0 => serializer.serialize_str("0.0"),
            LoD::LoD0_1 => serializer.serialize_str("0.1"),
            LoD::LoD0_2 => serializer.serialize_str("0.2"),
            LoD::LoD0_3 => serializer.serialize_str("0.3"),
            LoD::LoD1 => serializer.serialize_str("1"),
            LoD::LoD1_0 => serializer.serialize_str("1.0"),
            LoD::LoD1_1 => serializer.serialize_str("1.1"),
            LoD::LoD1_2 => serializer.serialize_str("1.2"),
            LoD::LoD1_3 => serializer.serialize_str("1.3"),
            LoD::LoD2 => serializer.serialize_str("2"),
            LoD::LoD2_0 => serializer.serialize_str("2.0"),
            LoD::LoD2_1 => serializer.serialize_str("2.1"),
            LoD::LoD2_2 => serializer.serialize_str("2.2"),
            LoD::LoD2_3 => serializer.serialize_str("2.3"),
            LoD::LoD3 => serializer.serialize_str("3"),
            LoD::LoD3_0 => serializer.serialize_str("3.0"),
            LoD::LoD3_1 => serializer.serialize_str("3.1"),
            LoD::LoD3_2 => serializer.serialize_str("3.2"),
            LoD::LoD3_3 => serializer.serialize_str("3.3"),
        }
    }
}

impl Default for ImageType {
    fn default() -> Self {
        ImageType::Png
    }
}

impl Display for SemanticType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'de> Deserialize<'de> for SemanticType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(SemanticTypeVisitor)
    }
}

struct SemanticTypeVisitor;

impl<'de> Visitor<'de> for SemanticTypeVisitor {
    type Value = SemanticType;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string of a valid Semantic type")
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        // It would be nice to be case-insensitive, however the &str.to_lowercase()
        // method allocates a new String for the output, which would mean an extra allocation for
        // each semantic type in the data.
        match value {
            "RoofSurface" => Ok(SemanticType::RoofSurface),
            "GroundSurface" => Ok(SemanticType::GroundSurface),
            "WallSurface" => Ok(SemanticType::WallSurface),
            "ClosureSurface" => Ok(SemanticType::ClosureSurface),
            "OuterCeilingSurface" => Ok(SemanticType::OuterCeilingSurface),
            "OuterFloorSurface" => Ok(SemanticType::OuterFloorSurface),
            "Window" => Ok(SemanticType::Window),
            "Door" => Ok(SemanticType::Door),
            "InteriorWallSurface" => Ok(SemanticType::InteriorWallSurface),
            "CeilingSurface" => Ok(SemanticType::CeilingSurface),
            "FloorSurface" => Ok(SemanticType::FloorSurface),
            "WaterSurface" => Ok(SemanticType::WaterSurface),
            "WaterGroundSurface" => Ok(SemanticType::WaterGroundSurface),
            "WaterClosureSurface" => Ok(SemanticType::WaterClosureSurface),
            "TrafficArea" => Ok(SemanticType::TrafficArea),
            "AuxiliaryTrafficArea" => Ok(SemanticType::AuxiliaryTrafficArea),
            "TransportationMarking" => Ok(SemanticType::TransportationMarking),
            "TransportationHole" => Ok(SemanticType::TransportationHole),
            &_ => {
                if value
                    .chars()
                    .nth(0)
                    .is_some_and(|first_char| first_char == '+')
                {
                    Ok(SemanticType::Extension(value.to_string()))
                } else {
                    Err(serde::de::Error::custom(format!(
                        "invalid Semantic type: {}",
                        value
                    )))
                }
            }
        }
    }
}

impl Serialize for SemanticType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match *self {
            SemanticType::RoofSurface => {
                serializer.serialize_unit_variant("SemanticType", 0, "RoofSurface")
            }
            SemanticType::GroundSurface => {
                serializer.serialize_unit_variant("SemanticType", 1, "GroundSurface")
            }
            SemanticType::WallSurface => {
                serializer.serialize_unit_variant("SemanticType", 2, "WallSurface")
            }
            SemanticType::ClosureSurface => {
                serializer.serialize_unit_variant("SemanticType", 3, "ClosureSurface")
            }
            SemanticType::OuterCeilingSurface => {
                serializer.serialize_unit_variant("SemanticType", 4, "OuterCeilingSurface")
            }
            SemanticType::OuterFloorSurface => {
                serializer.serialize_unit_variant("SemanticType", 5, "OuterFloorSurface")
            }
            SemanticType::Window => serializer.serialize_unit_variant("SemanticType", 6, "Window"),
            SemanticType::Door => serializer.serialize_unit_variant("SemanticType", 7, "Door"),
            SemanticType::InteriorWallSurface => {
                serializer.serialize_unit_variant("SemanticType", 8, "InteriorWallSurface")
            }
            SemanticType::CeilingSurface => {
                serializer.serialize_unit_variant("SemanticType", 9, "CeilingSurface")
            }
            SemanticType::FloorSurface => {
                serializer.serialize_unit_variant("SemanticType", 10, "FloorSurface")
            }
            SemanticType::WaterSurface => {
                serializer.serialize_unit_variant("SemanticType", 11, "WaterSurface")
            }
            SemanticType::WaterGroundSurface => {
                serializer.serialize_unit_variant("SemanticType", 12, "WaterGroundSurface")
            }
            SemanticType::WaterClosureSurface => {
                serializer.serialize_unit_variant("SemanticType", 13, "WaterClosureSurface")
            }
            SemanticType::TrafficArea => {
                serializer.serialize_unit_variant("SemanticType", 14, "TrafficArea")
            }
            SemanticType::AuxiliaryTrafficArea => {
                serializer.serialize_unit_variant("SemanticType", 15, "AuxiliaryTrafficArea")
            }
            SemanticType::TransportationMarking => {
                serializer.serialize_unit_variant("SemanticType", 16, "TransportationMarking")
            }
            SemanticType::TransportationHole => {
                serializer.serialize_unit_variant("SemanticType", 17, "TransportationHole")
            }
            SemanticType::Extension(ref s) => serializer.serialize_str(s),
        }
    }
}

impl CompositeSolidSemantics {
    pub fn new(surfaces: Vec<Semantic>, values: CompositeSolidSemanticsValues) -> Self {
        Self { surfaces, values }
    }
}

impl MultiSolidSemantics {
    pub fn new(surfaces: Vec<Semantic>, values: MultiSolidSemanticsValues) -> Self {
        Self { surfaces, values }
    }
}

impl SolidSemantics {
    pub fn new(surfaces: Vec<Semantic>, values: SolidSemanticsValues) -> Self {
        Self { surfaces, values }
    }
}

impl CompositeSurfaceSemantics {
    pub fn new(surfaces: Vec<Semantic>, values: CompositeSurfaceSemanticsValues) -> Self {
        Self { surfaces, values }
    }
}

impl MultiSurfaceSemantics {
    pub fn new(surfaces: Vec<Semantic>, values: MultiSurfaceSemanticsValues) -> Self {
        Self { surfaces, values }
    }
}

impl MultiLineStringSemantics {
    pub fn new(surfaces: Vec<Semantic>, values: MultiLineStringSemanticsValues) -> Self {
        Self { surfaces, values }
    }
}

impl MultiPointSemantics {
    pub fn new(surfaces: Vec<Semantic>, values: MultiPointSemanticsValues) -> Self {
        Self { surfaces, values }
    }
}

impl Metadata {
    pub fn new() -> Self {
        Metadata::default()
    }

    pub fn set_geographical_extent(&mut self, bbox: BBox) {
        self.geographical_extent = Some(bbox);
    }

    pub fn set_identifier<S: AsRef<str>>(&mut self, identifier: S) {
        self.identifier = Some(identifier.as_ref().to_owned());
    }

    pub fn set_reference_date<S: AsRef<str>>(&mut self, date: S) {
        self.reference_date = Some(date.as_ref().to_owned());
    }

    pub fn set_reference_system<S: AsRef<str>>(&mut self, crs: S) {
        self.reference_system = Some(crs.as_ref().to_owned());
    }

    pub fn set_title<S: AsRef<str>>(&mut self, title: S) {
        self.title = Some(title.as_ref().to_owned());
    }

    pub fn set_contact_name<S: AsRef<str>>(&mut self, name: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.contact_name = name.as_ref().to_owned()
        } else {
            self.point_of_contact = Some(Contact {
                contact_name: name.as_ref().to_owned(),
                ..Default::default()
            })
        }
    }

    pub fn set_email_address<S: AsRef<str>>(&mut self, email: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.email_address = email.as_ref().to_owned()
        } else {
            self.point_of_contact = Some(Contact {
                email_address: email.as_ref().to_owned(),
                ..Default::default()
            })
        }
    }

    pub fn set_role(&mut self, role: ContactRole) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.role = Some(role);
        } else {
            self.point_of_contact = Some(Contact {
                role: Some(role),
                ..Default::default()
            })
        }
    }

    pub fn set_website<S: AsRef<str>>(&mut self, website: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.website = Some(website.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                website: Some(website.as_ref().to_owned()),
                ..Default::default()
            })
        }
    }

    pub fn set_contact_type(&mut self, contact_type: ContactType) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.contact_type = Some(contact_type);
        } else {
            self.point_of_contact = Some(Contact {
                contact_type: Some(contact_type),
                ..Default::default()
            })
        }
    }

    pub fn set_address<S: AsRef<str>>(&mut self, address: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.address = Some(address.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                address: Some(address.as_ref().to_owned()),
                ..Default::default()
            })
        }
    }

    pub fn set_phone<S: AsRef<str>>(&mut self, phone: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.phone = Some(phone.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                phone: Some(phone.as_ref().to_owned()),
                ..Default::default()
            })
        }
    }

    pub fn set_organization<S: AsRef<str>>(&mut self, organization: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.organization = Some(organization.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                organization: Some(organization.as_ref().to_owned()),
                ..Default::default()
            })
        }
    }
}

impl Display for Metadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "geographical_extent: {:?}, identifier: {:?}, point_of_contact: {:?},
        reference_date: {:?}, reference_system: {:?}, title: {:?}",
            self.geographical_extent,
            self.identifier,
            self.point_of_contact,
            self.reference_date,
            self.reference_system,
            self.title
        )
    }
}

impl Display for Contact {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "contact_name: {}, email_address: {}, role: {:?}, website: {:?},
        contact_type: {:?}, address: {:?}, phone: {:?}, organization: {:?},",
            self.contact_name,
            self.email_address,
            self.role,
            self.website,
            self.contact_type,
            self.address,
            self.phone,
            self.organization
        )
    }
}

impl Display for ContactRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl Display for ContactType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl Display for Extension {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "url: {}, version: {}", self.url, self.version)
    }
}

pub fn deserialize_attributes<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Attributes>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s = HashMap::deserialize(deserializer)?;
    Ok((s.len() != 0).then_some(s))
}
