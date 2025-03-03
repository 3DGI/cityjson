//! # Material
//!
//! Represents a [Material object](https://www.cityjson.org/specs/1.1.3/#material-object).

use crate::cityjson::appearance::material::MaterialTrait;
use crate::cityjson::appearance::RGB;
use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};

/// Type alias for a material with owned string storage
pub type OwnedMaterial = Material<OwnedStringStorage>;

/// Type alias for a material with borrowed string storage
pub type BorrowedMaterial<'a> = Material<BorrowedStringStorage<'a>>;

/// A structure representing a material in CityJSON.
///
/// Materials define the visual appearance properties of surfaces in a 3D city model.
/// This implementation supports all material properties defined in the
/// [CityJSON 1.1.3 specification](https://www.cityjson.org/specs/1.1.3/#material-object).
///
/// # Type Parameters
///
/// * `SS` - The string storage strategy (owned or borrowed)
///
/// # Examples
///
/// Creating a new material and setting its properties:
///
/// ```
/// use cityjson::v1_1::material::{Material};
/// use cityjson::cityjson::appearance::*;
/// use cityjson::cityjson::appearance::RGB;
/// use cityjson::resources::storage::OwnedStringStorage;
///
/// // Create a new material with a name
/// let mut material = Material::<OwnedStringStorage>::new("BuildingFacade".to_string());
///
/// // Set material properties
/// material.set_ambient_intensity(Some(0.5));
/// material.set_diffuse_color(Some([0.8, 0.8, 0.8]));
/// material.set_specular_color(Some([1.0, 1.0, 1.0]));
/// material.set_shininess(Some(0.2));
/// material.set_transparency(Some(0.0));
/// material.set_is_smooth(Some(true));
///
/// // Access material properties
/// assert_eq!(material.name(), "BuildingFacade");
/// assert_eq!(material.ambient_intensity(), Some(0.5));
/// assert_eq!(material.diffuse_color(), Some(&[0.8, 0.8, 0.8]));
/// assert_eq!(material.specular_color(), Some(&[1.0, 1.0, 1.0]));
/// assert_eq!(material.shininess(), Some(0.2));
/// assert_eq!(material.transparency(), Some(0.0));
/// assert_eq!(material.is_smooth(), Some(true));
/// ```
///
/// Using the `OwnedMaterial` type alias:
///
/// ```
/// use cityjson::cityjson::appearance::MaterialTrait;
/// use cityjson::v1_1::material::{OwnedMaterial};
///
/// let mut material = OwnedMaterial::new("Brick".to_string());
/// material.set_emissive_color(Some([0.1, 0.0, 0.0]));
///
/// assert_eq!(material.name(), "Brick");
/// assert_eq!(material.emissive_color(), Some(&[0.1, 0.0, 0.0]));
/// ```
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

impl<SS: StringStorage> MaterialTrait<SS> for Material<SS> {
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