use crate::cityjson::core::appearance::RGB;
use crate::macros::impl_material_trait;
use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};

/// Type alias for a material with owned string storage
pub type OwnedMaterial = Material<OwnedStringStorage>;

/// Type alias for a material with borrowed string storage
pub type BorrowedMaterial<'a> = Material<BorrowedStringStorage<'a>>;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Material<SS: StringStorage> {
    name: SS::String,
    ambient_intensity: Option<f32>,
    diffuse_color: Option<RGB>,
    emissive_color: Option<RGB>,
    specular_color: Option<RGB>,
    shininess: Option<f32>,
    transparency: Option<f32>,
    is_smooth: Option<bool>,
}

impl_material_trait!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_equality() {
        // Create two materials with identical properties
        let mut material1 = OwnedMaterial::new("TestMaterial".to_string());
        material1.set_ambient_intensity(Some(0.5));
        material1.set_diffuse_color(Some([0.8, 0.7, 0.6]));
        material1.set_emissive_color(Some([0.1, 0.2, 0.3]));
        material1.set_specular_color(Some([1.0, 1.0, 1.0]));
        material1.set_shininess(Some(0.9));
        material1.set_transparency(Some(0.0));
        material1.set_is_smooth(Some(true));

        let mut material2 = OwnedMaterial::new("TestMaterial".to_string());
        material2.set_ambient_intensity(Some(0.5));
        material2.set_diffuse_color(Some([0.8, 0.7, 0.6]));
        material2.set_emissive_color(Some([0.1, 0.2, 0.3]));
        material2.set_specular_color(Some([1.0, 1.0, 1.0]));
        material2.set_shininess(Some(0.9));
        material2.set_transparency(Some(0.0));
        material2.set_is_smooth(Some(true));

        // Test equality - all fields are equal
        assert_eq!(material1, material2);

        // Test inequality - change one field
        material2.set_diffuse_color(Some([0.9, 0.8, 0.7]));
        assert_ne!(material1, material2);
    }
}
