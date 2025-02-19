//! # Resource mapping
pub mod materials;
pub mod semantics;
pub mod textures;

use crate::common::boundary::BoundaryType;
use crate::index::VertexRef;
pub use crate::resources::mapping::materials::MaterialMap;
pub use crate::resources::mapping::semantics::SemanticMap;
pub use crate::resources::mapping::textures::TextureMap;
use crate::resources::pool::ResourceRef;

/// Stores the Semantic or Material indices of a Boundary and maps them to the
/// boundary primitives.
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct SemanticOrMaterialMap<VR: VertexRef, RR: ResourceRef> {
    /// Each item corresponds to the point with the same index in a MultiPoint boundary, the value
    /// of the item is the index of the Semantic or Material object.
    pub(crate) points: Vec<Option<RR>>,
    /// Each item corresponds to the linestring with the same index in a MultiLineString boundary,
    /// the value of the item is the index of the Semantic or Material object.
    pub(crate) linestrings: Vec<Option<RR>>,
    /// Each item corresponds to the surface with the same index, the value
    /// of the item is the index of the Semantic or Material object.
    pub(crate) surfaces: Vec<Option<RR>>,
    pub(crate) shells: Vec<VR>,
    pub(crate) solids: Vec<VR>,
}

impl<VR: VertexRef, RR: ResourceRef> SemanticOrMaterialMap<VR, RR> {
    /// Hint what [BoundaryType] does the SemanticOrMaterialMap belong to.
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
