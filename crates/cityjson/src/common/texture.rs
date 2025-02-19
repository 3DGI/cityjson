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
    fn new(image: S::String, image_type: ImageType) -> Self;
    fn image_type(&self) -> &ImageType;
    fn set_image_type(&mut self, image_type: ImageType);
    fn image(&self) -> &S::String;
    fn set_image(&mut self, image: S::String);
    fn wrap_mode(&self) -> Option<WrapMode>;
    fn set_wrap_mode(&mut self, wrap_mode: Option<WrapMode>);
    fn texture_type(&self) -> Option<TextureType>;
    fn set_texture_type(&mut self, texture_type: Option<TextureType>);
    fn border_color(&self) -> Option<RGBA>;
    fn set_border_color(&mut self, border_color: Option<RGBA>);
}