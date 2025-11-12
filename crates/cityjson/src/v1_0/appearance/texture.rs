use crate::cityjson::core::appearance::*;
use crate::macros::impl_texture_trait;
use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};

/// Type alias for a texture with owned string storage
pub type OwnedTexture = Texture<OwnedStringStorage>;

/// Type alias for a texture with borrowed string storage
pub type BorrowedTexture<'a> = Texture<BorrowedStringStorage<'a>>;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Texture<SS: StringStorage> {
    image_type: ImageType,
    image: SS::String,
    wrap_mode: Option<WrapMode>,
    texture_type: Option<TextureType>,
    border_color: Option<RGBA>,
}

impl_texture_trait!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_equality() {
        // Test equality with all fields identical
        let mut texture1 = OwnedTexture::new("textures/facade.jpg".to_string(), ImageType::Jpg);
        texture1.set_wrap_mode(Some(WrapMode::Mirror));
        texture1.set_texture_type(Some(TextureType::Specific));
        texture1.set_border_color(Some([0.0, 0.0, 0.0, 1.0]));

        let mut texture2 = OwnedTexture::new("textures/facade.jpg".to_string(), ImageType::Jpg);
        texture2.set_wrap_mode(Some(WrapMode::Mirror));
        texture2.set_texture_type(Some(TextureType::Specific));
        texture2.set_border_color(Some([0.0, 0.0, 0.0, 1.0]));

        assert_eq!(texture1, texture2);

        // Test inequality with different image
        let texture3 = OwnedTexture::new("textures/roof.jpg".to_string(), ImageType::Jpg);
        assert_ne!(texture1, texture3);

        // Test inequality with different image_type
        let mut texture4 = OwnedTexture::new("textures/facade.jpg".to_string(), ImageType::Png);
        texture4.set_wrap_mode(Some(WrapMode::Mirror));
        texture4.set_texture_type(Some(TextureType::Specific));
        texture4.set_border_color(Some([0.0, 0.0, 0.0, 1.0]));
        assert_ne!(texture1, texture4);

        // Test inequality with different wrap_mode
        let mut texture5 = OwnedTexture::new("textures/facade.jpg".to_string(), ImageType::Jpg);
        texture5.set_wrap_mode(Some(WrapMode::Wrap));
        texture5.set_texture_type(Some(TextureType::Specific));
        texture5.set_border_color(Some([0.0, 0.0, 0.0, 1.0]));
        assert_ne!(texture1, texture5);

        // Test inequality with different texture_type
        let mut texture6 = OwnedTexture::new("textures/facade.jpg".to_string(), ImageType::Jpg);
        texture6.set_wrap_mode(Some(WrapMode::Mirror));
        texture6.set_texture_type(Some(TextureType::Typical));
        texture6.set_border_color(Some([0.0, 0.0, 0.0, 1.0]));
        assert_ne!(texture1, texture6);

        // Test inequality with different border_color
        let mut texture7 = OwnedTexture::new("textures/facade.jpg".to_string(), ImageType::Jpg);
        texture7.set_wrap_mode(Some(WrapMode::Mirror));
        texture7.set_texture_type(Some(TextureType::Specific));
        texture7.set_border_color(Some([1.0, 1.0, 1.0, 1.0]));
        assert_ne!(texture1, texture7);
    }
}
