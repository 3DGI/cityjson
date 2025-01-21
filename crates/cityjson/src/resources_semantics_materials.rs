use std::fmt::Debug;
use crate::boundary::BoundaryType;
use crate::indices::{GeometryIndices, OptionalGeometryIndices};

pub trait SemanticType{}

pub trait Semantic: Clone + Debug {
    fn type_semantic<ST: SemanticType>(&self) -> ST;

    /// Access to the Semantic attributes.
    fn attributes<A>(&self) -> Option<A>;

    /// Mutable access to the Semantic attributes.
    fn attributes_mut<A>(&mut self) -> &mut A;
}

pub struct Semantics<S: Semantic>(Vec<S>);

/// Stores the Semantic and Material indices of a Boundary and maps them to the
/// boundary primitives.
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct SemanticMaterialMap {
    /// Each item corresponds to the point with the same index in a MultiPoint boundary, the value
    /// of the item is the index of the Semantic or Material object.
    pub points: OptionalGeometryIndices,
    /// Each item corresponds to the linestring with the same index in a MultiLineString boundary,
    /// the value of the item is the index of the Semantic or Material object.
    pub linestrings: OptionalGeometryIndices,
    /// Each item corresponds to the surface with the same index, the value
    /// of the item is the index of the Semantic or Material object.
    pub surfaces: OptionalGeometryIndices,
    pub shells: GeometryIndices,
    pub solids: GeometryIndices,
}

impl SemanticMaterialMap {
    /// Hint what [BoundaryType] does the SemanticMaterialMap belong to.
    pub fn check_type(&self) -> BoundaryType {
        if !self.solids.is_empty() {
            BoundaryType::MultiOrCompositeSolid
        } else if !self.shells.is_empty() {
            BoundaryType::Solid
        } else if !self.surfaces.is_empty() {
            BoundaryType::MultiOrCompositeSurface
        } else if !self.linestrings.is_empty() {
            BoundaryType::MultiLineString
        } else if !self.points.is_empty() {
            BoundaryType::MultiPoint
        } else {
            BoundaryType::None
        }
    }
}