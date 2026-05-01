//! Surface texture definitions for `CityJSON` v2.0.
//!
//! A [`Texture`] references an image file and defines how it wraps onto surfaces.
//! Textures are stored in the model's texture pool and referenced from geometry by
//! [`TextureHandle`] via a theme map. UV coordinates are stored in the model's UV vertex pool.
//!
//! `wrapMode` is mandatory in the spec: `"wrap"`, `"mirror"`, `"clamp"`, or `"border"`.
//! When `wrapMode` is `"border"`, `borderColor` gives the RGBA fill color for areas outside
//! the texture.
//!
//! Spec: [Texture Object](https://www.cityjson.org/specs/2.0.1/#texture-object).
//!
//! [`TextureHandle`]: crate::resources::handles::TextureHandle
//!
//! ```rust
//! use cityjson_types::CityModelType;
//! use cityjson_types::v2_0::{ImageType, OwnedCityModel, WrapMode};
//! use cityjson_types::v2_0::appearance::texture::OwnedTexture;
//!
//! let mut model = OwnedCityModel::new(CityModelType::CityJSON);
//!
//! let mut tex = OwnedTexture::new("textures/brick.png".to_string(), ImageType::Png);
//! tex.set_wrap_mode(Some(WrapMode::Wrap));
//!
//! let handle = model.add_texture(tex).unwrap();
//! assert!(model.get_texture(handle).is_some());
//! ```
//!
//! ```rust
//! use cityjson_types::v2_0::appearance::texture::OwnedTexture;
//! use cityjson_types::v2_0::appearance::{ImageType, RGBA, TextureType, WrapMode};
//!
//! let mut texture = OwnedTexture::new("roof.png".to_string(), ImageType::Png);
//! texture.set_wrap_mode(Some(WrapMode::Border));
//! texture.set_texture_type(Some(TextureType::Specific));
//!
//! let border = RGBA::new(1.0, 1.0, 1.0, 0.5);
//! let raw_border = border.to_array();
//! assert_eq!(raw_border, [1.0, 1.0, 1.0, 0.5]);
//!
//! let replacement = RGBA::from([0.2, 0.3, 0.4, 1.0]);
//! let raw_replacement: [f32; 4] = replacement.into();
//! assert_eq!(raw_replacement, [0.2, 0.3, 0.4, 1.0]);
//!
//! texture.set_border_color(Some(border));
//! assert_eq!(texture.border_color(), Some(RGBA::from([1.0, 1.0, 1.0, 0.5])));
//! assert_eq!(texture.wrap_mode(), Some(WrapMode::Border));
//! assert_eq!(texture.texture_type(), Some(TextureType::Specific));
//! ```

use crate::format_option;
use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};
use crate::v2_0::appearance::{ImageType, RGBA, TextureType, WrapMode};
use std::fmt::{Display, Formatter};

pub type OwnedTexture = Texture<OwnedStringStorage>;
pub type BorrowedTexture<'a> = Texture<BorrowedStringStorage<'a>>;

/// A surface texture. See the [module docs](self) for usage.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Texture<SS: StringStorage> {
    image_type: ImageType,
    image: SS::String,
    wrap_mode: Option<WrapMode>,
    mapping_type: Option<TextureType>,
    border_color: Option<RGBA>,
}

impl<SS: StringStorage> Texture<SS> {
    #[inline]
    pub fn new(image: SS::String, image_type: ImageType) -> Self {
        Self {
            image_type,
            image,
            wrap_mode: None,
            mapping_type: None,
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
        self.mapping_type
    }
    #[inline]
    pub fn set_texture_type(&mut self, texture_type: Option<TextureType>) {
        self.mapping_type = texture_type;
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

impl<SS: StringStorage> Display for Texture<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "image_type: {:?}, image: {:?}, wrap_mode: {}, mapping_type: {}, border_color: {}",
            self.image_type,
            self.image,
            format_option(self.wrap_mode.as_ref()),
            format_option(self.mapping_type.as_ref()),
            format_option(self.border_color.as_ref())
        )
    }
}
