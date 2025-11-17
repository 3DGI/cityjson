//! Appearance types for the nested backend.
//!

use crate::prelude::StringStorage;

#[allow(clippy::upper_case_acronyms)]
pub type RGB = [f32; 3];
#[allow(clippy::upper_case_acronyms)]
pub type RGBA = [f32; 4];

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum MaterialValues {
    PointOrLineStringOrSurface(Vec<Option<usize>>),
    Solid(Vec<Vec<Option<usize>>>),
    MultiSolid(Vec<Vec<Vec<Option<usize>>>>),
}

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[allow(clippy::type_complexity)]
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
    pub default_theme_texture: Option<SS::String>,
    pub default_theme_material: Option<SS::String>,
}

impl<SS: StringStorage> Appearance<SS> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn materials(&self) -> Option<&Vec<Material<SS>>> {
        self.materials.as_ref()
    }

    pub fn materials_mut(&mut self) -> &mut Vec<Material<SS>> {
        if self.materials.is_none() {
            self.materials = Some(Vec::new());
        }
        self.materials.as_mut().unwrap()
    }

    pub fn textures(&self) -> Option<&Vec<Texture<SS>>> {
        self.textures.as_ref()
    }

    pub fn textures_mut(&mut self) -> &mut Vec<Texture<SS>> {
        if self.textures.is_none() {
            self.textures = Some(Vec::new());
        }
        self.textures.as_mut().unwrap()
    }

    pub fn vertices_texture(&self) -> Option<&VerticesTexture> {
        self.vertices_texture.as_ref()
    }

    pub fn vertices_texture_mut(&mut self) -> &mut VerticesTexture {
        if self.vertices_texture.is_none() {
            self.vertices_texture = Some(Vec::new());
        }
        self.vertices_texture.as_mut().unwrap()
    }

    pub fn default_theme_material(&self) -> Option<&SS::String> {
        self.default_theme_material.as_ref()
    }

    pub fn set_default_theme_material(&mut self, theme: Option<SS::String>) {
        self.default_theme_material = theme;
    }

    pub fn default_theme_texture(&self) -> Option<&SS::String> {
        self.default_theme_texture.as_ref()
    }

    pub fn set_default_theme_texture(&mut self, theme: Option<SS::String>) {
        self.default_theme_texture = theme;
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Material<SS: StringStorage> {
    name: SS::String,
    ambient_intensity: Option<f32>,
    diffuse_color: Option<RGB>,
    emissive_color: Option<RGB>,
    specular_color: Option<RGB>,
    shininess: Option<f32>,
    transparency: Option<f32>,
    is_smooth: Option<bool>,
}

impl<SS: StringStorage> Material<SS> {
    pub fn new(name: SS::String) -> Self {
        Self {
            name,
            ambient_intensity: None,
            diffuse_color: None,
            emissive_color: None,
            specular_color: None,
            shininess: None,
            transparency: None,
            is_smooth: None,
        }
    }

    #[inline]
    pub fn name(&self) -> &SS::String {
        &self.name
    }

    #[inline]
    pub fn set_name(&mut self, name: SS::String) {
        self.name = name;
    }

    #[inline]
    pub fn ambient_intensity(&self) -> Option<f32> {
        self.ambient_intensity
    }

    #[inline]
    pub fn set_ambient_intensity(&mut self, ambient_intensity: Option<f32>) {
        self.ambient_intensity = ambient_intensity;
    }

    #[inline]
    pub fn diffuse_color(&self) -> Option<&RGB> {
        self.diffuse_color.as_ref()
    }

    #[inline]
    pub fn set_diffuse_color(&mut self, diffuse_color: Option<RGB>) {
        self.diffuse_color = diffuse_color;
    }

    #[inline]
    pub fn emissive_color(&self) -> Option<&RGB> {
        self.emissive_color.as_ref()
    }

    #[inline]
    pub fn set_emissive_color(&mut self, emissive_color: Option<RGB>) {
        self.emissive_color = emissive_color;
    }

    #[inline]
    pub fn specular_color(&self) -> Option<&RGB> {
        self.specular_color.as_ref()
    }

    #[inline]
    pub fn set_specular_color(&mut self, specular_color: Option<RGB>) {
        self.specular_color = specular_color;
    }

    #[inline]
    pub fn shininess(&self) -> Option<f32> {
        self.shininess
    }

    #[inline]
    pub fn set_shininess(&mut self, shininess: Option<f32>) {
        self.shininess = shininess;
    }

    #[inline]
    pub fn transparency(&self) -> Option<f32> {
        self.transparency
    }

    #[inline]
    pub fn set_transparency(&mut self, transparency: Option<f32>) {
        self.transparency = transparency;
    }

    #[inline]
    pub fn is_smooth(&self) -> Option<bool> {
        self.is_smooth
    }

    #[inline]
    pub fn set_is_smooth(&mut self, is_smooth: Option<bool>) {
        self.is_smooth = is_smooth;
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Texture<SS: StringStorage> {
    image_type: ImageType,
    image: SS::String,
    wrap_mode: Option<WrapMode>,
    texture_type: Option<TextureType>,
    border_color: Option<RGBA>,
}

impl<SS: StringStorage> Texture<SS> {
    #[inline]
    pub fn new(image: SS::String, image_type: ImageType) -> Self {
        Self {
            image_type,
            image,
            wrap_mode: None,
            texture_type: None,
            border_color: None,
        }
    }

    #[inline]
    pub fn image_type(&self) -> &ImageType {
        &self.image_type
    }

    #[inline]
    pub fn set_image_type(&mut self, image_type: ImageType) {
        self.image_type = image_type;
    }

    #[inline]
    pub fn image(&self) -> &SS::String {
        &self.image
    }

    #[inline]
    pub fn set_image(&mut self, image: SS::String) {
        self.image = image;
    }

    #[inline]
    pub fn wrap_mode(&self) -> Option<WrapMode> {
        self.wrap_mode
    }

    #[inline]
    pub fn set_wrap_mode(&mut self, wrap_mode: Option<WrapMode>) {
        self.wrap_mode = wrap_mode;
    }

    #[inline]
    pub fn texture_type(&self) -> Option<TextureType> {
        self.texture_type
    }

    #[inline]
    pub fn set_texture_type(&mut self, texture_type: Option<TextureType>) {
        self.texture_type = texture_type;
    }

    #[inline]
    pub fn border_color(&self) -> Option<RGBA> {
        self.border_color
    }

    #[inline]
    pub fn set_border_color(&mut self, border_color: Option<RGBA>) {
        self.border_color = border_color;
    }
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
