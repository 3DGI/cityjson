use crate::common::boundary::BoundaryType;
use crate::common::index::{OptionalVertexIndices, VertexIndices, VertexInteger};
use std::fmt::Debug;

/// Stores the Semantic and Material indices of a Boundary and maps them to the
/// boundary primitives.
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct SemanticMaterialMap<T: VertexInteger> {
    /// Each item corresponds to the point with the same index in a MultiPoint boundary, the value
    /// of the item is the index of the Semantic or Material object.
    pub points: OptionalVertexIndices<T>,
    /// Each item corresponds to the linestring with the same index in a MultiLineString boundary,
    /// the value of the item is the index of the Semantic or Material object.
    pub linestrings: OptionalVertexIndices<T>,
    /// Each item corresponds to the surface with the same index, the value
    /// of the item is the index of the Semantic or Material object.
    pub surfaces: OptionalVertexIndices<T>,
    pub shells: VertexIndices<T>,
    pub solids: VertexIndices<T>,
}

impl<T: VertexInteger> SemanticMaterialMap<T> {
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
