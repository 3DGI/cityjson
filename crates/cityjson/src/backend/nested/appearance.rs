//! Appearance types for the nested backend.
//!

use crate::prelude::StringStorage;

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum MaterialValues {
    PointOrLineStringOrSurface(Vec<Option<usize>>),
    Solid(Vec<Vec<Option<usize>>>),
    MultiSolid(Vec<Vec<Vec<Option<usize>>>>),
}

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum TextureValues {
    MultiOrCompositeSurface(Vec<Vec<Vec<Option<usize>>>>),
    Solid(Vec<Vec<Vec<Vec<Option<usize>>>>>),
    MultiOrCompositeSolid(Vec<Vec<Vec<Vec<Vec<Option<usize>>>>>>),
}

#[derive(Clone, Default, Debug, PartialEq)]

pub struct Appearance<SS: StringStorage> {
    pub materials: Option<Vec<Material<SS>>>,

    pub textures: Option<Vec<Texture<SS>>>,

    pub vertices_texture: Option<VerticesTexture>,

    pub default_theme_texture: Option<SS>,

    pub default_theme_material: Option<SS>,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Material<SS: StringStorage> {
    pub name: SS::String,

    pub ambient_intensity: Option<f32>,

    pub diffuse_color: Option<[f32; 3]>,

    pub emissive_color: Option<[f32; 3]>,

    pub specular_color: Option<[f32; 3]>,

    pub shininess: Option<f32>,

    pub transparency: Option<f32>,

    pub is_smooth: Option<bool>,
}

#[derive(Clone, Default, Debug, PartialEq)]

pub struct Texture<SS: StringStorage> {
    pub image_type: ImageType,
    pub image: SS::String,
    pub wrap_mode: Option<WrapMode>,
    pub texture_type: Option<TextureType>,
    pub border_color: Option<[f32; 4]>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ImageType {
    #[default]
    Png,
    Jpg,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WrapMode {
    Wrap,
    Mirror,
    Clamp,
    Border,
    #[default]
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextureType {
    #[default]
    Unknown,
    Specific,
    Typical,
}

pub type VerticesTexture = Vec<[f32; 2]>;
