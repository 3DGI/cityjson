//! Tests for appearance-related functionality.

use cityjson::v2_0::{ImageType, OwnedMaterial, OwnedTexture, TextureType, WrapMode};

/// Two identical materials should be considered equal.
#[test]
fn material_equality() {
    // Create two materials with identical properties
    let mut material1 = OwnedMaterial::new("TestMaterial".to_string());
    material1.set_ambient_intensity(Some(0.5));
    material1.set_diffuse_color(Some([0.8, 0.7, 0.6].into()));
    material1.set_emissive_color(Some([0.1, 0.2, 0.3].into()));
    material1.set_specular_color(Some([1.0, 1.0, 1.0].into()));
    material1.set_shininess(Some(0.9));
    material1.set_transparency(Some(0.0));
    material1.set_is_smooth(Some(true));

    let mut material2 = OwnedMaterial::new("TestMaterial".to_string());
    material2.set_ambient_intensity(Some(0.5));
    material2.set_diffuse_color(Some([0.8, 0.7, 0.6].into()));
    material2.set_emissive_color(Some([0.1, 0.2, 0.3].into()));
    material2.set_specular_color(Some([1.0, 1.0, 1.0].into()));
    material2.set_shininess(Some(0.9));
    material2.set_transparency(Some(0.0));
    material2.set_is_smooth(Some(true));

    // Test equality - all fields are equal
    assert_eq!(material1, material2);

    // Test inequality - change one field
    material2.set_diffuse_color(Some([0.9, 0.8, 0.7].into()));
    assert_ne!(material1, material2);
}

/// Two textures with identical properties should be considered equal.
#[test]
fn texture_equality() {
    // Test equality with all fields identical
    let mut texture1 = OwnedTexture::new("textures/facade.jpg".to_string(), ImageType::Jpg);
    texture1.set_wrap_mode(Some(WrapMode::Mirror));
    texture1.set_texture_type(Some(TextureType::Specific));
    texture1.set_border_color(Some([0.0, 0.0, 0.0, 1.0].into()));

    let mut texture2 = OwnedTexture::new("textures/facade.jpg".to_string(), ImageType::Jpg);
    texture2.set_wrap_mode(Some(WrapMode::Mirror));
    texture2.set_texture_type(Some(TextureType::Specific));
    texture2.set_border_color(Some([0.0, 0.0, 0.0, 1.0].into()));

    assert_eq!(texture1, texture2);

    // Test inequality with different image
    let texture3 = OwnedTexture::new("textures/roof.jpg".to_string(), ImageType::Jpg);
    assert_ne!(texture1, texture3);

    // Test inequality with different image_type
    let mut texture4 = OwnedTexture::new("textures/facade.jpg".to_string(), ImageType::Png);
    texture4.set_wrap_mode(Some(WrapMode::Mirror));
    texture4.set_texture_type(Some(TextureType::Specific));
    texture4.set_border_color(Some([0.0, 0.0, 0.0, 1.0].into()));
    assert_ne!(texture1, texture4);

    // Test inequality with different wrap_mode
    let mut texture5 = OwnedTexture::new("textures/facade.jpg".to_string(), ImageType::Jpg);
    texture5.set_wrap_mode(Some(WrapMode::Wrap));
    texture5.set_texture_type(Some(TextureType::Specific));
    texture5.set_border_color(Some([0.0, 0.0, 0.0, 1.0].into()));
    assert_ne!(texture1, texture5);

    // Test inequality with different texture_type
    let mut texture6 = OwnedTexture::new("textures/facade.jpg".to_string(), ImageType::Jpg);
    texture6.set_wrap_mode(Some(WrapMode::Mirror));
    texture6.set_texture_type(Some(TextureType::Typical));
    texture6.set_border_color(Some([0.0, 0.0, 0.0, 1.0].into()));
    assert_ne!(texture1, texture6);

    // Test inequality with different border_color
    let mut texture7 = OwnedTexture::new("textures/facade.jpg".to_string(), ImageType::Jpg);
    texture7.set_wrap_mode(Some(WrapMode::Mirror));
    texture7.set_texture_type(Some(TextureType::Specific));
    texture7.set_border_color(Some([1.0, 1.0, 1.0, 1.0].into()));
    assert_ne!(texture1, texture7);
}
