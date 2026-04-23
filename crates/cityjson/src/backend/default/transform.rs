use std::fmt::{Display, Formatter};

/// Core transform type that is wrapped by versioned implementations
/// ([`crate::v2_0::Transform`]).
/// Core types are expected to remain stable across several versions.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(crate) struct TransformCore {
    scale: [f64; 3],
    translate: [f64; 3],
}

impl TransformCore {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub(crate) fn scale(&self) -> [f64; 3] {
        self.scale
    }
    #[must_use]
    pub(crate) fn translate(&self) -> [f64; 3] {
        self.translate
    }
    pub(crate) fn set_scale(&mut self, scale: [f64; 3]) {
        self.scale = scale;
    }
    pub(crate) fn set_translate(&mut self, translate: [f64; 3]) {
        self.translate = translate;
    }
}

impl Display for TransformCore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "scale: [{}, {}, {}], translate:[{}, {}, {}]",
            self.scale[0],
            self.scale[1],
            self.scale[2],
            self.translate[0],
            self.translate[1],
            self.translate[2]
        )
    }
}

impl Default for TransformCore {
    fn default() -> Self {
        Self {
            scale: [1.0, 1.0, 1.0],
            translate: [0.0, 0.0, 0.0],
        }
    }
}
