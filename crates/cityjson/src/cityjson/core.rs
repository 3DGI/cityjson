//! Curated core `CityJSON` modules.

pub mod appearance {
    pub use crate::backend::default::appearance::{ImageType, TextureType, WrapMode};
}

pub mod attributes {
    pub use crate::backend::default::attributes::*;
}

pub mod boundary {
    pub use crate::backend::default::boundary::*;
}

pub(crate) mod citymodel {
    pub use crate::backend::default::citymodel::*;
}

pub(crate) mod cityobject {
    pub use crate::backend::default::cityobject::*;
}

pub mod coordinate {
    pub use crate::backend::default::coordinate::*;
}

pub mod extension {
    pub use crate::backend::default::extension::*;
}

pub mod geometry {
    pub use crate::backend::default::geometry::{BuilderMode, GeometryType, LoD};
}

pub(crate) mod geometry_struct {
    pub use crate::backend::default::geometry_struct::*;
}

pub mod metadata {
    pub use crate::backend::default::metadata::*;
}

pub mod transform {
    pub use crate::backend::default::transform::*;
}

pub mod vertex {
    pub use crate::backend::default::vertex::*;
}
