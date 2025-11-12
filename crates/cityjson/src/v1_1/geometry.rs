//! # Geometry
//!
//! Represents a [Geometry object](https://www.cityjson.org/specs/1.1.3/#geometry-objects).
use crate::cityjson::core::geometry_struct::GeometryCore;
use crate::cityjson::core::vertex::VertexRef;
use crate::cityjson::traits::geometry::GeometryTrait;
use crate::prelude::StringStorage;
use crate::resources::pool::ResourceRef;

pub mod semantic;

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Geometry<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    inner: GeometryCore<VR, RR, SS>,
}

crate::macros::impl_geometry_methods!();

// Trait implementation for internal use (required by CityModelTypes)
impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> GeometryTrait<VR, RR, SS>
    for Geometry<VR, RR, SS>
{
    fn new(
        type_geometry: crate::cityjson::core::geometry::GeometryType,
        lod: Option<crate::cityjson::core::geometry::LoD>,
        boundaries: Option<crate::cityjson::core::boundary::Boundary<VR>>,
        semantics: Option<crate::resources::mapping::SemanticMap<VR, RR>>,
        materials: Option<Vec<(SS::String, crate::resources::mapping::MaterialMap<VR, RR>)>>,
        textures: Option<Vec<(SS::String, crate::resources::mapping::TextureMap<VR, RR>)>>,
        instance_template: Option<RR>,
        instance_reference_point: Option<crate::cityjson::core::vertex::VertexIndex<VR>>,
        instance_transformation_matrix: Option<[f64; 16]>,
    ) -> Self {
        Self::new(
            type_geometry,
            lod,
            boundaries,
            semantics,
            materials,
            textures,
            instance_template,
            instance_reference_point,
            instance_transformation_matrix,
        )
    }

    fn type_geometry(&self) -> &crate::cityjson::core::geometry::GeometryType {
        self.type_geometry()
    }

    fn lod(&self) -> Option<&crate::cityjson::core::geometry::LoD> {
        self.lod()
    }

    fn boundaries(&self) -> Option<&crate::cityjson::core::boundary::Boundary<VR>> {
        self.boundaries()
    }

    fn semantics(&self) -> Option<&crate::resources::mapping::SemanticMap<VR, RR>> {
        self.semantics()
    }

    fn materials(
        &self,
    ) -> Option<&Vec<(SS::String, crate::resources::mapping::MaterialMap<VR, RR>)>> {
        self.materials()
    }

    fn textures(
        &self,
    ) -> Option<&Vec<(SS::String, crate::resources::mapping::TextureMap<VR, RR>)>> {
        self.textures()
    }

    fn instance_template(&self) -> Option<&RR> {
        self.instance_template()
    }

    fn instance_reference_point(&self) -> Option<&crate::cityjson::core::vertex::VertexIndex<VR>> {
        self.instance_reference_point()
    }

    fn instance_transformation_matrix(&self) -> Option<&[f64; 16]> {
        self.instance_transformation_matrix()
    }
}
