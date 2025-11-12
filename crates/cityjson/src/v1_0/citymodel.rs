use crate::prelude::*;
use crate::v1_0::appearance::material::Material;
use crate::v1_0::appearance::texture::Texture;
use crate::v1_0::geometry::Geometry;
use crate::v1_0::geometry::semantic::{Semantic, SemanticType};
use crate::v1_0::metadata::Metadata;
use crate::v1_0::{CityObject, CityObjectType, CityObjects, Extension, Extensions, Transform};
use crate::{CityJSONVersion, format_option};
use std::fmt;
use std::marker::PhantomData;

pub struct V1_0<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    _phantom_vr: PhantomData<VR>,
    _phantom_rr: PhantomData<RR>,
    _phantom_ss: PhantomData<SS>,
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> CityModelTypes for V1_0<VR, RR, SS> {
    type CoordinateType = FlexibleCoordinate;
    type VertexRef = VR;
    type ResourceRef = RR;
    type StringStorage = SS;
    type SemType = SemanticType<SS>;
    type Semantic = Semantic<RR, SS>;
    type Material = Material<SS>;
    type Texture = Texture<SS>;
    type Geometry = Geometry<VR, RR, SS>;
    type Metadata = Metadata<SS, RR>;
    type Transform = Transform;
    type Extension = Extension<SS>;
    type Extensions = Extensions<SS>;
    type CityObjectType = CityObjectType<SS>;
    type BBox = BBox;
    type CityObject = CityObject<SS, RR>;
    type CityObjects = CityObjects<SS, RR>;
    type GeometryPool = DefaultResourcePool<Geometry<VR, RR, SS>, RR>;
    type SemanticPool = DefaultResourcePool<Semantic<RR, SS>, RR>;
    type MaterialPool = DefaultResourcePool<Material<SS>, RR>;
    type TexturePool = DefaultResourcePool<Texture<SS>, RR>;
}

#[derive(Debug, Clone)]
pub struct CityModel<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    #[allow(clippy::type_complexity)]
    inner: crate::cityjson::core::citymodel::CityModelCore<
        FlexibleCoordinate,
        VR,
        RR,
        SS,
        Semantic<RR, SS>,
        Material<SS>,
        Texture<SS>,
        Geometry<VR, RR, SS>,
        Metadata<SS, RR>, // v1_0 uses SS, RR order
        Transform,
        Extensions<SS>,
        CityObjects<SS, RR>,
    >,
}

crate::macros::impl_citymodel_methods!(FlexibleCoordinate, CityJSONVersion::V1_0, Metadata<SS, RR>);

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> fmt::Display for CityModel<VR, RR, SS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "CityModel {{")?;
        writeln!(f, "\ttype: {}", self.type_citymodel())?;
        writeln!(f, "\tversion: {}", format_option(&self.version()))?;
        writeln!(
            f,
            "\textensions: {{ {} }}",
            format_option(&self.extensions())
        )?;
        writeln!(f, "\ttransform: {{ {} }}", format_option(&self.transform()))?;
        writeln!(f, "\tmetadata: {}", format_option(&self.metadata()))?;
        writeln!(
            f,
            "\tCityObjects: {{ nr. cityobjects: {}, nr. geometries: {} }}",
            self.cityobjects().len(),
            self.geometries().len()
        )?;
        writeln!(
            f,
            "\tappearance: {{ nr. materials: {}, nr. textures: {}, nr. vertices-texture: {}, default-theme-texture: {}, default-theme-material: {} }}",
            self.materials().len(),
            self.textures().len(),
            self.uv_coordinate_count(),
            format_option(&self.default_theme_texture()),
            format_option(&self.default_theme_material())
        )?;
        writeln!(f, "\tgeometry-templates: not implemented")?;
        writeln!(
            f,
            "\tvertices: {{ nr. vertices: {}, quantized coordinates: not implemented }}",
            self.vertices().len()
        )?;
        writeln!(f, "\textra: {}", format_option(&self.extra()))?;
        writeln!(f, "}}")
    }
}
