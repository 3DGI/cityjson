use crate::cityjson::core::vertex::VertexRef;
use crate::prelude::{
    Boundary, GeometryType, LoD, MaterialMap, ResourceRef, SemanticMap, StringStorage, TextureMap,
    VertexIndex,
};

// Type aliases to simplify complex type signatures
type ThemedMaterials<VR, RR, SS> = Vec<(SS, MaterialMap<VR, RR>)>;
type ThemedTextures<VR, RR, SS> = Vec<(SS, TextureMap<VR, RR>)>;

pub trait GeometryTrait<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    /// Create a new geometry given the parts
    #[allow(clippy::too_many_arguments)]
    fn new(
        type_geometry: GeometryType,
        lod: Option<LoD>,
        boundaries: Option<Boundary<VR>>,
        semantics: Option<SemanticMap<VR, RR>>,
        materials: Option<ThemedMaterials<VR, RR, SS::String>>,
        textures: Option<ThemedTextures<VR, RR, SS::String>>,
        instance_template: Option<RR>,
        instance_reference_point: Option<VertexIndex<VR>>,
        instance_transformation_matrix: Option<[f64; 16]>,
    ) -> Self;

    /// Returns the geometry type
    fn type_geometry(&self) -> &GeometryType;

    /// Returns the level of detail
    fn lod(&self) -> Option<&LoD>;

    /// Returns the geometry boundaries
    fn boundaries(&self) -> Option<&Boundary<VR>>;

    /// Returns the semantic mapping
    fn semantics(&self) -> Option<&SemanticMap<VR, RR>>;

    /// Returns the material mapping
    fn materials(&self) -> Option<&ThemedMaterials<VR, RR, SS::String>>;

    /// Returns the texture mapping
    fn textures(&self) -> Option<&ThemedTextures<VR, RR, SS::String>>;

    /// Returns the template of the GeometryInstance
    fn instance_template(&self) -> Option<&RR>;

    /// Returns the reference point of the GeometryInstance
    fn instance_reference_point(&self) -> Option<&VertexIndex<VR>>;

    /// Returns the transformation matrix of the GeometryInstance
    fn instance_transformation_matrix(&self) -> Option<&[f64; 16]>;
}
