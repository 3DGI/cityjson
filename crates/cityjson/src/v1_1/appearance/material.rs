//! # Material
//!
//! Represents a [Material object](https://www.cityjson.org/specs/1.1.3/#material-object).

use crate::cityjson::appearance::material::RGB;
use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};

pub type OwnedMaterial = Material<OwnedStringStorage>;
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

impl<SS: StringStorage> crate::cityjson::appearance::material::Material<SS> for Material<SS> {
    fn new(name: SS::String) -> Self {
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
    fn name(&self) -> &SS::String {
        &self.name
    }
    #[inline]
    fn set_name(&mut self, name: SS::String) {
        self.name = name;
    }
    #[inline]
    fn ambient_intensity(&self) -> Option<f32> {
        self.ambient_intensity
    }
    #[inline]
    fn set_ambient_intensity(&mut self, ambient_intensity: Option<f32>) {
        self.ambient_intensity = ambient_intensity;
    }
    #[inline]
    fn diffuse_color(&self) -> Option<&RGB> {
        self.diffuse_color.as_ref()
    }
    #[inline]
    fn set_diffuse_color(&mut self, diffuse_color: Option<RGB>) {
        self.diffuse_color = diffuse_color;
    }
    #[inline]
    fn emissive_color(&self) -> Option<&RGB> {
        self.emissive_color.as_ref()
    }
    #[inline]
    fn set_emissive_color(&mut self, emissive_color: Option<RGB>) {
        self.emissive_color = emissive_color;
    }
    #[inline]
    fn specular_color(&self) -> Option<&RGB> {
        self.specular_color.as_ref()
    }
    #[inline]
    fn set_specular_color(&mut self, specular_color: Option<RGB>) {
        self.specular_color = specular_color;
    }
    #[inline]
    fn shininess(&self) -> Option<f32> {
        self.shininess
    }
    #[inline]
    fn set_shininess(&mut self, shininess: Option<f32>) {
        self.shininess = shininess;
    }
    #[inline]
    fn transparency(&self) -> Option<f32> {
        self.transparency
    }
    #[inline]
    fn set_transparency(&mut self, transparency: Option<f32>) {
        self.transparency = transparency;
    }
    #[inline]
    fn is_smooth(&self) -> Option<bool> {
        self.is_smooth
    }
    #[inline]
    fn set_is_smooth(&mut self, is_smooth: Option<bool>) {
        self.is_smooth = is_smooth;
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::cityjson::material::Material;
//
//     #[test]
//     fn test_owned_material() {
//         let mat = OwnedMaterial::new("brick".to_string());
//         assert_eq!(mat.name, "brick");
//     }
//
//     #[test]
//     fn test_borrowed_material() {
//         let name = "brick";
//         let mat = BorrowedMaterial::new(name);
//         assert_eq!(mat.name, "brick");
//     }
// }
