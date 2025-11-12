//! # Geometry
//!
//! Represents a [Geometry object](https://www.cityjson.org/specs/1.1.3/#geometry-objects).
use crate::cityjson::core::geometry_struct::GeometryCore;
use crate::cityjson::core::vertex::VertexRef;
use crate::prelude::StringStorage;
use crate::resources::pool::ResourceRef;

pub mod semantic;

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Geometry<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    inner: GeometryCore<VR, RR, SS>,
}

crate::macros::impl_geometry_methods!();
