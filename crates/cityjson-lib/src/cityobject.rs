use crate::geometry::Geometry;
use crate::{errors, Attributes, BBox};
use std::collections::HashMap;
use std::fmt;

pub type CityObjects = HashMap<String, CityObject>;

pub type CityObjectId = String;
#[derive(Debug, Default, Clone)]
pub struct CityObject {
    type_object: CityObjectType,
    attributes: Option<Attributes>,
    geographical_extent: Option<BBox>,
    geometry: Option<Vec<Geometry>>,
    children: Option<Vec<CityObjectId>>,
    parents: Option<Vec<CityObjectId>>,
    extra: Option<Attributes>,
}

impl CityObject {
    pub fn new(type_co: CityObjectType) -> Self {
        Self {
            type_object: type_co,
            attributes: None,
            geographical_extent: None,
            geometry: None,
            children: None,
            parents: None,
            extra: None,
        }
    }

    pub fn type_object(&self) -> &CityObjectType {
        &self.type_object
    }

    pub fn attributes(&self) -> Option<&Attributes> {
        self.attributes.as_ref()
    }

    pub fn geographical_extent(&self) -> Option<&BBox> {
        self.geographical_extent.as_ref()
    }

    pub fn children(&self) -> Option<&Vec<String>> {
        self.children.as_ref()
    }

    pub fn parents(&self) -> Option<&Vec<String>> {
        self.parents.as_ref()
    }

    pub fn extra(&self) -> Option<&Attributes> {
        self.extra.as_ref()
    }

    pub fn attributes_mut(&mut self) -> &mut Option<Attributes> {
        &mut self.attributes
    }

    pub fn geographical_extent_mut(&mut self) -> &mut Option<BBox> {
        &mut self.geographical_extent
    }

    pub fn children_mut(&mut self) -> &mut Option<Vec<String>> {
        &mut self.children
    }

    pub fn parents_mut(&mut self) -> &mut Option<Vec<String>> {
        &mut self.parents
    }

    pub fn extra_mut(&mut self) -> &mut Option<Attributes> {
        &mut self.extra
    }

    // Setters
    pub fn set_attributes(&mut self, attributes: Attributes) {
        self.attributes = Some(attributes);
    }

    pub fn set_geographical_extent(&mut self, extent: BBox) {
        self.geographical_extent = Some(extent);
    }

    pub fn set_children(&mut self, children: Vec<String>) {
        self.children = Some(children);
    }

    pub fn set_parents(&mut self, parents: Vec<String>) {
        self.parents = Some(parents);
    }

    pub fn set_extra(&mut self, extra: Attributes) {
        self.extra = Some(extra);
    }
}

impl fmt::Display for CityObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CityObject {{")?;
        write!(f, "\n  type: {:?}", self.type_object)?;

        if let Some(attrs) = &self.attributes {
            write!(f, "\n  attributes: {}", attrs)?;
        }

        if let Some(extent) = &self.geographical_extent {
            write!(f, "\n  geographical_extent: {:?}", extent)?;
        }

        if let Some(children) = &self.children {
            write!(f, "\n  children: {:?}", children)?;
        }

        if let Some(parents) = &self.parents {
            write!(f, "\n  parents: {:?}", parents)?;
        }

        if let Some(extra) = &self.extra {
            write!(f, "\n  extra: {}", extra)?;
        }

        write!(f, "\n}}")
    }
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

impl From<serde_cityjson::v1_1::CityObjectType> for CityObjectType {
    fn from(cotype: serde_cityjson::v1_1::CityObjectType) -> Self {
        match cotype {
            serde_cityjson::v1_1::CityObjectType::Bridge => Self::Bridge,
            serde_cityjson::v1_1::CityObjectType::BridgePart => Self::BridgePart,
            serde_cityjson::v1_1::CityObjectType::BridgeInstallation => Self::BridgeInstallation,
            serde_cityjson::v1_1::CityObjectType::BridgeConstructiveElement => {
                Self::BridgeConstructiveElement
            }
            serde_cityjson::v1_1::CityObjectType::BridgeRoom => Self::BridgeRoom,
            serde_cityjson::v1_1::CityObjectType::BridgeFurniture => Self::BridgeFurniture,
            serde_cityjson::v1_1::CityObjectType::Building => Self::Building,
            serde_cityjson::v1_1::CityObjectType::BuildingPart => Self::BuildingPart,
            serde_cityjson::v1_1::CityObjectType::BuildingInstallation => {
                Self::BuildingInstallation
            }
            serde_cityjson::v1_1::CityObjectType::BuildingConstructiveElement => {
                Self::BuildingConstructiveElement
            }
            serde_cityjson::v1_1::CityObjectType::BuildingFurniture => Self::BuildingFurniture,
            serde_cityjson::v1_1::CityObjectType::BuildingStorey => Self::BuildingStorey,
            serde_cityjson::v1_1::CityObjectType::BuildingRoom => Self::BuildingRoom,
            serde_cityjson::v1_1::CityObjectType::BuildingUnit => Self::BuildingUnit,
            serde_cityjson::v1_1::CityObjectType::CityFurniture => Self::CityFurniture,
            serde_cityjson::v1_1::CityObjectType::CityObjectGroup => Self::CityObjectGroup,
            serde_cityjson::v1_1::CityObjectType::Default => Self::Default,
            serde_cityjson::v1_1::CityObjectType::LandUse => Self::LandUse,
            serde_cityjson::v1_1::CityObjectType::OtherConstruction => Self::OtherConstruction,
            serde_cityjson::v1_1::CityObjectType::PlantCover => Self::PlantCover,
            serde_cityjson::v1_1::CityObjectType::SolitaryVegetationObject => {
                Self::SolitaryVegetationObject
            }
            serde_cityjson::v1_1::CityObjectType::TINRelief => Self::TINRelief,
            serde_cityjson::v1_1::CityObjectType::WaterBody => Self::WaterBody,
            serde_cityjson::v1_1::CityObjectType::Road => Self::Road,
            serde_cityjson::v1_1::CityObjectType::Railway => Self::Railway,
            serde_cityjson::v1_1::CityObjectType::Waterway => Self::Waterway,
            serde_cityjson::v1_1::CityObjectType::TransportSquare => Self::TransportSquare,
            serde_cityjson::v1_1::CityObjectType::Tunnel => Self::Tunnel,
            serde_cityjson::v1_1::CityObjectType::TunnelPart => Self::TunnelPart,
            serde_cityjson::v1_1::CityObjectType::TunnelInstallation => Self::TunnelInstallation,
            serde_cityjson::v1_1::CityObjectType::TunnelConstructiveElement => {
                Self::TunnelConstructiveElement
            }
            serde_cityjson::v1_1::CityObjectType::TunnelHollowSpace => Self::TunnelHollowSpace,
            serde_cityjson::v1_1::CityObjectType::TunnelFurniture => Self::TunnelFurniture,
            serde_cityjson::v1_1::CityObjectType::Extension(s) => Self::Extension(s),
        }
    }
}

impl<'a> TryFrom<serde_cityjson::v1_1::CityObject<'a>> for CityObject {
    type Error = errors::Error;

    fn try_from(co: serde_cityjson::v1_1::CityObject<'a>) -> errors::Result<Self> {
        let mut city_object = CityObject::new(CityObjectType::from(co.type_co));

        if let Some(attrs) = co.attributes {
            city_object.set_attributes(Attributes::try_from(attrs)?);
        }

        if let Some(extent) = co.geographical_extent {
            city_object.set_geographical_extent(extent);
        }

        if let Some(children) = co.children {
            city_object.set_children(children.into_iter().map(|s| s.into_owned()).collect());
        }

        if let Some(parents) = co.parents {
            city_object.set_parents(parents.into_iter().map(|s| s.into_owned()).collect());
        }

        if let Some(extra) = co.extra {
            city_object.set_extra(Attributes::try_from(extra)?);
        }

        Ok(city_object)
    }
}
