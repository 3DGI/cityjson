//! # Material
//!
//! Represents a [Material object](https://www.cityjson.org/specs/1.1.3/#material-object).
use crate::common::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};

pub type OwnedMaterial = Material<OwnedStringStorage>;
pub type BorrowedMaterial<'a> = Material<BorrowedStringStorage<'a>>;

pub type RGB = [f32; 3];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Material<S: StringStorage> {
    name: S::String,
    ambient_intensity: Option<f32>,
    diffuse_color: Option<RGB>,
    emissive_color: Option<RGB>,
    specular_color: Option<RGB>,
    shininess: Option<f32>,
    transparency: Option<f32>,
    is_smooth: Option<bool>,
}

impl<S: StringStorage> Material<S> {
    pub fn new(name: S::String) -> Self {
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
    pub fn name(&self) -> &S::String {
        &self.name
    }

    #[inline]
    pub fn set_name(&mut self, name: S::String) {
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
    pub fn diffuse_color(&self) -> Option<&RGB> {
        self.diffuse_color.as_ref()
    }

    #[inline]
    pub fn set_diffuse_color(&mut self, diffuse_color: Option<RGB>) {
        self.diffuse_color = diffuse_color;
    }

    #[inline]
    pub fn emissive_color(&self) -> Option<&RGB> {
        self.emissive_color.as_ref()
    }

    #[inline]
    pub fn set_emissive_color(&mut self, emissive_color: Option<RGB>) {
        self.emissive_color = emissive_color;
    }

    #[inline]
    pub fn specular_color(&self) -> Option<&RGB> {
        self.specular_color.as_ref()
    }

    #[inline]
    pub fn set_specular_color(&mut self, specular_color: Option<RGB>) {
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
    fn test_owned_material() {
        let mat = OwnedMaterial::new("brick".to_string());
        assert_eq!(mat.name, "brick");
    }

    #[test]
    fn test_borrowed_material() {
        let name = "brick";
        let mat = BorrowedMaterial::new(name);
        assert_eq!(mat.name, "brick");
    }
}
