use crate::cityjson::core::cityobject::{CityObjectCore, CityObjectsCore};
use crate::error::Error;
use crate::prelude::*;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct CityObjects<SS: StringStorage, RR: ResourceRef> {
    inner: CityObjectsCore<SS, RR, CityObject<SS, RR>>,
}

crate::macros::impl_cityobjects_methods!();

/// A CityObjects container using owned strings.
pub type OwnedCityObjects<RR> = CityObjects<OwnedStringStorage, RR>;

/// A CityObjects container using borrowed strings.
pub type BorrowedCityObjects<'a, RR> = CityObjects<BorrowedStringStorage<'a>, RR>;

#[derive(Debug, Default, Clone)]
pub struct CityObject<SS: StringStorage, RR: ResourceRef> {
    inner: CityObjectCore<SS, RR, CityObjectType<SS>>,
}

crate::macros::impl_cityobject_methods!(CityObjectType<SS>);

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
    GenericCityObject,
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

impl FromStr for CityObjectType<OwnedStringStorage> {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Bridge" => Ok(CityObjectType::Bridge),
            "BridgePart" => Ok(CityObjectType::BridgePart),
            "BridgeInstallation" => Ok(CityObjectType::BridgeInstallation),
            "BridgeConstructiveElement" => Ok(CityObjectType::BridgeConstructiveElement),
            "BridgeRoom" => Ok(CityObjectType::BridgeRoom),
            "BridgeFurniture" => Ok(CityObjectType::BridgeFurniture),
            "Building" => Ok(CityObjectType::Building),
            "BuildingPart" => Ok(CityObjectType::BuildingPart),
            "BuildingInstallation" => Ok(CityObjectType::BuildingInstallation),
            "BuildingConstructiveElement" => Ok(CityObjectType::BuildingConstructiveElement),
            "BuildingFurniture" => Ok(CityObjectType::BuildingFurniture),
            "BuildingStorey" => Ok(CityObjectType::BuildingStorey),
            "BuildingRoom" => Ok(CityObjectType::BuildingRoom),
            "BuildingUnit" => Ok(CityObjectType::BuildingUnit),
            "CityFurniture" => Ok(CityObjectType::CityFurniture),
            "CityObjectGroup" => Ok(CityObjectType::CityObjectGroup),
            "Default" => Ok(CityObjectType::Default),
            "GenericCityObject" => Ok(CityObjectType::GenericCityObject),
            "LandUse" => Ok(CityObjectType::LandUse),
            "OtherConstruction" => Ok(CityObjectType::OtherConstruction),
            "PlantCover" => Ok(CityObjectType::PlantCover),
            "SolitaryVegetationObject" => Ok(CityObjectType::SolitaryVegetationObject),
            "TINRelief" => Ok(CityObjectType::TINRelief),
            "WaterBody" => Ok(CityObjectType::WaterBody),
            "Road" => Ok(CityObjectType::Road),
            "Railway" => Ok(CityObjectType::Railway),
            "Waterway" => Ok(CityObjectType::Waterway),
            "TransportSquare" => Ok(CityObjectType::TransportSquare),
            "Tunnel" => Ok(CityObjectType::Tunnel),
            "TunnelPart" => Ok(CityObjectType::TunnelPart),
            "TunnelInstallation" => Ok(CityObjectType::TunnelInstallation),
            "TunnelConstructiveElement" => Ok(CityObjectType::TunnelConstructiveElement),
            "TunnelHollowSpace" => Ok(CityObjectType::TunnelHollowSpace),
            "TunnelFurniture" => Ok(CityObjectType::TunnelFurniture),
            _ => {
                if s.chars().nth(0).is_some_and(|first_char| first_char == '+') {
                    Ok(CityObjectType::Extension(s.to_string()))
                } else {
                    Err(Error::InvalidCityObjectType(s.to_string()))
                }
            }
        }
    }
}

// Note: Tests for CityObjects container (test_basic_operations, test_filtering,
// test_bulk_operations, and test_attribute_filtering) are now generated by the
// impl_cityobjects_methods! macro and are available in the cityobjects_macro_tests module
