//! # Texture
//!
//! Represents a [Texture object](https://www.cityjson.org/specs/1.1.3/#texture-object).
use crate::common::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};

pub type OwnedTexture = Texture<OwnedStringStorage>;
pub type BorrowedTexture<'a> = Texture<BorrowedStringStorage<'a>>;

pub type RGBA = [f32; 4];

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Texture<S: StringStorage> {
    image_type: ImageType,
    image: S::String,
    wrap_mode: Option<WrapMode>,
    texture_type: Option<TextureType>,
    border_color: Option<RGBA>,
}

impl<S: StringStorage> Texture<S> {
    #[inline]
    pub fn image_type(&self) -> &ImageType {
        &self.image_type
    }

    #[inline]
    pub fn set_image_type(&mut self, image_type: ImageType) {
        self.image_type = image_type;
    }

    #[inline]
    pub fn image(&self) -> &S::String {
        &self.image
    }

    #[inline]
    pub fn set_image(&mut self, image: S::String) {
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
