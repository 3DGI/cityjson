use crate::common::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};

pub type OwnedMaterial = Material<OwnedStringStorage>;
pub type BorrowedMaterial<'a> = Material<BorrowedStringStorage<'a>>;

pub type RGB = [f32; 3];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Material<S: StringStorage> {
    pub name: S::String,
    pub ambient_intensity: Option<f32>,
    pub diffuse_color: Option<RGB>,
    pub emissive_color: Option<RGB>,
    pub specular_color: Option<RGB>,
    pub shininess: Option<f32>,
    pub transparency: Option<f32>,
    pub is_smooth: Option<bool>,
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
