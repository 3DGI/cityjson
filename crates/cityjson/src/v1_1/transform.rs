use std::fmt::{Display, Formatter};
use crate::cityjson::core;
use crate::cityjson::traits::transform::TransformTrait;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Transform{
    inner: core::transform::TransformCore,
}

impl TransformTrait for Transform {
    fn new() -> Self {
        Self {
            inner: core::transform::TransformCore::new(),
        }
    }

    fn scale(&self) -> [f64; 3] {
        self.inner.scale()
    }

    fn translate(&self) -> [f64; 3] {
        self.inner.translate()
    }

    fn set_scale(&mut self, scale: [f64; 3]) {
        self.inner.set_scale(scale);
    }

    fn set_translate(&mut self, translate: [f64; 3]) {
        self.inner.set_translate(translate);
    }
}

impl Display for Transform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}