use std::fmt::{Debug, Display};
use crate::cityjson::appearance::material::MaterialTrait;
use crate::cityjson::appearance::texture::TextureTrait;
use crate::cityjson::coordinate::{Coordinate, RealWorldCoordinate};
use crate::cityjson::geometry::semantic::{SemanticTrait, SemanticTypeTrait};
use crate::cityjson::geometry::GeometryTrait;
use crate::cityjson::metadata::MetadataTrait;
use crate::cityjson::vertex::{VertexIndex, VertexRef};
use crate::errors;
use crate::resources::pool::{ResourcePool, ResourceRef};
use crate::resources::storage::StringStorage;
use crate::v1_1::Metadata;

/// Bundles all the associated types for a CityJSON version implementation, specializing
/// the [GenericCityModel].
pub trait CityModelTypes {
    type CoordinateType: Coordinate;
    type VertexRef: VertexRef;
    type ResourceRef: ResourceRef;
    type StringStorage: StringStorage;
    type SemType: SemanticTypeTrait;

    type Semantic: SemanticTrait<Self::ResourceRef, Self::StringStorage, Self::SemType>;
    type Material: MaterialTrait<Self::StringStorage>;
    type Texture: TextureTrait<Self::StringStorage>;
    type Geometry: GeometryTrait<Self::VertexRef, Self::ResourceRef>;
    type Metadata: MetadataTrait<Self::StringStorage>;

    type GeometryPool: ResourcePool<Self::Geometry, Self::ResourceRef>;
    type SemanticPool: ResourcePool<Self::Semantic, Self::ResourceRef>;
    type MaterialPool: ResourcePool<Self::Material, Self::ResourceRef>;
    type TexturePool: ResourcePool<Self::Texture, Self::ResourceRef>;
}

pub trait CityModelTrait<V: CityModelTypes>: Debug + Debug + Clone {
    /// Create a new empty CityModel
    fn new() -> Self;
    /// Create a new CityModel with the specified capacity
    fn with_capacity(
        _vertex_capacity: usize,
        semantic_capacity: usize,
        material_capacity: usize,
        texture_capacity: usize,
        geometry_capacity: usize,
    ) -> Self;
    /// Add a semantic object to the pool
    fn add_semantic(&mut self, semantic: V::Semantic) -> V::ResourceRef;
    /// Get a reference to a semantic object
    fn get_semantic(&self, id: V::ResourceRef) -> Option<&V::Semantic>;
    /// Get a mutable reference to a semantic object
    fn get_semantic_mut(&mut self, id: V::ResourceRef) -> Option<&mut V::Semantic>;
    fn add_material(&mut self, material: V::Material) -> V::ResourceRef;
    fn get_material(&self, id: V::ResourceRef) -> Option<&V::Material>;
    fn get_material_mut(&mut self, id: V::ResourceRef) -> Option<&mut V::Material>;
    fn add_texture(&mut self, texture: V::Texture) -> V::ResourceRef;
    fn get_texture(&self, id: V::ResourceRef) -> Option<&V::Texture>;
    fn get_texture_mut(&mut self, id: V::ResourceRef) -> Option<&mut V::Texture>;
    /// Add a geometry to the model
    fn add_geometry(&mut self, geometry: V::Geometry) -> V::ResourceRef;
    fn geometries(&self) -> &V::GeometryPool;
    fn geometries_mut(&mut self) -> &mut V::GeometryPool;
    /// Add a vertex coordinate
    fn add_vertex(
        &mut self,
        coordinate: RealWorldCoordinate,
    ) -> errors::Result<VertexIndex<V::VertexRef>>;
    /// Get a reference to a vertex coordinate
    fn get_vertex(&self, index: VertexIndex<V::VertexRef>) -> Option<&RealWorldCoordinate>;
    /// Get the number of geometries
    fn geometry_count(&self) -> usize;
    /// Get the number of semantics
    fn semantic_count(&self) -> usize;
    /// Get the number of vertices
    fn vertex_count(&self) -> usize;
    fn metadata(&self) -> Option<&V::Metadata>;
    fn metadata_mut(&mut self) -> &mut V::Metadata;
}
