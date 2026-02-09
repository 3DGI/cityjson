use crate::cityjson::core::transform;
use std::fmt::{Display, Formatter};

#[repr(transparent)]
#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct Transform(transform::TransformCore);

impl Display for Transform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_inner())
    }
}

impl Transform {
    #[must_use] 
    pub fn new() -> Self {
        Self(transform::TransformCore::new())
    }
    #[must_use] 
    pub fn scale(&self) -> [f64; 3] {
        self.0.scale()
    }
    #[must_use] 
    pub fn translate(&self) -> [f64; 3] {
        self.0.translate()
    }
    pub fn set_scale(&mut self, scale: [f64; 3]) {
        self.0.set_scale(scale);
    }
    pub fn set_translate(&mut self, translate: [f64; 3]) {
        self.0.set_translate(translate);
    }

    pub(crate) fn as_inner(&self) -> &transform::TransformCore {
        &self.0
    }
    #[allow(unused)]
    pub(crate) fn as_inner_mut(&mut self) -> &mut transform::TransformCore {
        &mut self.0
    }
}
