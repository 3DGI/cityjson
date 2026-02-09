//! Core Geometry structure shared across `CityJSON` versions

use crate::cityjson::core::boundary::Boundary;
use crate::cityjson::core::geometry::{GeometryType, LoD};
use crate::cityjson::core::vertex::VertexRef;
use crate::prelude::{StringStorage, VertexIndex};
use crate::resources::mapping::textures::TextureMapCore;
use crate::resources::mapping::SemanticOrMaterialMap;
use crate::resources::pool::ResourceRef;

// Type aliases to simplify complex type signatures
type ThemedMaterials<VR, RR, SS> = Vec<(SS, SemanticOrMaterialMap<VR, RR>)>;
type ThemedTextures<VR, RR, SS> = Vec<(SS, TextureMapCore<VR, RR>)>;

/// Core geometry structure that contains the data for all `CityJSON` versions.
/// Version-specific types wrap this core structure and implement methods via macros.
#[derive(Clone, Debug)]
pub struct GeometryCore<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    type_geometry: GeometryType,
    lod: Option<LoD>,
    boundaries: Option<Boundary<VR>>,
    semantics: Option<SemanticOrMaterialMap<VR, RR>>,
    materials: Option<ThemedMaterials<VR, RR, SS::String>>,
    textures: Option<ThemedTextures<VR, RR, SS::String>>,
    instance_template: Option<RR>,
    instance_reference_point: Option<VertexIndex<VR>>,
    instance_transformation_matrix: Option<[f64; 16]>,
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> GeometryCore<VR, RR, SS> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        type_geometry: GeometryType,
        lod: Option<LoD>,
        boundaries: Option<Boundary<VR>>,
        semantics: Option<SemanticOrMaterialMap<VR, RR>>,
        materials: Option<ThemedMaterials<VR, RR, SS::String>>,
        textures: Option<ThemedTextures<VR, RR, SS::String>>,
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

    pub fn type_geometry(&self) -> &GeometryType {
        &self.type_geometry
    }

    pub fn lod(&self) -> Option<&LoD> {
        self.lod.as_ref()
    }

    pub fn boundaries(&self) -> Option<&Boundary<VR>> {
        self.boundaries.as_ref()
    }

    pub(crate) fn semantics(&self) -> Option<&SemanticOrMaterialMap<VR, RR>> {
        self.semantics.as_ref()
    }

    pub(crate) fn materials(&self) -> Option<&ThemedMaterials<VR, RR, SS::String>> {
        self.materials.as_ref()
    }

    pub(crate) fn textures(&self) -> Option<&ThemedTextures<VR, RR, SS::String>> {
        self.textures.as_ref()
    }

    pub fn instance_template(&self) -> Option<&RR> {
        self.instance_template.as_ref()
    }

    pub fn instance_reference_point(&self) -> Option<&VertexIndex<VR>> {
        self.instance_reference_point.as_ref()
    }

    pub fn instance_transformation_matrix(&self) -> Option<&[f64; 16]> {
        self.instance_transformation_matrix.as_ref()
    }
}
