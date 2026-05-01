//! Surface semantic types for `CityJSON` v2.0 geometries.
//!
//! Semantics describe what a geometric primitive represents — a wall, a roof surface, a door,
//! etc. Each [`Semantic`] object has a [`SemanticType`] and optional parent/child references
//! to model openings (e.g. a `Window` or `Door` is a child of a `WallSurface`).
//!
//! Semantics are stored once in the model's semantic pool and referenced from geometry maps
//! by [`SemanticHandle`]. Use [`CityModel::add_semantic`] or
//! [`CityModel::get_or_insert_semantic`] to register them.
//!
//! [`SemanticHandle`]: crate::resources::handles::SemanticHandle
//! [`CityModel::add_semantic`]: super::super::citymodel::CityModel::add_semantic
//! [`CityModel::get_or_insert_semantic`]: super::super::citymodel::CityModel::get_or_insert_semantic
//!
//! ```rust
//! use cityjson_types::v2_0::{OwnedSemantic, SemanticType};
//!
//! let mut wall = OwnedSemantic::new(SemanticType::WallSurface);
//!
//! let mut window = OwnedSemantic::new(SemanticType::Window);
//! // parent/child links are set after inserting both into the model pool
//! assert!(!window.has_parent());
//! assert!(!wall.has_children());
//! ```
//!
//! ```
//! use cityjson_types::v2_0::{OwnedAttributeValue, OwnedSemantic, SemanticType};
//!
//! let mut semantic = OwnedSemantic::new(SemanticType::RoofSurface);
//! semantic.attributes_mut().insert(
//!     "material".to_string(),
//!     OwnedAttributeValue::String("tiles".to_string()),
//! );
//!
//! assert_eq!(semantic.type_semantic(), &SemanticType::RoofSurface);
//! assert_eq!(
//!     semantic.attributes().and_then(|attributes| attributes.get("material")),
//!     Some(&OwnedAttributeValue::String("tiles".to_string()))
//! );
//! assert!(!semantic.has_children());
//! assert!(!semantic.has_parent());
//! ```

use crate::cityjson::core::semantic::SemanticTypeTrait;
use crate::format_option;
use crate::resources::handles::SemanticHandle;
use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};
use crate::v2_0::attributes::Attributes;
use std::fmt::{Display, Formatter};

pub type OwnedSemantic = Semantic<OwnedStringStorage>;
pub type BorrowedSemantic<'a> = Semantic<BorrowedStringStorage<'a>>;

/// A semantic object describing what a geometric surface represents.
///
/// Spec: [Semantic Object](https://www.cityjson.org/specs/2.0.1/#semantics-of-geometric-primitives).
#[derive(Debug, Clone, PartialEq)]
pub struct Semantic<SS: StringStorage> {
    kind: SemanticType<SS>,
    children: Option<Vec<SemanticHandle>>,
    parent: Option<SemanticHandle>,
    attributes: Option<Attributes<SS>>,
}

impl<SS: StringStorage> Semantic<SS> {
    pub fn new(semantic_type: SemanticType<SS>) -> Self {
        Self {
            kind: semantic_type,
            children: None,
            parent: None,
            attributes: None,
        }
    }

    pub fn type_semantic(&self) -> &SemanticType<SS> {
        &self.kind
    }

    pub fn has_children(&self) -> bool {
        self.children.as_ref().is_some_and(|c| !c.is_empty())
    }

    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }

    pub fn children(&self) -> Option<&[SemanticHandle]> {
        self.children.as_deref()
    }

    pub fn children_mut(&mut self) -> &mut Vec<SemanticHandle> {
        self.children.get_or_insert_with(Vec::new)
    }

    pub fn parent(&self) -> Option<SemanticHandle> {
        self.parent
    }

    pub fn set_parent(&mut self, parent_ref: SemanticHandle) {
        self.parent = Some(parent_ref);
    }

    pub fn attributes(&self) -> Option<&Attributes<SS>> {
        self.attributes.as_ref()
    }

    pub fn attributes_mut(&mut self) -> &mut Attributes<SS> {
        self.attributes.get_or_insert_with(Attributes::new)
    }
}

impl<SS: StringStorage> Display for Semantic<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "type: {}, children: {:?}, parent: {:?}, attributes: {}",
            self.kind,
            self.children,
            self.parent,
            format_option(self.attributes.as_ref())
        )
    }
}

/// The type of a semantic surface.
///
/// Standard types cover buildings, water bodies, and transportation surfaces.
/// Extension types start with `"+"` and are represented by `Extension(name)`.
///
/// Allowed types per city object:
/// - Buildings: `RoofSurface`, `GroundSurface`, `WallSurface`, `ClosureSurface`,
///   `OuterCeilingSurface`, `OuterFloorSurface`, `Window`, `Door`,
///   `InteriorWallSurface`, `CeilingSurface`, `FloorSurface`
/// - Water bodies: `WaterSurface`, `WaterGroundSurface`, `WaterClosureSurface`
/// - Transportation: `TrafficArea`, `AuxiliaryTrafficArea`,
///   `TransportationMarking`, `TransportationHole`
#[derive(Debug, Default, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[non_exhaustive]
pub enum SemanticType<SS: StringStorage> {
    #[default]
    Default,
    RoofSurface,
    GroundSurface,
    WallSurface,
    ClosureSurface,
    OuterCeilingSurface,
    OuterFloorSurface,
    Window,
    Door,
    InteriorWallSurface,
    CeilingSurface,
    FloorSurface,
    WaterSurface,
    WaterGroundSurface,
    WaterClosureSurface,
    TrafficArea,
    AuxiliaryTrafficArea,
    TransportationMarking,
    TransportationHole,
    Extension(SS::String),
}

impl<SS: StringStorage> Display for SemanticType<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<SS: StringStorage> SemanticTypeTrait for SemanticType<SS> {}
