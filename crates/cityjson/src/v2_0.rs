//! `CityJSON` v2.0 types.
//!
//! This module mirrors the `CityJSON` 2.0 data model. If you know the spec, the types map
//! directly: [`CityModel`] is the root object, [`CityObject`] is each
//! entry in the `CityObjects` map, and [`Geometry`] covers all eight geometry types.
//!
//! A few things differ from the JSON representation:
//!
//! - **Vertices are floats.** The transform (scale + translate) is a serialization concern;
//!   internally all coordinates are `f64` real-world values.
//! - **Boundaries are flat.** The nested JSON arrays are stored as flat vectors with offset
//!   counters. Use [`Boundary::to_nested_*`](boundary::Boundary) methods or the `raw` module
//!   to get the nested form for serialization.
//! - **Semantics, materials, and textures live in global pools.** Instead of inline objects,
//!   the model holds one pool per resource type. References in geometry are typed handles
//!   ([`SemanticHandle`], [`MaterialHandle`], [`TextureHandle`]) returned when you add a
//!   resource to the model.
//! - **Geometry authoring uses a draft API.** [`GeometryDraft`] lets you build geometries from
//!   raw coordinates. It deduplicates vertices, validates geometry, and inserts in one step.
//!
//! The typical import:
//!
//! ```rust
//! use cityjson::v2_0::*;
//! use cityjson::prelude::*;
//! ```
//!
//! [`SemanticHandle`]: crate::resources::handles::SemanticHandle
//! [`MaterialHandle`]: crate::resources::handles::MaterialHandle
//! [`TextureHandle`]: crate::resources::handles::TextureHandle
pub use crate::{CityJSONVersion, CityModelType};

pub mod appearance;
pub mod attributes {
    pub use crate::cityjson::core::attributes::{
        AttributeValue, Attributes, BorrowedAttributeValue, BorrowedAttributes,
        OwnedAttributeValue, OwnedAttributes,
    };
}
pub mod boundary {
    pub use crate::cityjson::core::boundary::nested;
    pub use crate::cityjson::core::boundary::{
        Boundary, Boundary16, Boundary32, Boundary64, BoundaryType,
    };
}
pub mod citymodel;
pub mod cityobject;
pub mod coordinate {
    pub use crate::cityjson::core::coordinate::{Coordinate, RealWorldCoordinate, UVCoordinate};
}
pub mod extension;
pub mod geometry;
pub mod geometry_draft;
pub mod metadata;
pub mod transform;
pub mod vertex {
    pub use crate::cityjson::core::vertex::{
        RawVertexView, VertexIndex, VertexIndex16, VertexIndex32, VertexIndex64, VertexIndexVec,
        VertexIndicesSequence, VertexRef,
    };
}
pub mod vertices {
    pub use crate::cityjson::core::vertices::{
        GeometryVertices16, GeometryVertices32, GeometryVertices64, UVVertices16, UVVertices32,
        UVVertices64, Vertices,
    };
}

pub use appearance::{
    ImageType, RGB, RGBA, TextureType, ThemeName, WrapMode,
    material::{BorrowedMaterial, Material, OwnedMaterial},
    texture::{BorrowedTexture, OwnedTexture, Texture},
};
pub use attributes::{
    AttributeValue, Attributes, BorrowedAttributeValue, BorrowedAttributes, OwnedAttributeValue,
    OwnedAttributes,
};
pub use boundary::{
    Boundary, Boundary16, Boundary32, Boundary64, BoundaryType,
    nested::{
        BoundaryNestedMultiLineString, BoundaryNestedMultiLineString16,
        BoundaryNestedMultiLineString32, BoundaryNestedMultiLineString64,
        BoundaryNestedMultiOrCompositeSolid, BoundaryNestedMultiOrCompositeSolid16,
        BoundaryNestedMultiOrCompositeSolid32, BoundaryNestedMultiOrCompositeSolid64,
        BoundaryNestedMultiOrCompositeSurface, BoundaryNestedMultiOrCompositeSurface16,
        BoundaryNestedMultiOrCompositeSurface32, BoundaryNestedMultiOrCompositeSurface64,
        BoundaryNestedMultiPoint, BoundaryNestedMultiPoint16, BoundaryNestedMultiPoint32,
        BoundaryNestedMultiPoint64, BoundaryNestedSolid, BoundaryNestedSolid16,
        BoundaryNestedSolid32, BoundaryNestedSolid64,
    },
};
pub use citymodel::{BorrowedCityModel, CityModel, CityModelCapacities, OwnedCityModel};
pub use cityobject::{
    BorrowedCityObjects, CityObject, CityObjectIdentifier, CityObjectType, CityObjects,
    OwnedCityObjects,
};
pub use coordinate::{Coordinate, RealWorldCoordinate, UVCoordinate};
pub use extension::{Extension, Extensions};
pub use geometry::{
    AffineTransform3D, Geometry, GeometryInstanceView, GeometryType, GeometryView, LoD,
    semantic::{BorrowedSemantic, OwnedSemantic, Semantic, SemanticType},
};
pub use geometry_draft::{
    DraftUv, DraftVertex, GeometryDraft, LineStringDraft, PointDraft, RingDraft, ShellDraft,
    SolidDraft, SurfaceDraft,
};
pub use metadata::{
    BBox, CRS, CityModelIdentifier, Contact, ContactRole, ContactType, Date, Metadata,
};
pub use transform::Transform;
pub use vertex::{
    RawVertexView, VertexIndex, VertexIndex16, VertexIndex32, VertexIndex64, VertexIndexVec,
    VertexIndicesSequence, VertexRef,
};
pub use vertices::{
    GeometryVertices16, GeometryVertices32, GeometryVertices64, UVVertices16, UVVertices32,
    UVVertices64, Vertices,
};
