use crate::resource_pool::ResourceId;

#[derive(Debug, Clone)]
pub struct Material;

#[derive(Debug, Clone)]
pub struct Semantic;

#[derive(Debug, Clone)]
pub struct Texture;

#[derive(Debug, Clone)]
pub struct GeometryType;

#[derive(Debug, Clone)]
pub struct Boundary;

#[derive(Debug, Clone)]
pub struct Geometry {
    type_: GeometryType,
    lod: Option<LoD>,
    boundaries: Boundary,
    material_ids: Vec<ResourceId>,
    semantic_ids: Vec<ResourceId>,
    texture_ids: Vec<ResourceId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LoD {
    LoD0,
    LoD0_0,
    LoD0_1,
    LoD0_2,
    LoD0_3,
    LoD1,
    LoD1_0,
    LoD1_1,
    LoD1_2,
    LoD1_3,
    LoD2,
    LoD2_0,
    LoD2_1,
    LoD2_2,
    LoD2_3,
    LoD3,
    LoD3_0,
    LoD3_1,
    LoD3_2,
    LoD3_3,
}
