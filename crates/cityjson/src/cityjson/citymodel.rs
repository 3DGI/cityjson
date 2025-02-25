use crate::cityjson::appearance::material::Material;
use crate::cityjson::appearance::texture::Texture;
use crate::cityjson::attributes::Attributes;
use crate::cityjson::coordinate::{Coordinate, RealWorldCoordinate, UVCoordinate, Vertices};
use crate::cityjson::geometry::semantic::Semantic;
use crate::cityjson::geometry::GeometryTrait;
use crate::cityjson::metadata::Metadata;
use crate::cityjson::vertex::{VertexIndex, VertexRef};
use crate::errors;
use crate::resources::pool::{ResourcePool, ResourceRef};
use crate::resources::storage::StringStorage;

/// Bundles all the associated types for a CityJSON version implementation, specializing
/// the [GenericCityModel].
pub trait CityModelVersion {
    type CoordinateType: Coordinate;
    type VertexRef: VertexRef;
    type ResourceRef: ResourceRef;
    type StringStorage: StringStorage;

    type Semantic: Semantic<Self::ResourceRef, Self::StringStorage>;
    type Material: Material<Self::StringStorage>;
    type Texture: Texture<Self::StringStorage>;
    type Geometry: GeometryTrait<Self::VertexRef, Self::ResourceRef, Self::StringStorage>;
    type Metadata: Metadata<Self::StringStorage>;

    type SemanticPool: ResourcePool<Self::Semantic, Self::ResourceRef>;
    type MaterialPool: ResourcePool<Self::Material, Self::ResourceRef>;
    type TexturePool: ResourcePool<Self::Texture, Self::ResourceRef>;
}

#[derive(Debug)]
pub struct GenericCityModel<V: CityModelVersion> {
    /// Pool of vertex coordinates
    vertices: Vertices<V::VertexRef, RealWorldCoordinate>,
    /// Pool of semantic objects
    semantics: V::SemanticPool,
    /// Pool of material objects
    materials: V::MaterialPool,
    /// Pool of texture objects
    textures: V::TexturePool,
    vertices_texture: Vertices<V::VertexRef, UVCoordinate>,
    /// Collection of geometries
    geometries: Vec<V::Geometry>,
    extra: Option<Attributes<V::StringStorage>>,
    metadata: Option<V::Metadata>,
}

impl<V: CityModelVersion> GenericCityModel<V> {
    /// Create a new empty CityModel
    pub fn new() -> Self {
        Self {
            vertices: Vertices::new(),
            semantics: V::SemanticPool::new(),
            materials: V::MaterialPool::new(),
            textures: V::TexturePool::new(),
            vertices_texture: Vertices::new(),
            geometries: Vec::new(),
            extra: None,
            metadata: None,
        }
    }

    /// Create a new CityModel with the specified capacity
    pub fn with_capacity(
        _vertex_capacity: usize,
        semantic_capacity: usize,
        material_capacity: usize,
        texture_capacity: usize,
        geometry_capacity: usize,
    ) -> Self {
        Self {
            vertices: Vertices::new(),
            semantics: V::SemanticPool::with_capacity(semantic_capacity),
            materials: V::MaterialPool::with_capacity(material_capacity),
            textures: V::TexturePool::with_capacity(texture_capacity),
            vertices_texture: Vertices::new(),
            geometries: Vec::with_capacity(geometry_capacity),
            extra: None,
            metadata: None
        }
    }

    /// Add a semantic object to the pool
    pub fn add_semantic(&mut self, semantic: V::Semantic) -> V::ResourceRef {
        self.semantics.add(semantic)
    }

    /// Get a reference to a semantic object
    pub fn get_semantic(&self, id: V::ResourceRef) -> Option<&V::Semantic> {
        self.semantics.get(id)
    }

    /// Get a mutable reference to a semantic object
    pub fn get_semantic_mut(&mut self, id: V::ResourceRef) -> Option<&mut V::Semantic> {
        self.semantics.get_mut(id)
    }

    pub fn add_material(&mut self, material: V::Material) -> V::ResourceRef {
        self.materials.add(material)
    }

    pub fn get_material(&self, id: V::ResourceRef) -> Option<&V::Material> {
        self.materials.get(id)
    }

    pub fn get_material_mut(&mut self, id: V::ResourceRef) -> Option<&mut V::Material> {
        self.materials.get_mut(id)
    }

    pub fn add_texture(&mut self, texture: V::Texture) -> V::ResourceRef {
        self.textures.add(texture)
    }

    pub fn get_texture(&self, id: V::ResourceRef) -> Option<&V::Texture> {
        self.textures.get(id)
    }

    pub fn get_texture_mut(&mut self, id: V::ResourceRef) -> Option<&mut V::Texture> {
        self.textures.get_mut(id)
    }

    /// Add a geometry to the model
    pub fn add_geometry(&mut self, geometry: V::Geometry) {
        self.geometries.push(geometry);
    }

    /// Add a vertex coordinate
    pub fn add_vertex(
        &mut self,
        coordinate: RealWorldCoordinate,
    ) -> errors::Result<VertexIndex<V::VertexRef>> {
        self.vertices.push(coordinate)
    }

    /// Get a reference to a vertex coordinate
    pub fn get_vertex(&self, index: VertexIndex<V::VertexRef>) -> Option<&RealWorldCoordinate> {
        self.vertices.get(index)
    }

    /// Get the number of geometries
    pub fn geometry_count(&self) -> usize {
        self.geometries.len()
    }

    /// Get the number of semantics
    pub fn semantic_count(&self) -> usize {
        self.semantics.iter().count()
    }

    /// Get the number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertices.as_slice().len()
    }
}

impl<V: CityModelVersion> Default for GenericCityModel<V> {
    fn default() -> Self {
        Self::new()
    }
}
