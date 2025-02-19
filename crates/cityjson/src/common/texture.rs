use crate::common::storage::StringStorage;

pub type RGBA = [f32; 4];

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

pub trait Texture<S: StringStorage> {
    /// Create a new texture with the given image and image type
    #[inline]
    fn new(image: S::String, image_type: ImageType) -> Self;
    #[inline]
    fn image_type(&self) -> &ImageType;
    #[inline]
    fn set_image_type(&mut self, image_type: ImageType);
    #[inline]
    fn image(&self) -> &S::String;
    #[inline]
    fn set_image(&mut self, image: S::String);
    #[inline]
    fn wrap_mode(&self) -> Option<WrapMode>;
    #[inline]
    fn set_wrap_mode(&mut self, wrap_mode: Option<WrapMode>);
    #[inline]
    fn texture_type(&self) -> Option<TextureType>;
    #[inline]
    fn set_texture_type(&mut self, texture_type: Option<TextureType>);
    #[inline]
    fn border_color(&self) -> Option<RGBA>;
    #[inline]
    fn set_border_color(&mut self, border_color: Option<RGBA>);
}