//! Surface material definitions for `CityJSON` v2.0.
//!
//! A [`Material`] is a named set of rendering properties. Materials are stored in the model's
//! material pool and referenced from geometry by [`MaterialHandle`] via a theme map.
//!
//! All color values are in the range 0.0–1.0. Colors are represented as [`RGB`] (`[R, G, B]`).
//! Transparency is 0.0 = fully opaque, 1.0 = fully transparent.
//!
//! Spec: [Material Object](https://www.cityjson.org/specs/2.0.1/#material-object).
//!
//! [`MaterialHandle`]: crate::resources::handles::MaterialHandle
//!
//! ```rust
//! use cityjson_types::CityModelType;
//! use cityjson_types::v2_0::{OwnedCityModel, RGB};
//! use cityjson_types::v2_0::appearance::material::OwnedMaterial;
//!
//! let mut model = OwnedCityModel::new(CityModelType::CityJSON);
//!
//! let mut mat = OwnedMaterial::new("roof-tiles".to_string());
//! mat.set_diffuse_color(Some(RGB::new(0.8, 0.3, 0.1)));
//! mat.set_shininess(Some(0.2));
//! mat.set_transparency(Some(0.0));
//!
//! let handle = model.add_material(mat).unwrap();
//! assert!(model.get_material(handle).is_some());
//! ```
//!
//! ```rust
//! use cityjson_types::v2_0::appearance::material::OwnedMaterial;
//! use cityjson_types::v2_0::appearance::RGB;
//!
//! let mut material = OwnedMaterial::new("Roof".to_string());
//!
//! let diffuse = RGB::new(0.8, 0.2, 0.1);
//! let raw_diffuse = diffuse.to_array();
//! assert_eq!(raw_diffuse, [0.8, 0.2, 0.1]);
//!
//! let specular = RGB::from([0.9, 0.9, 0.9]);
//! let raw_specular: [f32; 3] = specular.into();
//! assert_eq!(raw_specular, [0.9, 0.9, 0.9]);
//!
//! material.set_diffuse_color(Some(diffuse));
//! material.set_specular_color(Some(specular));
//!
//! assert_eq!(material.diffuse_color(), Some(RGB::from([0.8, 0.2, 0.1])));
//! assert_eq!(material.specular_color(), Some(RGB::from([0.9, 0.9, 0.9])));
//! ```

use crate::format_option;
use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};
use crate::v2_0::appearance::RGB;
use std::fmt::{Display, Formatter};

pub type OwnedMaterial = Material<OwnedStringStorage>;
pub type BorrowedMaterial<'a> = Material<BorrowedStringStorage<'a>>;

/// A surface material. See the [module docs](self) for usage.
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
        self.diffuse_color.as_ref().copied()
    }
    #[inline]
    pub fn set_diffuse_color(&mut self, diffuse_color: Option<RGB>) {
        self.diffuse_color = diffuse_color;
    }
    #[inline]
    pub fn emissive_color(&self) -> Option<RGB> {
        self.emissive_color
    }
    #[inline]
    pub fn set_emissive_color(&mut self, emissive_color: Option<RGB>) {
        self.emissive_color = emissive_color;
    }
    #[inline]
    pub fn specular_color(&self) -> Option<RGB> {
        self.specular_color
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

impl<SS: StringStorage> Display for Material<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "name: {:?}, ambient_intensity: {}, diffuse_color: {}, emissive_color: {}, specular_color: {}, shininess: {}, transparency: {}, is_smooth: {}",
            self.name,
            format_option(self.ambient_intensity.as_ref()),
            format_option(self.diffuse_color.as_ref()),
            format_option(self.emissive_color.as_ref()),
            format_option(self.specular_color.as_ref()),
            format_option(self.shininess.as_ref()),
            format_option(self.transparency.as_ref()),
            format_option(self.is_smooth.as_ref())
        )
    }
}
