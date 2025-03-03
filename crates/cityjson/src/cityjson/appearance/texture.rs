use crate::cityjson::appearance::{ImageType, TextureType, WrapMode, RGBA};
use crate::resources::storage::StringStorage;

pub trait Texture<SS: StringStorage> {
    /// Create a new texture with the given image and image type
    fn new(image: SS::String, image_type: ImageType) -> Self;
    fn image_type(&self) -> &ImageType;
    fn set_image_type(&mut self, image_type: ImageType);
    fn image(&self) -> &SS::String;
    fn set_image(&mut self, image: SS::String);
    fn wrap_mode(&self) -> Option<WrapMode>;
    fn set_wrap_mode(&mut self, wrap_mode: Option<WrapMode>);
    fn texture_type(&self) -> Option<TextureType>;
    fn set_texture_type(&mut self, texture_type: Option<TextureType>);
    fn border_color(&self) -> Option<RGBA>;
    fn set_border_color(&mut self, border_color: Option<RGBA>);
}
