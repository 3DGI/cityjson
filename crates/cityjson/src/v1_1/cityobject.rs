//! # CityObject
//!
//! Represents a [CityObject object](https://www.cityjson.org/specs/2.0.1/#the-different-city-objects).

use std::fmt::{Display, Formatter};
use crate::cityjson;
use crate::cityjson::attributes::Attributes;
use crate::resources::pool::ResourceRef;
use crate::resources::storage::StringStorage;
use crate::v1_1::metadata::BBox;

#[derive(Debug, Default, Clone)]
pub struct CityObject<SS: StringStorage, RR: ResourceRef> {
    type_cityobject: CityObjectType,
    geometry: Option<Vec<RR>>,
    attributes: Option<Attributes<SS>>,
    geographical_extent: Option<BBox>,
    children: Option<Vec<SS>>,
    parents: Option<Vec<SS>>,
    extra: Option<Attributes<SS>>,
}

#[derive(Debug, Default, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum CityObjectType {
    Bridge,
    BridgePart,
    BridgeInstallation,
    BridgeConstructiveElement,
    BridgeRoom,
    BridgeFurniture,
    Building,
    BuildingPart,
    BuildingInstallation,
    BuildingConstructiveElement,
    BuildingFurniture,
    BuildingStorey,
    BuildingRoom,
    BuildingUnit,
    CityFurniture,
    CityObjectGroup,
    #[default]
    Default,
    LandUse,
    OtherConstruction,
    PlantCover,
    SolitaryVegetationObject,
    TINRelief,
    WaterBody,
    Road,
    Railway,
    Waterway,
    TransportSquare,
    Tunnel,
    TunnelPart,
    TunnelInstallation,
    TunnelConstructiveElement,
    TunnelHollowSpace,
    TunnelFurniture,
    Extension(String),
}

impl<SS: StringStorage, RR: ResourceRef> CityObject<SS, RR> {
    pub fn new(type_cityobject: CityObjectType) -> Self {
        Self {
            type_cityobject,
            geometry: None,
            attributes: None,
            geographical_extent: None,
            children: None,
            parents: None,
            extra: None,
        }
    }

    pub fn get_type(&self) -> &CityObjectType {
        &self.type_cityobject
    }


    pub fn get_geometry(&self) -> Option<&Vec<RR>> {
        self.geometry.as_ref()
    }

    pub fn get_geometry_mut(&mut self) -> &mut Vec<RR> {
        self.geometry.get_or_insert_with(Vec::new)
    }

    pub fn get_attributes(&self) -> Option<&Attributes<SS>> {
        self.attributes.as_ref()
    }

    pub fn get_attributes_mut(&mut self) -> &mut Attributes<SS> {
        self.attributes.get_or_insert_with(Attributes::new)
    }

    pub fn get_geographical_extent(&self) -> Option<&BBox> {
        self.geographical_extent.as_ref()
    }

    pub fn set_geographical_extent(&mut self, bbox: Option<BBox>) {
        self.geographical_extent = bbox;
    }

    pub fn get_children(&self) -> Option<&Vec<SS>> {
        self.children.as_ref()
    }

    pub fn get_children_mut(&mut self) -> &mut Vec<SS> {
        self.children.get_or_insert_with(Vec::new)
    }

    pub fn get_parents(&self) -> Option<&Vec<SS>> {
        self.parents.as_ref()
    }

    pub fn get_parents_mut(&mut self) -> &mut Vec<SS> {
        self.parents.get_or_insert_with(Vec::new)
    }

    pub fn get_extra(&self) -> Option<&Attributes<SS>> {
        self.extra.as_ref()
    }

    pub fn get_extra_mut(&mut self) -> &mut Attributes<SS> {
        self.extra.get_or_insert_with(Attributes::new)
    }
}

impl<SS: StringStorage, RR: ResourceRef> Display for CityObject<SS, RR> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Display for CityObjectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

#[test]
fn t() {
    println!("{}", CityObjectType::CityObjectGroup);
}

impl<SS: StringStorage, RR: ResourceRef> cityjson::cityobject::CityObject for CityObject<SS, RR> {}
