//! # CityModel
//!
//! Represents a [CityJSON object](https://www.cityjson.org/specs/1.1.3/#cityjson-object).

use crate::cityjson::citymodel::{CityModelVersion, GenericCityModel};
use crate::cityjson::coordinate::RealWorldCoordinate;
use crate::cityjson::vertex::VertexRef;
use crate::resources::pool::{DefaultResourcePool, ResourceRef};
use crate::resources::storage::StringStorage;
use crate::v1_1::appearance::material::Material;
use crate::v1_1::appearance::texture::Texture;
use crate::v1_1::geometry::semantic::Semantic;
use crate::v1_1::geometry::Geometry;
use std::marker::PhantomData;
use crate::v1_1::metadata::Metadata;

struct CityModelVersion11<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    _phantom_vr: PhantomData<VR>,
    _phantom_rr: PhantomData<RR>,
    _phantom_ss: PhantomData<SS>,
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> CityModelVersion
    for CityModelVersion11<VR, RR, SS>
{
    type CoordinateType = RealWorldCoordinate;
    type VertexRef = VR;
    type ResourceRef = RR;
    type StringStorage = SS;
    type Semantic = Semantic<RR, SS>;
    type Material = Material<SS>;
    type Texture = Texture<SS>;
    type Geometry = Geometry<VR, RR>;
    type Metadata = Metadata<SS>;
    type GeometryPool = DefaultResourcePool<Geometry<VR, RR>, RR>;
    type SemanticPool = DefaultResourcePool<Semantic<RR, SS>, RR>;
    type MaterialPool = DefaultResourcePool<Material<SS>, RR>;
    type TexturePool = DefaultResourcePool<Texture<SS>, RR>;
}

pub type CityModel<VR, RR, SS> = GenericCityModel<CityModelVersion11<VR, RR, SS>>;
