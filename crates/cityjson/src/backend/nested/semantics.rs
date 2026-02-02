use crate::backend::nested::attributes::Attributes;
use crate::prelude::StringStorage;
use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq)]
pub struct Semantics<SS: StringStorage, RR> {
    pub surfaces: Vec<Semantic<SS, RR>>,
    pub values: SemanticValues,
    _marker: PhantomData<RR>,
}

impl<SS: StringStorage, RR> Semantics<SS, RR> {
    pub fn new(surfaces: Vec<Semantic<SS, RR>>, values: SemanticValues) -> Self {
        Self {
            surfaces,
            values,
            _marker: PhantomData,
        }
    }

    pub fn surfaces(&self) -> &Vec<Semantic<SS, RR>> {
        &self.surfaces
    }

    pub fn surfaces_mut(&mut self) -> &mut Vec<Semantic<SS, RR>> {
        &mut self.surfaces
    }

    pub fn values(&self) -> &SemanticValues {
        &self.values
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Semantic<SS: StringStorage, RR> {
    pub type_sem: SemanticType,
    pub children: Option<Vec<usize>>,
    pub parent: Option<usize>,
    pub attributes: Option<Attributes<SS, RR>>,
    _marker: PhantomData<RR>,
}

impl<SS: StringStorage, RR> Semantic<SS, RR> {
    pub fn new(type_semantic: SemanticType) -> Self {
        Self {
            type_sem: type_semantic,
            children: None,
            parent: None,
            attributes: None,
            _marker: PhantomData,
        }
    }

    pub fn type_semantic(&self) -> &SemanticType {
        &self.type_sem
    }

    pub fn has_children(&self) -> bool {
        self.children.as_ref().is_some_and(|c| !c.is_empty())
    }

    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }

    pub fn children(&self) -> Option<&Vec<usize>> {
        self.children.as_ref()
    }

    pub fn children_mut(&mut self) -> &mut Vec<usize> {
        if self.children.is_none() {
            self.children = Some(Vec::new());
        }
        self.children.as_mut().unwrap()
    }

    pub fn parent(&self) -> Option<usize> {
        self.parent
    }

    pub fn set_parent(&mut self, parent_idx: usize) {
        self.parent = Some(parent_idx);
    }

    pub fn attributes(&self) -> Option<&Attributes<SS, RR>> {
        self.attributes.as_ref()
    }

    pub fn attributes_mut(&mut self) -> &mut Attributes<SS, RR> {
        if self.attributes.is_none() {
            self.attributes = Some(Attributes::new());
        }
        self.attributes.as_mut().unwrap()
    }
}

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum SemanticType {
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
    Extension(String),
}

#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum SemanticValues {
    PointOrLineStringOrSurface(Vec<Option<usize>>),
    Solid(Vec<Vec<Option<usize>>>),
    MultiSolid(Vec<Vec<Vec<Option<usize>>>>),
}
