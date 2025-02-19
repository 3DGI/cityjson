use crate::common::storage::StringStorage;

pub trait Material<S: StringStorage> {
    fn new(name: S::String) -> Self;
    fn name(&self) -> &S::String;
    fn set_name(&mut self, name: S::String);
    fn ambient_intensity(&self) -> Option<f32>;
    fn set_ambient_intensity(&mut self, ambient_intensity: Option<f32>);
    fn diffuse_color(&self) -> Option<&RGB>;
    fn set_diffuse_color(&mut self, diffuse_color: Option<RGB>);
    fn emissive_color(&self) -> Option<&RGB>;
    fn set_emissive_color(&mut self, emissive_color: Option<RGB>);
    fn specular_color(&self) -> Option<&RGB>;
    fn set_specular_color(&mut self, specular_color: Option<RGB>);
    fn shininess(&self) -> Option<f32>;
    fn set_shininess(&mut self, shininess: Option<f32>);
    fn transparency(&self) -> Option<f32>;
    fn set_transparency(&mut self, transparency: Option<f32>);
    fn is_smooth(&self) -> Option<bool>;
    fn set_is_smooth(&mut self, is_smooth: Option<bool>);
}

pub type RGB = [f32; 3];