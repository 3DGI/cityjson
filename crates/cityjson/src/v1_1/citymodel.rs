//! # CityModel
//!
//! Represents a [CityJSON object](https://www.cityjson.org/specs/1.1.3/#cityjson-object).

use crate::cityjson::attributes::Attributes;
use crate::cityjson::citymodel::{CityModelTrait, CityModelTypes};
use crate::cityjson::coordinate::{RealWorldCoordinate, UVCoordinate, Vertices};
use crate::cityjson::vertex::{VertexIndex, VertexRef};
use crate::resources::pool::{DefaultResourcePool, ResourceId32, ResourcePool, ResourceRef};
use crate::resources::storage::{OwnedStringStorage, StringStorage};
use crate::v1_1::appearance::material::Material;
use crate::v1_1::appearance::texture::Texture;
use crate::v1_1::geometry::semantic::{Semantic, SemanticType};
use crate::v1_1::geometry::Geometry;
use crate::v1_1::metadata::Metadata;
use std::marker::PhantomData;

pub struct V1_1<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    _phantom_vr: PhantomData<VR>,
    _phantom_rr: PhantomData<RR>,
    _phantom_ss: PhantomData<SS>,
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> CityModelTypes for V1_1<VR, RR, SS> {
    type CoordinateType = RealWorldCoordinate;
    type VertexRef = VR;
    type ResourceRef = RR;
    type StringStorage = SS;
    type SemType = SemanticType;
    type Semantic = Semantic<RR, SS>;
    type Material = Material<SS>;
    type Texture = Texture<SS>;
    type Geometry = Geometry<VR, RR>;
    type Metadata = Metadata<SS>;
    type GeometryPool = DefaultResourcePool<Geometry<VR, RR>, RR>;
    type SemanticPool = DefaultResourcePool<Semantic<RR, SS>, RR>;
    type MaterialPool = DefaultResourcePool<Material<SS>, RR>;
    type TexturePool = DefaultResourcePool<Texture<SS>, RR>;
}

#[derive(Debug)]
pub struct CityModel<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    /// Pool of vertex coordinates
    vertices: Vertices<VR, RealWorldCoordinate>,
    /// Pool of geometries
    geometries: DefaultResourcePool<Geometry<VR, RR>, RR>,
    /// Pool of semantic objects
    semantics: DefaultResourcePool<Semantic<RR, SS>, RR>,
    /// Pool of material objects
    materials: DefaultResourcePool<Material<SS>, RR>,
    /// Pool of texture objects
    textures: DefaultResourcePool<Texture<SS>, RR>,
    vertices_texture: Vertices<VR, UVCoordinate>,
    extra: Option<Attributes<SS>>,
    metadata: Option<Metadata<SS>>,
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> CityModelTrait<V1_1<VR, RR, SS>>
    for CityModel<VR, RR, SS>
{
    fn new() -> Self {
        Self {
            vertices: Vertices::new(),
            geometries: DefaultResourcePool::new_pool(),
            semantics: DefaultResourcePool::new_pool(),
            materials: DefaultResourcePool::new_pool(),
            textures: DefaultResourcePool::new_pool(),
            vertices_texture: Vertices::new(),
            extra: None,
            metadata: None,
        }
    }

    fn with_capacity(
        vertex_capacity: usize,
        semantic_capacity: usize,
        material_capacity: usize,
        texture_capacity: usize,
        geometry_capacity: usize,
    ) -> Self {
        Self {
            vertices: Vertices::new(),
            geometries: DefaultResourcePool::new_pool(),
            semantics: DefaultResourcePool::new_pool(),
            materials: DefaultResourcePool::new_pool(),
            textures: DefaultResourcePool::new_pool(),
            vertices_texture: Vertices::new(),
            extra: None,
            metadata: None,
        }
    }

    fn add_semantic(&mut self, semantic: Semantic<RR, SS>) -> RR {
        self.semantics.add(semantic)
    }

    fn get_semantic(&self, id: RR) -> Option<&Semantic<RR, SS>> {
        self.semantics.get(id)
    }

    fn get_semantic_mut(&mut self, id: RR) -> Option<&mut Semantic<RR, SS>> {
        self.semantics.get_mut(id)
    }

    fn add_material(&mut self, material: Material<SS>) -> RR {
        self.materials.add(material)
    }

    fn get_material(&self, id: RR) -> Option<&Material<SS>> {
        self.materials.get(id)
    }

    fn get_material_mut(&mut self, id: RR) -> Option<&mut Material<SS>> {
        self.materials.get_mut(id)
    }

    fn add_texture(&mut self, texture: Texture<SS>) -> RR {
        self.textures.add(texture)
    }

    fn get_texture(&self, id: RR) -> Option<&Texture<SS>> {
        self.textures.get(id)
    }

    fn get_texture_mut(&mut self, id: RR) -> Option<&mut Texture<SS>> {
        self.textures.get_mut(id)
    }

    fn add_geometry(&mut self, geometry: Geometry<VR, RR>) -> RR {
        self.geometries.add(geometry)
    }

    fn geometries(&self) -> &DefaultResourcePool<Geometry<VR, RR>, RR> {
        &self.geometries
    }

    fn geometries_mut(&mut self) -> &mut DefaultResourcePool<Geometry<VR, RR>, RR> {
        &mut self.geometries
    }

    fn add_vertex(
        &mut self,
        coordinate: RealWorldCoordinate,
    ) -> crate::errors::Result<VertexIndex<VR>> {
        self.vertices.push(coordinate)
    }

    fn get_vertex(&self, index: VertexIndex<VR>) -> Option<&RealWorldCoordinate> {
        self.vertices.get(index)
    }

    fn geometry_count(&self) -> usize {
        self.geometries.len()
    }

    fn semantic_count(&self) -> usize {
        self.semantics.len()
    }

    fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

#[test]
fn test_citymodel() {
    let cm: CityModel<u32, ResourceId32, OwnedStringStorage> = CityModel::new();
}
