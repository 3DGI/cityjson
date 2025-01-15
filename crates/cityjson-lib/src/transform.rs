use serde_cityjson::v1_1;
use std::fmt;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Transform {
    scale: [f64; 3],
    translate: [f64; 3],
}

impl Transform {
    pub fn new(scale: [f64; 3], translate: [f64; 3]) -> Self {
        Self { scale, translate }
    }

    pub fn scale(&self) -> &[f64; 3] {
        &self.scale
    }

    pub fn translate(&self) -> &[f64; 3] {
        &self.translate
    }

    pub fn set_scale(&mut self, scale: [f64; 3]) {
        self.scale = scale;
    }

    pub fn set_translate(&mut self, translate: [f64; 3]) {
        self.translate = translate;
    }
}

impl From<v1_1::Transform> for Transform {
    fn from(transform: v1_1::Transform) -> Self {
        Self {
            scale: transform.scale,
            translate: transform.translate,
        }
    }
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transform(scale: {:?}, translate: {:?})",
            self.scale, self.translate
        )
    }
}
