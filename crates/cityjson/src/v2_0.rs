//! `CityJSON` v2.0 types.
//!
//! The types map directly to the [CityJSON 2.0 spec](https://www.cityjson.org/specs/2.0.1/).
//!
//! | Spec concept | Rust type |
//! |---|---|
//! | Root `CityJSON` object | [`CityModel`] |
//! | `CityObjects` map entry | [`CityObject`] |
//! | Geometry (stored) | [`Geometry`] |
//! | Geometry (authoring) | [`GeometryDraft`] |
//! | Semantic surface | [`Semantic`], [`SemanticType`] |
//! | Material / Texture resource | [`Material`], [`Texture`] (registered in model pools) |
//! | Appearance assignment | [`MaterialMap`], [`TextureMap`] (per geometry, per theme) |
//! | Coordinate transform | [`Transform`] |
//! | Metadata | [`Metadata`] |
//!
//! ## Differences from the JSON representation
//!
//! - **Coordinates are floats, not integers.** The `transform` object (spec §4) is a
//!   serialization concern. Internally all vertices are `f64` real-world coordinates.
//! - **Boundaries are flat.** The nested JSON arrays (spec §3.2) are stored as flat `Vec`s
//!   with offset counters. Use `Boundary::to_nested_*` or the [`raw`](crate::raw) module to
//!   recover the nested form for serialization.
//! - **Resources live in model-level pools.** Semantics, materials, and textures are stored
//!   once on [`CityModel`] and referenced by typed handles ([`SemanticHandle`],
//!   [`MaterialHandle`], [`TextureHandle`]).
//! - **Geometry authoring uses a draft.** [`GeometryDraft`] accepts raw coordinates,
//!   deduplicates vertices, validates the geometry, and inserts it into the model in one step.
//!
//! ## Imports
//!
//! ```rust
//! use cityjson::v2_0::*;     // all domain types
//! use cityjson::prelude::*;  // handles, storage strategies, error types
//! ```
//!
//! [`SemanticHandle`]: crate::resources::handles::SemanticHandle
//! [`MaterialHandle`]: crate::resources::handles::MaterialHandle
//! [`TextureHandle`]: crate::resources::handles::TextureHandle
//! [`MaterialMap`]: crate::resources::mapping::materials::MaterialMap
//! [`TextureMap`]: crate::resources::mapping::textures::TextureMap
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
    UvDraft, DraftVertex, GeometryDraft, LineStringDraft, PointDraft, RingDraft, ShellDraft,
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
