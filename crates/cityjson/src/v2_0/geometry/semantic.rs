use crate::cityjson::core::attributes::Attributes;
use crate::cityjson::traits::semantic::SemanticTypeTrait;
use crate::format_option;
use crate::resources::handles::SemanticRef;
use crate::resources::storage::StringStorage;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub struct Semantic<SS: StringStorage> {
    kind: SemanticType<SS>,
    children: Option<Vec<SemanticRef>>,
    parent: Option<SemanticRef>,
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

    pub fn children(&self) -> Option<&[SemanticRef]> {
        self.children.as_deref()
    }

    pub fn children_mut(&mut self) -> &mut Vec<SemanticRef> {
        self.children.get_or_insert_with(Vec::new)
    }

    pub fn parent(&self) -> Option<SemanticRef> {
        self.parent
    }

    pub fn set_parent(&mut self, parent_ref: SemanticRef) {
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

#[derive(Debug, Default, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
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
