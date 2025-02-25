//! # CityModel
//!
//! Represents a [CityJSON object](https://www.cityjson.org/specs/1.1.3/#cityjson-object).

use crate::cityjson::citymodel::{CityModelTrait, CityModelVersion, GenericCityModel};
use crate::cityjson::coordinate::RealWorldCoordinate;
use crate::cityjson::vertex::{VertexIndex, VertexRef};
use crate::resources::pool::{DefaultResourcePool, ResourceId32, ResourceRef};
use crate::resources::storage::{OwnedStringStorage, StringStorage};
use crate::v1_1::appearance::material::Material;
use crate::v1_1::appearance::texture::Texture;
use crate::v1_1::geometry::semantic::Semantic;
use crate::v1_1::geometry::Geometry;
use crate::v1_1::metadata::Metadata;
use std::marker::PhantomData;

struct V1_1<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    _phantom_vr: PhantomData<VR>,
    _phantom_rr: PhantomData<RR>,
    _phantom_ss: PhantomData<SS>,
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> CityModelVersion for V1_1<VR, RR, SS> {
    type CoordinateType = RealWorldCoordinate;
    type VertexRef = VR;
    type ResourceRef = RR;
    type StringStorage = SS;
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

pub struct CityModel<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    inner: GenericCityModel<V1_1<VR, RR, SS>>,
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> CityModelTrait<V1_1<VR, RR, SS>>
    for CityModel<VR, RR, SS>
{
    fn new() -> Self {
        Self {
            inner: GenericCityModel::new()
        }
    }

    fn with_capacity(
        _vertex_capacity: usize,
        semantic_capacity: usize,
        material_capacity: usize,
        texture_capacity: usize,
        geometry_capacity: usize,
    ) -> Self {
        Self {
            inner: GenericCityModel::with_capacity(_vertex_capacity,semantic_capacity, material_capacity,texture_capacity, geometry_capacity)
        }
    }

    fn add_semantic(
        &mut self,
        semantic: Semantic<RR, SS>,
    ) -> RR {
        self.inner.add_semantic(semantic)
    }

    fn get_semantic(
        &self,
        id: RR,
    ) -> Option<&Semantic<RR, SS>> {
        self.inner.get_semantic(id)
    }

    fn get_semantic_mut(
        &mut self,
        id: RR,
    ) -> Option<&mut Semantic<RR, SS>> {
        self.inner.get_semantic_mut(id)
    }

    fn add_material(
        &mut self,
        material: Material<SS>,
    ) -> RR {
        self.inner.add_material(material)
    }

    fn get_material(
        &self,
        id: RR,
    ) -> Option<&Material<SS>> {
        self.inner.get_material(id)
    }

    fn get_material_mut(
        &mut self,
        id: RR,
    ) -> Option<&mut Material<SS>> {
        self.inner.get_material_mut(id)
    }

    fn add_texture(&mut self, texture: Texture<SS>) -> RR {
        self.inner.add_texture(texture)
    }

    fn get_texture(&self, id: RR) -> Option<&Texture<SS>> {
        self.inner.get_texture(id)
    }

    fn get_texture_mut(
        &mut self,
        id: RR,
    ) -> Option<&mut Texture<SS>> {
        self.inner.get_texture_mut(id)
    }

    fn add_geometry(&mut self, geometry: Geometry<VR, RR>) {
        self.inner.add_geometry(geometry)
    }

    fn add_vertex(
        &mut self,
        coordinate: RealWorldCoordinate,
    ) -> crate::errors::Result<VertexIndex<VR>> {
        self.inner.add_vertex(coordinate)
    }

    fn get_vertex(
        &self,
        index: VertexIndex<VR>,
    ) -> Option<&RealWorldCoordinate> {
        self.inner.get_vertex(index)
    }

    fn geometry_count(&self) -> usize {
        self.inner.geometry_count()
    }

    fn semantic_count(&self) -> usize {
        self.inner.semantic_count()
    }

    fn vertex_count(&self) -> usize {
        self.inner.vertex_count()
    }
}

#[test]
fn test_citymodel() {
    let cm: CityModel<u32, ResourceId32, OwnedStringStorage> = CityModel {
        inner: GenericCityModel::new(),
    };
}
