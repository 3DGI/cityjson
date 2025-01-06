//! CityJSON version 1.1
//!
//! Specs: <https://www.cityjson.org/specs/1.1.3/>.
//!
//! The main struct is [CityModel], which represents a CityJSON or CityJSONFeature object.
//! See the examples of usage by the various members.

use std::borrow::Cow;
use std::collections::HashMap as Map;
use std::fmt::Formatter;

#[cfg(feature = "datasize")]
use datasize::DataSize;
use derive_more::Display;
use serde::de::{IntoDeserializer, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::value::RawValue;
use std::result;
// use ahash::AHashMap as Map;

use crate::attributes::{deserialize_attributes, serialize_attributes, Attributes};
use crate::boundary::{
    Boundary, ExtendRingsVisitor, ExtendShellsVisitor, ExtendSolidsVisitor, ExtendSurfacesVisitor,
    ExtendVerticesVisitor,
};
#[cfg(feature = "datasize")]
use crate::datasize::sizeof_attributes_option;
use crate::indices::{GeometryIndex, OptionalGeometryIndex};
use crate::labels;

// TODO: rename all type_X to type_

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
/// let json_str = r#"{
///   "type": "CityJSON",
///   "version": "1.1",
///   "transform": {
///     "scale": [1.0, 1.0, 1.0],
///     "translate": [0.0, 0.0, 0.0]
///   },
///   "CityObjects": {},
///   "vertices": [],
///   "metadata": {},
///   "appearance": {},
///   "geometry-templates": {
///     "templates": [],
///     "vertices-templates": []
///   },
///   "extensions": {}
/// }"#;
/// let cm: CityModel = serde_json::from_str(json_str)?;
/// let cm_json = serde_json::to_string(&cm)?;
/// assert_eq!(cm_json, json_str.replace(" ", "").replace("\n", ""));
/// # Ok(())
/// # }
/// ```
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let json_str = r#"{
///   "type": "CityJSONFeature",
///   "id": "myid",
///   "CityObjects": {},
///   "vertices": [],
///   "appearance": {}
/// }"#;
/// let cjfeature: CityModel = serde_json::from_str(json_str)?;
/// let cjfeature_json = serde_json::to_string(&cjfeature)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct CityModel<'cm> {
    #[serde(borrow, skip_serializing_if = "Option::is_none")]
    pub id: Option<Cow<'cm, str>>,
    #[serde(rename = "type")]
    pub type_cm: crate::CityModelType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<crate::CityJSONVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<Transform>,
    #[serde(borrow, rename = "CityObjects")]
    pub cityobjects: CityObjects<'cm>,
    pub vertices: Vertices,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata<'cm>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appearance: Option<Appearance<'cm>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "geometry-templates")]
    pub geometry_templates: Option<GeometryTemplates<'cm>>,
    #[serde(
        borrow,
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_attributes",
        serialize_with = "serialize_attributes"
    )]
    #[cfg_attr(feature = "datasize", data_size(with = sizeof_attributes_option))]
    pub extra: Option<Attributes<'cm>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
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
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct Transform {
    pub scale: [f64; 3],
    pub translate: [f64; 3],
}

/// The `CityObjects` member of CityJSON.
pub type CityObjects<'cm> = Map<Cow<'cm, str>, CityObject<'cm>>;

/// CityObject.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#the-different-city-objects>.
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let co: CityObject = serde_json::from_str(r#"{
///   "type": "+ExtendedCityObject"
/// }"#)?;
/// println!("{}", &co);
/// let co_json = serde_json::to_string(&co)?;
///
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
    "type: {}, geometry: {:?}, attributes: {:?}, geographical_extent: {:?}, children: {:?}, parents: {:?}",
    type_co,
    geometry,
    attributes,
    geographical_extent,
    children,
    parents
)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct CityObject<'cm> {
    #[serde(rename = "type")]
    pub type_co: CityObjectType,
    #[serde(borrow, skip_serializing_if = "Option::is_none")]
    pub geometry: Option<Vec<Geometry<'cm>>>,
    #[serde(
        borrow,
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_attributes",
        serialize_with = "serialize_attributes"
    )]
    #[cfg_attr(feature = "datasize", data_size(with = sizeof_attributes_option))]
    pub attributes: Option<Attributes<'cm>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "geographicalExtent")]
    pub geographical_extent: Option<BBox>,
    #[serde(borrow, skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Cow<'cm, str>>>,
    #[serde(borrow, skip_serializing_if = "Option::is_none")]
    pub parents: Option<Vec<Cow<'cm, str>>>,
    #[serde(
        borrow,
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_attributes",
        serialize_with = "serialize_attributes"
    )]
    #[cfg_attr(feature = "datasize", data_size(with = sizeof_attributes_option))]
    pub extra: Option<Attributes<'cm>>,
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
#[derive(Debug, Default, Display, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
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
///   },
///   "texture": {
///    "summer-textures": {
///       "values": [
///           [ [[0, 10, 23, 22, 21]], [[0, 1, 2, 6, 5]], [[null]], [[null]] ]
///       ]
///    }
///   }
/// }"#)?;
/// println!("{:?}", &geom);
/// let geom_json = serde_json::to_string(&geom)?;
/// # Ok(())
/// # }
/// ```
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
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
#[cfg_attr(feature = "datasize", derive(DataSize))]
#[serde(try_from = "IntermediateGeometry")]
pub struct Geometry<'cm> {
    #[serde(rename = "type")]
    pub type_: GeometryType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lod: Option<LoD>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boundaries: Option<Boundary>,
    #[serde(borrow, skip_serializing_if = "Option::is_none")]
    pub semantics: Option<Semantics<'cm>>,
    #[serde(borrow, skip_serializing_if = "Option::is_none")]
    pub material: Option<MaterialMap<'cm>>,
    #[serde(borrow, skip_serializing_if = "Option::is_none")]
    pub texture: Option<TextureMap<'cm>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<u16>,
    #[serde(rename = "boundaries", skip_serializing_if = "Option::is_none")]
    pub template_boundaries: Option<[usize; 1]>,
    #[serde(
        rename = "transformationMatrix",
        skip_serializing_if = "Option::is_none"
    )]
    pub template_transformation_matrix: Option<[f64; 16]>,
}

impl<'a: 'cm, 'cm> TryFrom<IntermediateGeometry<'a>> for Geometry<'cm> {
    type Error = serde_json::Error;

    fn try_from(geometry: IntermediateGeometry<'a>) -> result::Result<Self, Self::Error> {
        let mut lod: Option<LoD> = None;
        let mut boundaries: Option<Boundary> = None;
        let mut semantics: Option<Semantics> = None;
        let mut material: Option<MaterialMap> = None;
        let mut texture: Option<TextureMap> = None;
        let mut template: Option<u16> = None;
        let mut template_boundaries: Option<[usize; 1]> = None;
        let mut template_transformation_matrix: Option<[f64; 16]> = None;
        match geometry.type_ {
            GeometryType::MultiPoint => {
                lod = geometry.lod;
                // Would be neater with get_or_insert_default once it's stabilized https://doc.rust-lang.org/std/option/enum.Option.html#method.get_or_insert_default
                let boundaries_mut_ref = boundaries.get_or_insert_with(Boundary::default);
                if let Some(boundaries_raw) = geometry.boundaries {
                    boundaries_raw.deserialize_seq(ExtendVerticesVisitor(boundaries_mut_ref))?;
                }
                if let Some(intermediate_semantics) = geometry.semantics {
                    let values = labels::LabelIndex {
                        points: Deserialize::deserialize(
                            intermediate_semantics.values.into_deserializer(),
                        )?,
                        ..Default::default()
                    };
                    let _ = semantics.insert(Semantics {
                        surfaces: Deserialize::deserialize(
                            intermediate_semantics.surfaces.into_deserializer(),
                        )?,
                        values,
                    });
                }
            }
            GeometryType::MultiLineString => {
                lod = geometry.lod;
                let boundaries_mut_ref = boundaries.get_or_insert_with(Boundary::default);
                if let Some(boundaries_raw) = geometry.boundaries {
                    boundaries_raw.deserialize_seq(ExtendRingsVisitor(boundaries_mut_ref))?;
                }
                if let Some(intermediate_semantics) = geometry.semantics {
                    let values = labels::LabelIndex {
                        linestrings: Deserialize::deserialize(
                            intermediate_semantics.values.into_deserializer(),
                        )?,
                        ..Default::default()
                    };
                    let _ = semantics.insert(Semantics {
                        surfaces: Deserialize::deserialize(
                            intermediate_semantics.surfaces.into_deserializer(),
                        )?,
                        values,
                    });
                }
            }
            GeometryType::MultiSurface => {
                lod = geometry.lod;
                let boundaries_mut_ref = boundaries.get_or_insert_with(Boundary::default);
                if let Some(boundaries_raw) = geometry.boundaries {
                    boundaries_raw.deserialize_seq(ExtendSurfacesVisitor(boundaries_mut_ref))?;
                }
                if let Some(intermediate_semantics) = geometry.semantics {
                    let values = labels::LabelIndex {
                        surfaces: Deserialize::deserialize(
                            intermediate_semantics.values.into_deserializer(),
                        )?,
                        ..Default::default()
                    };
                    let _ = semantics.insert(Semantics {
                        surfaces: Deserialize::deserialize(
                            intermediate_semantics.surfaces.into_deserializer(),
                        )?,
                        values,
                    });
                }
                if let Some(intermediate_material) = geometry.material {
                    let mut materialindex = MaterialMap::with_capacity(intermediate_material.len());
                    for (k, v) in intermediate_material.into_iter() {
                        let mut materialvalues = MaterialValues::default();
                        if let Some(values_raw) = v.values {
                            let mut values = labels::LabelIndex::default();
                            values_raw.deserialize_seq(labels::ExtendLabelIndexSurfacesVisitor(
                                &mut values,
                            ))?;
                            materialvalues.values = Some(values);
                        } else {
                            materialvalues.value =
                                v.value.map(|v| GeometryIndex::try_from(v).unwrap());
                        }
                        let _ = materialindex.insert(k, materialvalues);
                    }
                    let _ = material.insert(materialindex);
                }
                if let Some(intermediate_texture) = geometry.texture {
                    let mut texturemap = TextureMap::with_capacity(intermediate_texture.len());
                    for (k, v) in intermediate_texture.into_iter() {
                        let mut texturevalues = TextureValues::default();
                        if let Some(values_raw) = v.values {
                            let mut values = labels::TextureIndex::default();
                            values_raw.deserialize_seq(
                                labels::ExtendTextureIndexSurfacesVisitor(&mut values),
                            )?;
                            texturevalues.values = Some(values);
                        }
                        let _ = texturemap.insert(k, texturevalues);
                    }
                    let _ = texture.insert(texturemap);
                }
            }
            GeometryType::CompositeSurface => {
                lod = geometry.lod;
                let boundaries_mut_ref = boundaries.get_or_insert_with(Boundary::default);
                if let Some(boundaries_raw) = geometry.boundaries {
                    boundaries_raw.deserialize_seq(ExtendSurfacesVisitor(boundaries_mut_ref))?;
                }
                if let Some(intermediate_semantics) = geometry.semantics {
                    let values = labels::LabelIndex {
                        surfaces: Deserialize::deserialize(
                            intermediate_semantics.values.into_deserializer(),
                        )?,
                        ..Default::default()
                    };
                    let _ = semantics.insert(Semantics {
                        surfaces: Deserialize::deserialize(
                            intermediate_semantics.surfaces.into_deserializer(),
                        )?,
                        values,
                    });
                }
                if let Some(intermediate_material) = geometry.material {
                    let mut materialmap = MaterialMap::with_capacity(intermediate_material.len());
                    for (k, v) in intermediate_material.into_iter() {
                        let mut materialvalues = MaterialValues::default();
                        if let Some(values_raw) = v.values {
                            let mut values = labels::LabelIndex::default();
                            values_raw.deserialize_seq(labels::ExtendLabelIndexSurfacesVisitor(
                                &mut values,
                            ))?;
                            materialvalues.values = Some(values);
                        } else {
                            materialvalues.value =
                                v.value.map(|v| GeometryIndex::try_from(v).unwrap());
                        }
                        let _ = materialmap.insert(k, materialvalues);
                    }
                    let _ = material.insert(materialmap);
                }
                if let Some(intermediate_texture) = geometry.texture {
                    let mut texturemap = TextureMap::with_capacity(intermediate_texture.len());
                    for (k, v) in intermediate_texture.into_iter() {
                        let mut texturevalues = TextureValues::default();
                        if let Some(values_raw) = v.values {
                            let mut values = labels::TextureIndex::default();
                            values_raw.deserialize_seq(
                                labels::ExtendTextureIndexSurfacesVisitor(&mut values),
                            )?;
                            texturevalues.values = Some(values);
                        }
                        let _ = texturemap.insert(k, texturevalues);
                    }
                    let _ = texture.insert(texturemap);
                }
            }
            GeometryType::Solid => {
                lod = geometry.lod;
                let boundaries_mut_ref = boundaries.get_or_insert_with(Boundary::default);
                if let Some(boundaries_raw) = geometry.boundaries {
                    boundaries_raw.deserialize_seq(ExtendShellsVisitor(boundaries_mut_ref))?;
                }
                if let Some(intermediate_semantics) = geometry.semantics {
                    let mut values = labels::LabelIndex::default();
                    intermediate_semantics
                        .values
                        .deserialize_seq(labels::ExtendLabelIndexShellsVisitor(&mut values))?;
                    let _ = semantics.insert(Semantics {
                        surfaces: Deserialize::deserialize(
                            intermediate_semantics.surfaces.into_deserializer(),
                        )?,
                        values,
                    });
                }
                if let Some(intermediate_material) = geometry.material {
                    let mut materialmap = MaterialMap::with_capacity(intermediate_material.len());
                    for (k, v) in intermediate_material.into_iter() {
                        let mut materialvalues = MaterialValues::default();
                        if let Some(values_raw) = v.values {
                            let mut values = labels::LabelIndex::default();
                            values_raw.deserialize_seq(labels::ExtendLabelIndexShellsVisitor(
                                &mut values,
                            ))?;
                            materialvalues.values = Some(values);
                        } else {
                            materialvalues.value =
                                v.value.map(|v| GeometryIndex::try_from(v).unwrap());
                        }
                        let _ = materialmap.insert(k, materialvalues);
                    }
                    let _ = material.insert(materialmap);
                }
                if let Some(intermediate_texture) = geometry.texture {
                    let mut texturemap = TextureMap::with_capacity(intermediate_texture.len());
                    for (k, v) in intermediate_texture.into_iter() {
                        let mut texturevalues = TextureValues::default();
                        if let Some(values_raw) = v.values {
                            let mut values = labels::TextureIndex::default();
                            values_raw.deserialize_seq(labels::ExtendTextureIndexShellsVisitor(
                                &mut values,
                            ))?;
                            texturevalues.values = Some(values);
                        }
                        let _ = texturemap.insert(k, texturevalues);
                    }
                    let _ = texture.insert(texturemap);
                }
            }
            GeometryType::MultiSolid => {
                lod = geometry.lod;
                let boundaries_mut_ref = boundaries.get_or_insert_with(Boundary::default);
                if let Some(boundaries_raw) = geometry.boundaries {
                    boundaries_raw.deserialize_seq(ExtendSolidsVisitor(boundaries_mut_ref))?;
                }
                if let Some(intermediate_semantics) = geometry.semantics {
                    let mut values = labels::LabelIndex::default();
                    intermediate_semantics
                        .values
                        .deserialize_seq(labels::ExtendLabelIndexSolidsVisitor(&mut values))?;
                    let _ = semantics.insert(Semantics {
                        surfaces: Deserialize::deserialize(
                            intermediate_semantics.surfaces.into_deserializer(),
                        )?,
                        values,
                    });
                }
                if let Some(intermediate_material) = geometry.material {
                    let mut materialmap = MaterialMap::with_capacity(intermediate_material.len());
                    for (k, v) in intermediate_material.into_iter() {
                        let mut materialvalues = MaterialValues::default();
                        if let Some(values_raw) = v.values {
                            let mut values = labels::LabelIndex::default();
                            values_raw.deserialize_seq(labels::ExtendLabelIndexSolidsVisitor(
                                &mut values,
                            ))?;
                            materialvalues.values = Some(values);
                        } else {
                            materialvalues.value =
                                v.value.map(|v| GeometryIndex::try_from(v).unwrap());
                        }
                        let _ = materialmap.insert(k, materialvalues);
                    }
                    let _ = material.insert(materialmap);
                }
                if let Some(intermediate_texture) = geometry.texture {
                    let mut texturemap = TextureMap::with_capacity(intermediate_texture.len());
                    for (k, v) in intermediate_texture.into_iter() {
                        let mut texturevalues = TextureValues::default();
                        if let Some(values_raw) = v.values {
                            let mut values = labels::TextureIndex::default();
                            values_raw.deserialize_seq(labels::ExtendTextureIndexSolidsVisitor(
                                &mut values,
                            ))?;
                            texturevalues.values = Some(values);
                        }
                        let _ = texturemap.insert(k, texturevalues);
                    }
                    let _ = texture.insert(texturemap);
                }
            }
            GeometryType::CompositeSolid => {
                lod = geometry.lod;
                let boundaries_mut_ref = boundaries.get_or_insert_with(Boundary::default);
                if let Some(boundaries_raw) = geometry.boundaries {
                    boundaries_raw.deserialize_seq(ExtendSolidsVisitor(boundaries_mut_ref))?;
                }
                if let Some(intermediate_semantics) = geometry.semantics {
                    let mut values = labels::LabelIndex::default();
                    intermediate_semantics
                        .values
                        .deserialize_seq(labels::ExtendLabelIndexSolidsVisitor(&mut values))?;
                    let _ = semantics.insert(Semantics {
                        surfaces: Deserialize::deserialize(
                            intermediate_semantics.surfaces.into_deserializer(),
                        )?,
                        values,
                    });
                }
                if let Some(intermediate_material) = geometry.material {
                    let mut materialmap = MaterialMap::with_capacity(intermediate_material.len());
                    for (k, v) in intermediate_material.into_iter() {
                        let mut materialvalues = MaterialValues::default();
                        if let Some(values_raw) = v.values {
                            let mut values = labels::LabelIndex::default();
                            values_raw.deserialize_seq(labels::ExtendLabelIndexSolidsVisitor(
                                &mut values,
                            ))?;
                            materialvalues.values = Some(values);
                        } else {
                            materialvalues.value =
                                v.value.map(|v| GeometryIndex::try_from(v).unwrap());
                        }
                        let _ = materialmap.insert(k, materialvalues);
                    }
                    let _ = material.insert(materialmap);
                }
                if let Some(intermediate_texture) = geometry.texture {
                    let mut texturemap = TextureMap::with_capacity(intermediate_texture.len());
                    for (k, v) in intermediate_texture.into_iter() {
                        let mut texturevalues = TextureValues::default();
                        if let Some(values_raw) = v.values {
                            let mut values = labels::TextureIndex::default();
                            values_raw.deserialize_seq(labels::ExtendTextureIndexSolidsVisitor(
                                &mut values,
                            ))?;
                            texturevalues.values = Some(values);
                        }
                        let _ = texturemap.insert(k, texturevalues);
                    }
                    let _ = texture.insert(texturemap);
                }
            }
            GeometryType::GeometryInstance => {
                template = geometry.template;
                if let Some(boundaries_raw) = geometry.boundaries {
                    template_boundaries = Some(Deserialize::deserialize(
                        boundaries_raw.into_deserializer(),
                    )?);
                }
                template_transformation_matrix = geometry.template_transformation_matrix;
            }
        }
        Ok(Geometry {
            type_: geometry.type_,
            lod,
            boundaries,
            semantics,
            material,
            texture,
            template,
            template_boundaries,
            template_transformation_matrix,
        })
    }
}

#[derive(Debug, Deserialize)]
struct IntermediateGeometry<'a> {
    #[serde(alias = "type")]
    type_: GeometryType,
    lod: Option<LoD>,
    #[serde(borrow)]
    boundaries: Option<&'a RawValue>,
    #[serde(borrow)]
    semantics: Option<IntermediateSemantics<'a>>,
    #[serde(borrow)]
    material: Option<IntermediateAppearance<'a>>,
    #[serde(borrow)]
    texture: Option<IntermediateAppearance<'a>>,
    template: Option<u16>,
    #[serde(rename = "transformationMatrix")]
    template_transformation_matrix: Option<[f64; 16]>,
}

#[derive(Debug, Display, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub enum GeometryType {
    MultiPoint,
    MultiLineString,
    MultiSurface,
    CompositeSurface,
    Solid,
    MultiSolid,
    CompositeSolid,
    GeometryInstance,
}

/// The Level of Detail of a Geometry.
///
/// The `LoD` forms an order, such as `LoD0 < LoD0_0 < LoD0_1 < LoD0_2 < LoD0_3 < LoD1 < ...`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

pub type MaterialMap<'cm> = Map<Cow<'cm, str>, MaterialValues>;
#[derive(Clone, Debug, Default, Display, PartialEq, Deserialize, Serialize)]
#[display("value: {:?}, values: {:?}", value, values)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct MaterialValues {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: OptionalGeometryIndex,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<labels::LabelIndex>,
}

pub type TextureMap<'cm> = Map<Cow<'cm, str>, TextureValues>;
#[derive(Clone, Debug, Default, Display, PartialEq, Deserialize, Serialize)]
#[display("values: {:?}", values)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct TextureValues {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<labels::TextureIndex>,
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
    "materials: {:?}, textures: {:?}, vertices-texture: {:?}, default-theme-texture: {:?}, default-theme-material: {:?}",
    materials,
    textures,
    vertices_texture,
    default_theme_texture,
    default_theme_material
)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct Appearance<'cm> {
    #[serde(borrow, skip_serializing_if = "Option::is_none")]
    pub materials: Option<Vec<Material<'cm>>>,
    #[serde(borrow, skip_serializing_if = "Option::is_none")]
    pub textures: Option<Vec<Texture<'cm>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertices_texture: Option<VerticesTexture>,
    #[serde(borrow, skip_serializing_if = "Option::is_none")]
    pub default_theme_texture: Option<Cow<'cm, str>>,
    #[serde(borrow, skip_serializing_if = "Option::is_none")]
    pub default_theme_material: Option<Cow<'cm, str>>,
}

type IntermediateAppearance<'a> = Map<Cow<'a, str>, IntermediateAppearanceValues<'a>>;
#[derive(Debug, Deserialize)]
struct IntermediateAppearanceValues<'a> {
    value: Option<usize>,
    #[serde(borrow)]
    values: Option<&'a RawValue>,
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
    "name: {}, ambient_intensity: {:?}, diffuse_color: {:?}, emissive_color: {:?}, specular_color: {:?}, shininess: {:?}, transparency: {:?}, is_smooth: {:?}",
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
pub struct Material<'cm> {
    #[serde(borrow)]
    pub name: Cow<'cm, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambient_intensity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diffuse_color: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emissive_color: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specular_color: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shininess: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transparency: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_smooth: Option<bool>,
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
    "type: {:?}, image: {:?}, wrap_mode: {:?}, texture_type: {:?}, border_color: {:?}",
    image_type,
    image,
    wrap_mode,
    texture_type,
    border_color
)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct Texture<'cm> {
    #[serde(rename = "type")]
    pub image_type: ImageType,
    #[serde(borrow)]
    pub image: Cow<'cm, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap_mode: Option<WrapMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture_type: Option<TextureType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_color: Option<[f32; 4]>,
}

/// Texture image type.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#texture-object>.
#[derive(Clone, Copy, Debug, Default, Display, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub enum ImageType {
    #[default]
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

/// Geometry Templates.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#geometry-templates>.
///
/// # Examples
/// ```
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let json_str = r#"{
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
/// }"#;
/// let geometry_templates: GeometryTemplates = serde_json::from_str(json_str)?;
/// let geometry_templates_json = serde_json::to_string(&geometry_templates)?;
/// assert_eq!(geometry_templates_json, json_str.replace(" ", "").replace("\n", ""));
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Default, Debug, Display, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[display(
    "templates: {:?}, vertices-templates: {:?}",
    templates,
    vertices_templates
)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct GeometryTemplates<'cm> {
    #[serde(borrow)]
    pub templates: Vec<Geometry<'cm>>,
    pub vertices_templates: VerticesTemplates,
}

/// The `vertices_templates` member of `geometry-templates` of CityJSON.
pub type VerticesTemplates = Vec<[f64; 3]>;

/// The `semantics` of a Geometry object.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#semantics-of-geometric-primitives>.
///
/// # Examples
/// ```rust
/// # use serde_cityjson::v1_1::*;
/// # fn main() -> serde_json::Result<()> {
/// let sem: Geometry = serde_json::from_str(r#"{
///     "type": "MultiLineString",
///     "lod": "0",
///     "boundaries": [[0, 1, 2, 3, 4]],
///     "semantics": {
///         "surfaces": [
///             {
///                 "type": "TransportationMarking"
///             }
///         ],
///         "values": [
///             0, 0, null, 0, 0
///         ]
///     }
/// }"#)?;
/// println!("{:?}", &sem);
/// let sem_json = serde_json::to_string(&sem)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Display, PartialEq, Eq, Deserialize, Serialize)]
#[display("surfaces: {:?}, values: {:?}", surfaces, values)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct Semantics<'cm> {
    #[serde(borrow)]
    pub surfaces: Vec<Semantic<'cm>>,
    pub values: labels::LabelIndex,
}

#[derive(Debug, Deserialize, Serialize)]
struct IntermediateSemantics<'a> {
    #[serde(borrow)]
    surfaces: &'a RawValue,
    #[serde(borrow)]
    values: &'a RawValue,
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
///
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
    "type: {:?}, children: {:?}, parent: {:?}, attributes: {:?}",
    type_sem,
    children,
    parent,
    attributes
)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct Semantic<'cm> {
    #[serde(rename = "type")]
    pub type_sem: SemanticType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<usize>,
    #[serde(
        borrow,
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_attributes",
        serialize_with = "serialize_attributes"
    )]
    #[cfg_attr(feature = "datasize", data_size(with = sizeof_attributes_option))]
    pub attributes: Option<Attributes<'cm>>,
}

/// Semantic surface type.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#semantics-of-geometric-primitives>.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
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
#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
#[serde(rename_all = "camelCase")]
pub struct Metadata<'cm> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geographical_extent: Option<BBox>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<CityModelIdentifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub point_of_contact: Option<Contact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_date: Option<Date>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_system: Option<CRS>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(
        borrow,
        flatten,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_attributes",
        serialize_with = "serialize_attributes"
    )]
    #[cfg_attr(feature = "datasize", data_size(with = sizeof_attributes_option))]
    pub extra: Option<Attributes<'cm>>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<ContactRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_type: Option<ContactType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(rename = "co-author")]
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
pub type Extensions = Map<String, Extension>;

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

#[allow(clippy::too_many_arguments)]
impl<'cm> CityModel<'cm> {
    pub fn new(
        id: Option<Cow<'cm, str>>,
        type_cm: Option<crate::CityModelType>,
        version: Option<crate::CityJSONVersion>,
        transform: Option<Transform>,
        cityobjects: Option<CityObjects<'cm>>,
        vertices: Option<Vertices>,
        metadata: Option<Metadata<'cm>>,
        appearance: Option<Appearance<'cm>>,
        geometry_templates: Option<GeometryTemplates<'cm>>,
        extra: Option<Attributes<'cm>>,
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
}

impl<'cm> Default for CityModel<'cm> {
    fn default() -> Self {
        Self {
            id: None,
            type_cm: crate::CityModelType::CityJSON,
            version: Some(crate::CityJSONVersion::V1_1),
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

impl<'cm> CityObject<'cm> {
    pub fn new(
        cotype: CityObjectType,
        geometry: Option<Vec<Geometry<'cm>>>,
        attributes: Option<Attributes<'cm>>,
        geographical_extent: Option<BBox>,
        children: Option<Vec<Cow<'cm, str>>>,
        parents: Option<Vec<Cow<'cm, str>>>,
        extra: Option<Attributes<'cm>>,
    ) -> Self {
        Self {
            type_co: cotype,
            geometry,
            attributes,
            geographical_extent,
            children,
            parents,
            extra,
        }
    }
}

impl<'de> Deserialize<'de> for CityObjectType {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
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

    fn visit_str<E>(self, value: &str) -> result::Result<Self::Value, E>
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
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
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
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
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

    fn visit_str<E>(self, value: &str) -> result::Result<Self::Value, E>
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
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
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

impl<'cm> Semantics<'cm> {
    pub fn new(surfaces: Vec<Semantic<'cm>>, values: labels::LabelIndex) -> Self {
        Self { surfaces, values }
    }
}

impl Display for SemanticType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'de: 'cm, 'cm> Deserialize<'de> for SemanticType {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
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

    fn visit_str<E>(self, value: &str) -> result::Result<Self::Value, E>
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
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
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

impl<'cm> Material<'cm> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'cm> Texture<'cm> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'cm> Metadata<'cm> {
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

impl<'cm> Display for Metadata<'cm> {
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
