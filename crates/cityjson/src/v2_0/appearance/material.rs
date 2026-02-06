use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};
use crate::v2_0::types::RGB;

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

impl<SS: StringStorage> Material<SS> {
    pub fn new(name: SS::String) -> Self {
        Self {
            name,
            ambient_intensity: None,
            diffuse_color: None,
            emissive_color: None,
            specular_color: None,
            shininess: None,
            transparency: None,
            is_smooth: None,
        }
    }
    #[inline]
    pub fn name(&self) -> &SS::String {
        &self.name
    }
    #[inline]
    pub fn set_name(&mut self, name: SS::String) {
        self.name = name;
    }
    #[inline]
    pub fn ambient_intensity(&self) -> Option<f32> {
        self.ambient_intensity
    }
    #[inline]
    pub fn set_ambient_intensity(&mut self, ambient_intensity: Option<f32>) {
        self.ambient_intensity = ambient_intensity;
    }
    #[inline]
    pub fn diffuse_color(&self) -> Option<RGB> {
        self.diffuse_color.as_ref()
            .copied()
    }
    #[inline]
    pub fn set_diffuse_color(
        &mut self,
        diffuse_color: Option<RGB>,
    ) {
        self.diffuse_color = diffuse_color;
    }
    #[inline]
    pub fn emissive_color(&self) -> Option<RGB> {
        self.emissive_color
    }
    #[inline]
    pub fn set_emissive_color(
        &mut self,
        emissive_color: Option<RGB>,
    ) {
        self.emissive_color = emissive_color;
    }
    #[inline]
    pub fn specular_color(&self) -> Option<RGB> {
        self.specular_color
    }
    #[inline]
    pub fn set_specular_color(
        &mut self,
        specular_color: Option<RGB>,
    ) {
        self.specular_color = specular_color;
    }
    #[inline]
    pub fn shininess(&self) -> Option<f32> {
        self.shininess
    }
    #[inline]
    pub fn set_shininess(&mut self, shininess: Option<f32>) {
        self.shininess = shininess;
    }
    #[inline]
    pub fn transparency(&self) -> Option<f32> {
        self.transparency
    }
    #[inline]
    pub fn set_transparency(&mut self, transparency: Option<f32>) {
        self.transparency = transparency;
    }
    #[inline]
    pub fn is_smooth(&self) -> Option<bool> {
        self.is_smooth
    }
    #[inline]
    pub fn set_is_smooth(&mut self, is_smooth: Option<bool>) {
        self.is_smooth = is_smooth;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_equality() {
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
}
