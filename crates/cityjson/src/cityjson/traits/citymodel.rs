use crate::cityjson::core::vertex::VertexIndex;
use crate::cityjson::core::vertex::VertexRef;
use crate::cityjson::traits::coordinate::Coordinate;
use crate::cityjson::traits::semantic::SemanticTypeTrait;
use crate::error::Result;
use crate::prelude::{Attributes, RealWorldCoordinate, UVCoordinate, Vertices};
use crate::resources::pool::{ResourcePool, ResourceRef};
use crate::resources::storage::StringStorage;
use crate::{CityJSONVersion, CityModelType};
use std::fmt::{Debug, Display};

/// Bundles all the associated types for a CityJSON version implementation, specializing
/// the `CityModel`.
pub trait CityModelTypes {
    type CoordinateType: Coordinate;
    type VertexRef: VertexRef;
    type ResourceRef: ResourceRef;
    type StringStorage: StringStorage;
    type SemType: SemanticTypeTrait;

    type Semantic;
    type Material;
    type Texture;
    type Geometry: crate::cityjson::core::geometry::GeometryConstructor<
            Self::VertexRef,
            Self::ResourceRef,
            <Self::StringStorage as StringStorage>::String,
        >;
    type Metadata;
    type Transform;
    type Extension;
    type Extensions;
    type CityObjectType: Default + Display + Clone;
    type BBox;
    type CityObject;

    type CityObjects;
    type GeometryPool: ResourcePool<Self::Geometry, Self::ResourceRef>;
    type SemanticPool: ResourcePool<Self::Semantic, Self::ResourceRef>;
    type MaterialPool: ResourcePool<Self::Material, Self::ResourceRef>;
    type TexturePool: ResourcePool<Self::Texture, Self::ResourceRef>;
}

pub trait CityModelTrait2<V: CityModelTypes>: Debug + Clone {
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
    /// Get the type of CityModel
    fn type_citymodel(&self) -> CityModelType;
    /// Get the CityJSON version that this CityModel represents
    fn version(&self) -> Option<CityJSONVersion>;
    /// Gets an existing semantic or adds a new one.
    ///
    /// # Arguments
    /// * `semantic` - The semantic to get or add
    ///
    /// # Returns
    /// The resource ID of the existing or newly added semantic
    fn get_or_insert_semantic(&mut self, semantic: V::Semantic) -> V::ResourceRef
    where
        V::Semantic: PartialEq;
    /// Add a semantic object to the pool
    fn add_semantic(&mut self, semantic: V::Semantic) -> V::ResourceRef;
    /// Get a reference to a semantic object
    fn get_semantic(&self, id: V::ResourceRef) -> Option<&V::Semantic>;
    /// Get a mutable reference to a semantic object
    fn get_semantic_mut(&mut self, id: V::ResourceRef) -> Option<&mut V::Semantic>;
    fn semantics(&self) -> &V::SemanticPool;
    fn textures(&self) -> &V::TexturePool;
    /// Gets an existing material or adds a new one.
    ///
    /// # Arguments
    /// * `material` - The material to get or add
    ///
    /// # Returns
    /// The resource ID of the existing or newly added material
    fn get_or_insert_material(&mut self, material: V::Material) -> V::ResourceRef
    where
        V::Material: PartialEq;
    fn add_material(&mut self, material: V::Material) -> V::ResourceRef;
    fn get_material(&self, id: V::ResourceRef) -> Option<&V::Material>;
    fn get_material_mut(&mut self, id: V::ResourceRef) -> Option<&mut V::Material>;
    fn materials(&self) -> &V::MaterialPool;
    /// Gets an existing texture or adds a new one.
    ///
    /// # Arguments
    /// * `texture` - The material to get or add
    ///
    /// # Returns
    /// The resource ID of the existing or newly added material
    fn get_or_insert_texture(&mut self, texture: V::Texture) -> V::ResourceRef
    where
        V::Texture: PartialEq;
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
    /// Clears geometries from the model while preserving pool capacity.
    /// Shared resources (semantics, materials) are not removed.
    fn clear_geometries(&mut self);
    /// Add a vertex coordinate
    fn add_vertex(&mut self, coordinate: V::CoordinateType) -> Result<VertexIndex<V::VertexRef>>;
    /// Get a reference to a vertex coordinate
    fn get_vertex(&self, index: VertexIndex<V::VertexRef>) -> Option<&V::CoordinateType>;
    /// Get a reference to the vertices pool
    fn vertices(&self) -> &Vertices<V::VertexRef, V::CoordinateType>;
    /// Get a mutable reference to the vertices pool
    fn vertices_mut(&mut self) -> &mut Vertices<V::VertexRef, V::CoordinateType>;
    fn clear_vertices(&mut self);
    /// Add a vertex coordinate of a geometry template
    fn add_template_vertex(
        &mut self,
        coordinate: RealWorldCoordinate,
    ) -> Result<VertexIndex<V::VertexRef>>;
    fn get_template_vertex(&self, index: VertexIndex<V::VertexRef>)
    -> Option<&RealWorldCoordinate>;
    fn template_vertices(&self) -> &Vertices<V::VertexRef, RealWorldCoordinate>;
    fn template_vertices_mut(&mut self) -> &mut Vertices<V::VertexRef, RealWorldCoordinate>;
    fn clear_template_vertices(&mut self);
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
    /// Clears CityObjects from the model while preserving pool capacity.
    fn clear_cityobjects(&mut self);
    /// Get the default theme material reference
    fn default_theme_material(&self) -> Option<V::ResourceRef>;
    /// Set the default theme material reference
    fn set_default_theme_material(&mut self, material_ref: Option<V::ResourceRef>);
    /// Get the default theme texture reference
    fn default_theme_texture(&self) -> Option<V::ResourceRef>;
    /// Set the default theme texture reference
    fn set_default_theme_texture(&mut self, texture_ref: Option<V::ResourceRef>);
}
