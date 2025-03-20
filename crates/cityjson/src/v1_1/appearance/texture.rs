//! # Texture
//!
//! Represents a [Texture object](https://www.cityjson.org/specs/1.1.3/#texture-object).

use crate::cityjson::core::appearance::*;
use crate::cityjson::traits::appearance::*;
use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};

/// Type alias for a texture with owned string storage
pub type OwnedTexture = Texture<OwnedStringStorage>;

/// Type alias for a texture with borrowed string storage
pub type BorrowedTexture<'a> = Texture<BorrowedStringStorage<'a>>;

/// A structure representing a texture in CityJSON.
///
/// Textures define image-based visual appearance for surfaces in a 3D city model.
/// This implementation supports all texture properties defined in the
/// [CityJSON 1.1.3 specification](https://www.cityjson.org/specs/1.1.3/#texture-object).
///
/// # Type Parameters
///
/// * `SS` - The string storage strategy (owned or borrowed)
///
/// # Examples
///
/// Creating a new texture and setting its properties:
///
/// ```
/// use cityjson::prelude::*;
/// use cityjson::v1_1::*;
///
/// // Create a new texture with an image path and type
/// let mut texture = Texture::<OwnedStringStorage>::new(
///     "textures/facade.jpg".to_string(),
///     ImageType::Jpg
/// );
///
/// // Set texture properties
/// texture.set_wrap_mode(Some(WrapMode::Mirror));
/// texture.set_texture_type(Some(TextureType::Specific));
/// texture.set_border_color(Some([0.0, 0.0, 0.0, 1.0]));
///
/// // Access texture properties
/// assert_eq!(texture.image(), "textures/facade.jpg");
/// assert_eq!(*texture.image_type(), ImageType::Jpg);
/// assert_eq!(texture.wrap_mode(), Some(WrapMode::Mirror));
/// assert_eq!(texture.texture_type(), Some(TextureType::Specific));
/// assert_eq!(texture.border_color(), Some([0.0, 0.0, 0.0, 1.0]));
/// ```
///
/// Using the `OwnedTexture` type alias:
///
/// ```
/// use cityjson::prelude::*;
/// use cityjson::v1_1::*;
///
/// let mut texture = OwnedTexture::new("textures/roof.png".to_string(), ImageType::Png);
/// texture.set_image("textures/better_roof.png".to_string());
/// texture.set_image_type(ImageType::Png);
///
/// assert_eq!(texture.image(), "textures/better_roof.png");
/// assert_eq!(*texture.image_type(), ImageType::Png);
/// ```
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
