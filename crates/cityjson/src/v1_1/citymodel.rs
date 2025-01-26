//! # CityModel
//!
//! Represents a [CityJSON object](https://www.cityjson.org/specs/1.1.3/#cityjson-object).
use crate::common::attributes::Attributes;
use crate::common::coordinate::{RealWorldCoordinate, UVCoordinate, Vertices};
use crate::common::storage::{OwnedStringStorage, StringStorage};
use crate::common::index::{VertexIndex, VertexInteger};
use crate::errors;
use crate::resources::pool::{DefaultResourcePool, ResourceId, ResourcePool};
use crate::v1_1::geometry::Geometry;
use crate::v1_1::material::Material;
use crate::v1_1::semantic::Semantic;
use crate::v1_1::texture::Texture;

pub type CityModel<VI, S> = GenericCityModel<
    VI,
    DefaultResourcePool<Semantic<VI, S>>,
    DefaultResourcePool<Material<S>>,
    DefaultResourcePool<Texture<S>>,
    OwnedStringStorage,
>;

#[derive(Debug)]
pub struct GenericCityModel<VI, RPS, RPM, RPT, S>
where
    VI: VertexInteger,
    RPS: ResourcePool<Semantic<VI, S>>,
    RPM: ResourcePool<Material<S>>,
    RPT: ResourcePool<Texture<S>>,
    S: StringStorage,
{
    /// Pool of vertex coordinates
    vertices: Vertices<VI, RealWorldCoordinate>,
    /// Pool of semantic objects
    semantics: RPS,
    /// Pool of material objects
    materials: RPM,
    /// Pool of texture objects
    textures: RPT,
    vertices_texture: Vertices<VI, UVCoordinate>,
    /// Collection of geometries
    pub(crate) geometries: Vec<Geometry<VI>>,
    extra: Option<Attributes<S>>,
}

impl<VI, RPS, RPM, RPT, S> GenericCityModel<VI, RPS, RPM, RPT, S>
where
    VI: VertexInteger,
    RPS: ResourcePool<Semantic<VI, S>>,
    RPM: ResourcePool<Material<S>>,
    RPT: ResourcePool<Texture<S>>,
    S: StringStorage,
{
    /// Create a new empty CityModel
    pub fn new() -> Self {
        Self {
            vertices: Vertices::new(),
            semantics: RPS::new(),
            materials: RPM::new(),
            textures: RPT::new(),
            vertices_texture: Vertices::new(),
            geometries: Vec::new(),
            extra: None,
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
            semantics: RPS::with_capacity(semantic_capacity),
            materials: RPM::with_capacity(material_capacity),
            textures: RPT::with_capacity(texture_capacity),
            vertices_texture: Vertices::new(),
            geometries: Vec::with_capacity(geometry_capacity),
            extra: None,
        }
    }

    /// Add a semantic object to the pool
    pub fn add_semantic(&mut self, semantic: Semantic<VI, S>) -> ResourceId {
        self.semantics.add(semantic)
    }

    /// Get a reference to a semantic object
    pub fn get_semantic(&self, id: ResourceId) -> Option<&Semantic<VI, S>> {
        self.semantics.get(id)
    }

    /// Get a mutable reference to a semantic object
    pub fn get_semantic_mut(&mut self, id: ResourceId) -> Option<&mut Semantic<VI, S>> {
        self.semantics.get_mut(id)
    }

    pub fn add_material(&mut self, material: Material<S>) -> ResourceId {
        self.materials.add(material)
    }

    pub fn get_material(&self, id: ResourceId) -> Option<&Material<S>> {
        self.materials.get(id)
    }

    pub fn get_material_mut(&mut self, id: ResourceId) -> Option<&mut Material<S>> {
        self.materials.get_mut(id)
    }

    pub fn add_texture(&mut self, texture: Texture<S>) -> ResourceId {
        self.textures.add(texture)
    }

    pub fn get_texture(&self, id: ResourceId) -> Option<&Texture<S>> {
        self.textures.get(id)
    }

    pub fn get_texture_mut(&mut self, id: ResourceId) -> Option<&mut Texture<S>> {
        self.textures.get_mut(id)
    }

    /// Add a geometry to the model
    pub fn add_geometry(&mut self, geometry: Geometry<VI>) {
        self.geometries.push(geometry);
    }

    /// Add a vertex coordinate
    pub fn add_vertex(&mut self, coordinate: RealWorldCoordinate) -> errors::Result<VertexIndex<VI>> {
        self.vertices.push(coordinate)
    }

    /// Get a reference to a vertex coordinate
    pub fn get_vertex(&self, index: VertexIndex<VI>) -> Option<&RealWorldCoordinate> {
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

// Implement default for convenience
impl<VI, RPS, RPM, RPT, S> GenericCityModel<VI, RPS, RPM, RPT, S>
where
    VI: VertexInteger,
    RPS: ResourcePool<Semantic<VI, S>>,
    RPM: ResourcePool<Material<S>>,
    RPT: ResourcePool<Texture<S>>,
    S: StringStorage,
{
    fn default() -> Self {
        Self::new()
    }
}