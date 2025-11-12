use crate::cityjson::core::boundary::Boundary;
use crate::cityjson::core::geometry::{GeometryType, LoD};
use crate::cityjson::core::vertex::VertexRef;
use crate::cityjson::traits::geometry::GeometryTrait;
use crate::prelude::{StringStorage, VertexIndex};
use crate::resources::mapping::{MaterialMap, SemanticMap, TextureMap};
use crate::resources::pool::ResourceRef;

// Type aliases to simplify complex type signatures
type ThemedMaterials<VR, RR, SS> = Vec<(SS, MaterialMap<VR, RR>)>;
type ThemedTextures<VR, RR, SS> = Vec<(SS, TextureMap<VR, RR>)>;

pub mod semantic;

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Geometry<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    type_geometry: GeometryType,
    lod: Option<LoD>,
    boundaries: Option<Boundary<VR>>,
    semantics: Option<SemanticMap<VR, RR>>,
    materials: Option<ThemedMaterials<VR, RR, SS::String>>,
    textures: Option<ThemedTextures<VR, RR, SS::String>>,
    instance_template: Option<RR>,
    instance_reference_point: Option<VertexIndex<VR>>,
    instance_transformation_matrix: Option<[f64; 16]>,
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> GeometryTrait<VR, RR, SS>
    for Geometry<VR, RR, SS>
where
    VR: VertexRef,
    RR: ResourceRef,
{
    fn new(
        type_geometry: GeometryType,
        lod: Option<LoD>,
        boundaries: Option<Boundary<VR>>,
        semantics: Option<SemanticMap<VR, RR>>,
        materials: Option<Vec<(SS::String, MaterialMap<VR, RR>)>>,
        textures: Option<Vec<(SS::String, TextureMap<VR, RR>)>>,
        instance_template: Option<RR>,
        instance_reference_point: Option<VertexIndex<VR>>,
        instance_transformation_matrix: Option<[f64; 16]>,
    ) -> Self {
        Self {
            type_geometry,
            lod,
            boundaries,
            semantics,
            materials,
            textures,
            instance_template,
            instance_reference_point,
            instance_transformation_matrix,
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

    fn materials(&self) -> Option<&ThemedMaterials<VR, RR, SS::String>> {
        self.materials.as_ref()
    }

    fn textures(&self) -> Option<&ThemedTextures<VR, RR, SS::String>> {
        self.textures.as_ref()
    }

    fn instance_template(&self) -> Option<&RR> {
        self.instance_template.as_ref()
    }

    fn instance_reference_point(&self) -> Option<&VertexIndex<VR>> {
        self.instance_reference_point.as_ref()
    }

    fn instance_transformation_matrix(&self) -> Option<&[f64; 16]> {
        self.instance_transformation_matrix.as_ref()
    }
}
