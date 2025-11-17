//! Boundary types for the nested backend.
//!
//! TODO: Implement nested backend boundary types.

use crate::prelude::RealWorldCoordinate;

#[derive(Clone, Debug, PartialEq)]
pub enum Boundary {
    MultiPoint(BoundaryMultiPoint),
    MultiLineString(BoundaryMultiLineString),
    MultiSurface(BoundaryMultiOrCompositeSurface),
    CompositeSurface(BoundaryMultiOrCompositeSurface),
    Solid(BoundarySolid),
    MultiSolid(BoundaryMultiOrCompositeSolid),
    CompositeSolid(BoundaryMultiOrCompositeSolid),
}

pub type BoundaryMultiPoint = Vec<RealWorldCoordinate>;
pub type BoundaryMultiLineString = Vec<BoundaryMultiPoint>;
pub type BoundaryMultiOrCompositeSurface = Vec<BoundaryMultiLineString>;

pub type BoundarySolid = Vec<BoundaryMultiOrCompositeSurface>;

pub type BoundaryMultiOrCompositeSolid = Vec<BoundarySolid>;
