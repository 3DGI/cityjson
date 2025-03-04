//! # Geometry
//!
//! Represents a [Geometry object](https://www.cityjson.org/specs/1.1.3/#geometry-objects).
use crate::cityjson::geometry::boundary::Boundary;
use crate::cityjson::geometry::{GeometryTrait, GeometryType, LoD};
use crate::cityjson::vertex::VertexRef;
use crate::resources::mapping::{MaterialMap, SemanticMap, TextureMap};
use crate::resources::pool::{ResourceRef};

pub mod semantic;

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Geometry<VR: VertexRef, RR: ResourceRef> {
    type_geometry: GeometryType,
    lod: Option<LoD>,
    boundaries: Option<Boundary<VR>>,
    semantics: Option<SemanticMap<VR, RR>>,
    material: Option<MaterialMap<VR, RR>>,
    texture: Option<TextureMap<VR, RR>>,
    template_boundaries: Option<usize>,
    template_transformation_matrix: Option<[f64; 16]>,
}

impl<VR, RR> GeometryTrait<VR, RR> for Geometry<VR, RR>
where
    VR: VertexRef,
    RR: ResourceRef,
{
    fn new(
        type_geometry: GeometryType,
        lod: Option<LoD>,
        boundaries: Option<Boundary<VR>>,
        semantics: Option<SemanticMap<VR, RR>>,
        material: Option<MaterialMap<VR, RR>>,
        texture: Option<TextureMap<VR, RR>>,
        template_boundaries: Option<usize>,
        template_transformation_matrix: Option<[f64; 16]>,
    ) -> Self {
        Self {
            type_geometry,
            lod,
            boundaries,
            semantics,
            material,
            texture,
            template_boundaries,
            template_transformation_matrix,
        }
    }

    fn type_geometry(&self) -> &GeometryType {
        &self.type_geometry
    }

    fn lod(&self) -> Option<&LoD> {
        self.lod.as_ref()
    }

    fn boundaries(&self) -> Option<&Boundary<VR>> {
        self.boundaries.as_ref()
    }

    fn semantics(&self) -> Option<&SemanticMap<VR, RR>> {
        self.semantics.as_ref()
    }

    fn materials(&self) -> Option<&MaterialMap<VR, RR>> {
        self.material.as_ref()
    }

    fn textures(&self) -> Option<&TextureMap<VR, RR>> {
        self.texture.as_ref()
    }

    fn template_boundaries(&self) -> Option<&usize> {
        self.template_boundaries.as_ref()
    }

    fn template_transformation_matrix(&self) -> Option<&[f64; 16]> {
        self.template_transformation_matrix.as_ref()
    }
}
