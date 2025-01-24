use std::borrow::Cow;

pub type Rgb = [f32; 3];

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Material<'cm> {
    pub name: Cow<'cm, str>,
    pub ambient_intensity: Option<f32>,
    pub diffuse_color: Option<Rgb>,
    pub emissive_color: Option<Rgb>,
    pub specular_color: Option<Rgb>,
    pub shininess: Option<f32>,
    pub transparency: Option<f32>,
    pub is_smooth: Option<bool>,
}