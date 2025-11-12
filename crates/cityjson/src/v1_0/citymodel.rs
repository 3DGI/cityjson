use crate::prelude::*;
use crate::resources::pool::ResourcePool;
use crate::v1_0::appearance::material::Material;
use crate::v1_0::appearance::texture::Texture;
use crate::v1_0::geometry::Geometry;
use crate::v1_0::geometry::semantic::Semantic;
use crate::v1_0::metadata::Metadata;
use crate::v1_0::{CityObjects, Extensions, Transform};
use crate::{CityJSONVersion, format_option};
use std::fmt;

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
            self.vertices_texture().len(),
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
