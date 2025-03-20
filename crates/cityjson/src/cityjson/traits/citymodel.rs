use crate::cityjson::shared::vertex::VertexIndex;
use crate::cityjson::traits::appearance::material::MaterialTrait;
use crate::cityjson::traits::appearance::texture::TextureTrait;
use crate::cityjson::traits::cityobject::CityObjectsTrait;
use crate::cityjson::traits::coordinate::Coordinate;
use crate::cityjson::traits::geometry::GeometryTrait;
use crate::cityjson::traits::metadata::MetadataTrait;
use crate::cityjson::traits::semantic::{SemanticTrait, SemanticTypeTrait};
use crate::cityjson::traits::vertex::VertexRef;
use crate::errors::Result;
use crate::prelude::{
    Attributes, BBoxTrait, CityObjectTrait, CityObjectTypeTrait, ExtensionTrait, ExtensionsTrait,
    RealWorldCoordinate, TransformTrait, UVCoordinate, Vertices,
};
use crate::resources::pool::{ResourcePool, ResourceRef};
use crate::resources::storage::StringStorage;
use crate::CityModelType;
use std::fmt::Debug;

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
    type Geometry: GeometryTrait<Self::VertexRef, Self::ResourceRef, Self::StringStorage>;
    type Metadata: MetadataTrait<Self::StringStorage>;
    type Transform: TransformTrait;
    type Extension: ExtensionTrait<Self::StringStorage>;
    type Extensions: ExtensionsTrait<Self::StringStorage, Self::Extension>;
    type CityObjectType: CityObjectTypeTrait<Self::StringStorage>;
    type BBox: BBoxTrait;
    type CityObject: CityObjectTrait<
        Self::StringStorage,
        Self::ResourceRef,
        Self::CityObjectType,
        Self::BBox,
    >;

    type CityObjects: CityObjectsTrait<
        Self::StringStorage,
        Self::ResourceRef,
        Self::CityObject,
        Self::CityObjectType,
        Self::BBox,
    >;
    type GeometryPool: ResourcePool<Self::Geometry, Self::ResourceRef>;
    type SemanticPool: ResourcePool<Self::Semantic, Self::ResourceRef>;
    type MaterialPool: ResourcePool<Self::Material, Self::ResourceRef>;
    type TexturePool: ResourcePool<Self::Texture, Self::ResourceRef>;
}

pub trait CityModelTrait<V: CityModelTypes>: Debug + Debug + Clone {
    /// Create a new empty CityModel
    fn new(type_citymodel: CityModelType) -> Self;
    /// Create a new CityModel with the specified capacity
    fn with_capacity(
        type_citymodel: CityModelType,
        cityobjects_capacity: usize,
        vertex_capacity: usize,
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
    fn add_uv_coordinate(
        &mut self,
        uvcoordinate: UVCoordinate,
    ) -> Result<VertexIndex<V::VertexRef>>;
    fn get_uv_coordinate(&self, index: VertexIndex<V::VertexRef>) -> Option<&UVCoordinate>;
    /// Add a geometry to the model
    fn add_geometry(&mut self, geometry: V::Geometry) -> V::ResourceRef;
    fn geometries(&self) -> &V::GeometryPool;
    fn geometries_mut(&mut self) -> &mut V::GeometryPool;
    /// Add a vertex coordinate
    fn add_vertex(&mut self, coordinate: V::CoordinateType) -> Result<VertexIndex<V::VertexRef>>;
    /// Get a reference to a vertex coordinate
    fn get_vertex(&self, index: VertexIndex<V::VertexRef>) -> Option<&V::CoordinateType>;
    /// Get a reference to the vertices pool
    fn vertices(&self) -> &Vertices<V::VertexRef, V::CoordinateType>;
    /// Get a mutable reference to the vertices pool
    fn vertices_mut(&mut self) -> &mut Vertices<V::VertexRef, V::CoordinateType>;
    /// Add a vertex coordinate of a geometry template
    fn add_template_vertex(
        &mut self,
        coordinate: RealWorldCoordinate,
    ) -> Result<VertexIndex<V::VertexRef>>;
    fn get_template_vertex(&self, index: VertexIndex<V::VertexRef>)
        -> Option<&RealWorldCoordinate>;
    fn template_vertices(&self) -> &Vertices<V::VertexRef, RealWorldCoordinate>;
    fn template_vertices_mut(&mut self) -> &mut Vertices<V::VertexRef, RealWorldCoordinate>;
    /// Add a geometry template
    fn add_template_geometry(&mut self, geometry: V::Geometry) -> V::ResourceRef;
    fn template_geometries(&self) -> &V::GeometryPool;
    fn template_geometries_mut(&mut self) -> &mut V::GeometryPool;
    /// Get the number of geometries
    fn geometry_count(&self) -> usize;
    /// Get the number of semantics
    fn semantic_count(&self) -> usize;
    /// Get the number of vertices
    fn vertex_count(&self) -> usize;
    fn metadata(&self) -> Option<&V::Metadata>;
    fn metadata_mut(&mut self) -> &mut V::Metadata;
    fn extra(&self) -> Option<&Attributes<V::StringStorage, V::ResourceRef>>;
    fn extra_mut(&mut self) -> &mut Attributes<V::StringStorage, V::ResourceRef>;
    fn transform(&self) -> Option<&V::Transform>;
    fn transform_mut(&mut self) -> &mut V::Transform;
    fn extensions(&self) -> Option<&V::Extensions>;
    fn extensions_mut(&mut self) -> &mut V::Extensions;
    fn cityobjects(&self) -> &V::CityObjects;
    fn cityobjects_mut(&mut self) -> &mut V::CityObjects;
}
