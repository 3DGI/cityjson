//! Curated core `CityJSON` modules.

pub mod appearance {
    pub use crate::backend::default::appearance::{ImageType, TextureType, WrapMode};
}

pub mod attributes {
    pub use crate::backend::default::attributes::{
        AttributeValue, AttributeValueType, Attributes, BorrowedAttributeValue, BorrowedAttributes,
        OwnedAttributeValue, OwnedAttributes,
    };
}

pub mod boundary {
    pub(crate) use crate::backend::default::boundary::BoundaryCounter;
    pub use crate::backend::default::boundary::nested;
    pub use crate::backend::default::boundary::{
        Boundary, Boundary16, Boundary32, Boundary64, BoundaryType,
    };
}

pub(crate) mod citymodel {
    pub use crate::backend::default::citymodel::*;
}

pub(crate) mod cityobject {
    pub use crate::backend::default::cityobject::*;
}

pub mod coordinate {
    pub use crate::backend::default::coordinate::{
        FlexibleCoordinate, GeometryVertices16, GeometryVertices32, GeometryVertices64,
        QuantizedCoordinate, RealWorldCoordinate, UVCoordinate, UVVertices16, UVVertices32,
        UVVertices64, Vertices,
    };
}

pub(crate) mod extension {
    pub(crate) use crate::backend::default::extension::{
        ExtensionCore, ExtensionItem, ExtensionsCore,
    };
}

pub mod geometry {
    pub use crate::backend::default::geometry::{BuilderMode, GeometryBuilder, GeometryType, LoD};
}

pub(crate) mod geometry_struct {
    pub use crate::backend::default::geometry_struct::*;
}

pub mod metadata {
    pub use crate::backend::default::metadata::{BBox, CRS, CityModelIdentifier, Date};
}

pub(crate) mod transform {
    pub(crate) use crate::backend::default::transform::TransformCore;
}

pub mod vertex {
    pub use crate::backend::default::vertex::{
        RawVertexView, VertexIndex, VertexIndex16, VertexIndex32, VertexIndex64, VertexIndexVec,
        VertexIndicesSequence, VertexRef,
    };
}
