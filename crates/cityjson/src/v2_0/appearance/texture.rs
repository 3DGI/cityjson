use crate::cityjson::core::appearance::*;
use crate::cityjson::traits::appearance::*;
use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};

/// Type alias for a texture with owned string storage
pub type OwnedTexture = Texture<OwnedStringStorage>;

/// Type alias for a texture with borrowed string storage
pub type BorrowedTexture<'a> = Texture<BorrowedStringStorage<'a>>;

#[repr(C)]
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Texture<SS: StringStorage> {
    image_type: ImageType,
    image: SS::String,
    wrap_mode: Option<WrapMode>,
    texture_type: Option<TextureType>,
    border_color: Option<RGBA>,
}

impl<SS: StringStorage> TextureTrait<SS> for Texture<SS> {
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
