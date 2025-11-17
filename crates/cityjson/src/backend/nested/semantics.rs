use crate::backend::nested::attributes::Attributes;
use crate::prelude::StringStorage;

#[derive(Clone, Debug, PartialEq)]
pub struct Semantics<SS: StringStorage> {
    pub surfaces: Vec<Semantic<SS>>,
    pub values: SemanticValues,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Semantic<SS: StringStorage> {
    pub type_sem: SemanticType,
    pub children: Option<Vec<usize>>,
    pub parent: Option<usize>,
    pub attributes: Option<Attributes<SS>>,
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
