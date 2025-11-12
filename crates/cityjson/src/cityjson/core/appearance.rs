use crate::cityjson::traits::appearance::material::MaterialTrait;
use crate::cityjson::traits::appearance::texture::TextureTrait;
use crate::resources::storage::StringStorage;
use std::{fmt, write};

pub type RGB = [f32; 3];
pub type RGBA = [f32; 4];

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ImageType {
    #[default]
    Png,
    Jpg,
}

impl fmt::Display for ImageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageType::Png => write!(f, "PNG"),
            ImageType::Jpg => write!(f, "JPG"),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WrapMode {
    Wrap,
    Mirror,
    Clamp,
    Border,
    #[default]
    None,
}

impl fmt::Display for WrapMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WrapMode::Wrap => write!(f, "wrap"),
            WrapMode::Mirror => write!(f, "mirror"),
            WrapMode::Clamp => write!(f, "clamp"),
            WrapMode::Border => write!(f, "border"),
            WrapMode::None => write!(f, "none"),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextureType {
    #[default]
    Unknown,
    Specific,
    Typical,
}

impl fmt::Display for TextureType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextureType::Unknown => write!(f, "unknown"),
            TextureType::Specific => write!(f, "specific"),
            TextureType::Typical => write!(f, "typical"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaterialCore<SS: StringStorage> {
    name: SS::String,
    ambient_intensity: Option<f32>,
    diffuse_color: Option<RGB>,
    emissive_color: Option<RGB>,
    specular_color: Option<RGB>,
    shininess: Option<f32>,
    transparency: Option<f32>,
    is_smooth: Option<bool>,
}

impl<SS: StringStorage> MaterialTrait<SS> for MaterialCore<SS> {
    fn new(name: SS::String) -> Self {
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
    fn name(&self) -> &SS::String {
        &self.name
    }
    #[inline]
    fn set_name(&mut self, name: SS::String) {
        self.name = name;
    }
    #[inline]
    fn ambient_intensity(&self) -> Option<f32> {
        self.ambient_intensity
    }
    #[inline]
    fn set_ambient_intensity(&mut self, ambient_intensity: Option<f32>) {
        self.ambient_intensity = ambient_intensity;
    }
    #[inline]
    fn diffuse_color(&self) -> Option<&RGB> {
        self.diffuse_color.as_ref()
    }
    #[inline]
    fn set_diffuse_color(&mut self, diffuse_color: Option<RGB>) {
        self.diffuse_color = diffuse_color;
    }
    #[inline]
    fn emissive_color(&self) -> Option<&RGB> {
        self.emissive_color.as_ref()
    }
    #[inline]
    fn set_emissive_color(&mut self, emissive_color: Option<RGB>) {
        self.emissive_color = emissive_color;
    }
    #[inline]
    fn specular_color(&self) -> Option<&RGB> {
        self.specular_color.as_ref()
    }
    #[inline]
    fn set_specular_color(&mut self, specular_color: Option<RGB>) {
        self.specular_color = specular_color;
    }
    #[inline]
    fn shininess(&self) -> Option<f32> {
        self.shininess
    }
    #[inline]
    fn set_shininess(&mut self, shininess: Option<f32>) {
        self.shininess = shininess;
    }
    #[inline]
    fn transparency(&self) -> Option<f32> {
        self.transparency
    }
    #[inline]
    fn set_transparency(&mut self, transparency: Option<f32>) {
        self.transparency = transparency;
    }
    #[inline]
    fn is_smooth(&self) -> Option<bool> {
        self.is_smooth
    }
    #[inline]
    fn set_is_smooth(&mut self, is_smooth: Option<bool>) {
        self.is_smooth = is_smooth;
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct TextureCore<SS: StringStorage> {
    image_type: ImageType,
    image: SS::String,
    wrap_mode: Option<WrapMode>,
    texture_type: Option<TextureType>,
    border_color: Option<RGBA>,
}

impl<SS: StringStorage> TextureTrait<SS> for TextureCore<SS> {
    #[inline]
    fn new(image: SS::String, image_type: ImageType) -> Self {
        Self {
            image_type,
            image,
            wrap_mode: None,
            texture_type: None,
            border_color: None,
        }
    }
    #[inline]
    fn image_type(&self) -> &ImageType {
        &self.image_type
    }
    #[inline]
    fn set_image_type(&mut self, image_type: ImageType) {
        self.image_type = image_type;
    }
    #[inline]
    fn image(&self) -> &SS::String {
        &self.image
    }
    #[inline]
    fn set_image(&mut self, image: SS::String) {
        self.image = image;
    }
    #[inline]
    fn wrap_mode(&self) -> Option<WrapMode> {
        self.wrap_mode
    }
    #[inline]
    fn set_wrap_mode(&mut self, wrap_mode: Option<WrapMode>) {
        self.wrap_mode = wrap_mode;
    }
    #[inline]
    fn texture_type(&self) -> Option<TextureType> {
        self.texture_type
    }
    #[inline]
    fn set_texture_type(&mut self, texture_type: Option<TextureType>) {
        self.texture_type = texture_type;
    }
    #[inline]
    fn border_color(&self) -> Option<RGBA> {
        self.border_color
    }
    #[inline]
    fn set_border_color(&mut self, border_color: Option<RGBA>) {
        self.border_color = border_color;
    }
}
