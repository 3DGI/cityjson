//! # CityObject
//!
//! Represents a [CityObject object](https://www.cityjson.org/specs/1.1.3/#the-different-city-objects).

use crate::prelude::{
    Attributes, CityObjectTrait, CityObjectTypeTrait, ResourceRef,
    StringStorage,
};
use crate::v1_1::BBox;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone)]
pub struct CityObject<SS: StringStorage, RR: ResourceRef> {
    type_cityobject: CityObjectType<SS>,
    geometry: Option<Vec<RR>>,
    attributes: Option<Attributes<SS>>,
    geographical_extent: Option<BBox>,
    children: Option<Vec<SS>>,
    parents: Option<Vec<SS>>,
    extra: Option<Attributes<SS>>,
}

impl<SS: StringStorage, RR: ResourceRef> CityObjectTrait<SS, RR, CityObjectType<SS>, BBox>
    for CityObject<SS, RR>
{
    fn new(type_cityobject: CityObjectType<SS>) -> Self {
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
    fn get_type(&self) -> &CityObjectType<SS> {
        &self.type_cityobject
    }
    fn get_geometry(&self) -> Option<&Vec<RR>> {
        self.geometry.as_ref()
    }
    fn get_geometry_mut(&mut self) -> &mut Vec<RR> {
        self.geometry.get_or_insert_with(Vec::new)
    }
    fn get_attributes(&self) -> Option<&Attributes<SS>> {
        self.attributes.as_ref()
    }
    fn get_attributes_mut(&mut self) -> &mut Attributes<SS> {
        self.attributes.get_or_insert_with(Attributes::new)
    }
    fn get_geographical_extent(&self) -> Option<&BBox> {
        self.geographical_extent.as_ref()
    }
    fn set_geographical_extent(&mut self, bbox: Option<BBox>) {
        self.geographical_extent = bbox;
    }
    fn get_children(&self) -> Option<&Vec<SS>> {
        self.children.as_ref()
    }
    fn get_children_mut(&mut self) -> &mut Vec<SS> {
        self.children.get_or_insert_with(Vec::new)
    }
    fn get_parents(&self) -> Option<&Vec<SS>> {
        self.parents.as_ref()
    }
    fn get_parents_mut(&mut self) -> &mut Vec<SS> {
        self.parents.get_or_insert_with(Vec::new)
    }
    fn get_extra(&self) -> Option<&Attributes<SS>> {
        self.extra.as_ref()
    }
    fn get_extra_mut(&mut self) -> &mut Attributes<SS> {
        self.extra.get_or_insert_with(Attributes::new)
    }
}

impl<SS: StringStorage, RR: ResourceRef> Display for CityObject<SS, RR> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

#[derive(Debug, Default, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum CityObjectType<SS: StringStorage> {
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
    Extension(SS::String),
}

impl<SS: StringStorage> Display for CityObjectType<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let CityObjectType::Extension(ext) = self {
            write!(f, "{}", ext)
        } else {
            write!(f, "{:#?}", self)
        }
    }
}

impl<SS: StringStorage> CityObjectTypeTrait for CityObjectType<SS> {}

#[test]
fn t() {
    println!("{}", CityObjectType::<OwnedStringStorage>::CityObjectGroup);
    println!(
        "{}",
        CityObjectType::<OwnedStringStorage>::Extension("+NoiseBuilding".to_string())
    );
}
