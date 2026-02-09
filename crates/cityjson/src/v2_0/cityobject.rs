use crate::cityjson::core::cityobject::{CityObjectCore, CityObjectsCore};
use crate::error::{Error, Result};
use crate::resources::handles::{CityObjectRef, GeometryRef};
use crate::resources::pool::ResourceId32;
use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};
use crate::v2_0::types::CityObjectIdentifier;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct CityObjects<SS: StringStorage> {
    inner: CityObjectsCore<SS, ResourceId32, CityObject<SS>>,
}

impl<SS: StringStorage> CityObjects<SS> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: CityObjectsCore::new(),
        }
    }

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: CityObjectsCore::with_capacity(capacity),
        }
    }

    /// Add a city object and return its handle.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when the city-object pool cannot store
    /// additional entries for `ResourceId32`.
    pub fn add(&mut self, city_object: CityObject<SS>) -> Result<CityObjectRef> {
        self.inner.add(city_object).map(CityObjectRef::from_raw)
    }

    #[must_use]
    pub fn get(&self, id: CityObjectRef) -> Option<&CityObject<SS>> {
        self.inner.get(id.to_raw())
    }

    pub fn get_mut(&mut self, id: CityObjectRef) -> Option<&mut CityObject<SS>> {
        self.inner.get_mut(id.to_raw())
    }

    pub fn remove(&mut self, id: CityObjectRef) -> Option<CityObject<SS>> {
        self.inner.remove(id.to_raw())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (CityObjectRef, &'a CityObject<SS>)>
    where
        CityObject<SS>: 'a,
    {
        self.inner
            .iter()
            .map(|(id, value)| (CityObjectRef::from_raw(id), value))
    }

    pub fn iter_mut<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = (CityObjectRef, &'a mut CityObject<SS>)>
    where
        CityObject<SS>: 'a,
    {
        self.inner
            .iter_mut()
            .map(|(id, value)| (CityObjectRef::from_raw(id), value))
    }

    #[must_use]
    pub fn first(&self) -> Option<(CityObjectRef, &CityObject<SS>)> {
        self.inner
            .first()
            .map(|(id, value)| (CityObjectRef::from_raw(id), value))
    }

    #[must_use]
    pub fn last(&self) -> Option<(CityObjectRef, &CityObject<SS>)> {
        self.inner
            .last()
            .map(|(id, value)| (CityObjectRef::from_raw(id), value))
    }

    pub fn ids(&self) -> Vec<CityObjectRef> {
        self.inner
            .ids()
            .into_iter()
            .map(CityObjectRef::from_raw)
            .collect()
    }

    /// Add many city objects and return their handles.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when inserting one of the objects
    /// exceeds city-object pool capacity.
    pub fn add_many<I: IntoIterator<Item = CityObject<SS>>>(
        &mut self,
        objects: I,
    ) -> Result<Vec<CityObjectRef>> {
        self.inner
            .add_many(objects)
            .map(|ids| ids.into_iter().map(CityObjectRef::from_raw).collect())
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn filter<F>(&self, predicate: F) -> Vec<(CityObjectRef, &CityObject<SS>)>
    where
        F: Fn(&CityObject<SS>) -> bool,
    {
        self.inner
            .filter(predicate)
            .into_iter()
            .map(|(id, value)| (CityObjectRef::from_raw(id), value))
            .collect()
    }
}

impl<SS: StringStorage> Default for CityObjects<SS> {
    fn default() -> Self {
        Self::new()
    }
}

pub type OwnedCityObjects = CityObjects<OwnedStringStorage>;
pub type BorrowedCityObjects<'a> = CityObjects<BorrowedStringStorage<'a>>;

#[derive(Debug, Default, Clone)]
pub struct CityObject<SS: StringStorage> {
    inner: CityObjectCore<SS, ResourceId32, CityObjectType<SS>>,
}

impl<SS: StringStorage> CityObject<SS> {
    pub fn new(id: CityObjectIdentifier<SS>, type_cityobject: CityObjectType<SS>) -> Self {
        Self {
            inner: CityObjectCore::new(id.into_inner(), type_cityobject),
        }
    }

    pub fn id(&self) -> CityObjectIdentifier<SS>
    where
        SS::String: Clone,
    {
        CityObjectIdentifier::new(self.inner.id().clone())
    }

    pub fn type_cityobject(&self) -> &CityObjectType<SS> {
        self.inner.type_cityobject()
    }

    pub fn geometry(&self) -> Option<Vec<GeometryRef>> {
        self.inner.geometry().map(|items| {
            items
                .iter()
                .copied()
                .map(GeometryRef::from_raw)
                .collect::<Vec<_>>()
        })
    }

    pub fn add_geometry(&mut self, geometry_ref: GeometryRef) {
        self.inner.geometry_mut().push(geometry_ref.to_raw());
    }

    pub fn clear_geometry(&mut self) {
        self.inner.geometry_mut().clear();
    }

    pub fn attributes(&self) -> Option<&crate::cityjson::core::attributes::Attributes<SS>> {
        self.inner.attributes()
    }

    pub fn attributes_mut(&mut self) -> &mut crate::cityjson::core::attributes::Attributes<SS> {
        self.inner.attributes_mut()
    }

    pub fn geographical_extent(&self) -> Option<&crate::cityjson::core::metadata::BBox> {
        self.inner.geographical_extent()
    }

    pub fn set_geographical_extent(&mut self, bbox: Option<crate::cityjson::core::metadata::BBox>) {
        self.inner.set_geographical_extent(bbox);
    }

    pub fn children(&self) -> Option<Vec<CityObjectRef>> {
        self.inner.children().map(|items| {
            items
                .iter()
                .copied()
                .map(CityObjectRef::from_raw)
                .collect::<Vec<_>>()
        })
    }

    pub fn add_child(&mut self, child: CityObjectRef) {
        self.inner.children_mut().push(child.to_raw());
    }

    pub fn clear_children(&mut self) {
        self.inner.children_mut().clear();
    }

    pub fn parents(&self) -> Option<Vec<CityObjectRef>> {
        self.inner.parents().map(|items| {
            items
                .iter()
                .copied()
                .map(CityObjectRef::from_raw)
                .collect::<Vec<_>>()
        })
    }

    pub fn add_parent(&mut self, parent: CityObjectRef) {
        self.inner.parents_mut().push(parent.to_raw());
    }

    pub fn clear_parents(&mut self) {
        self.inner.parents_mut().clear();
    }

    pub fn extra(&self) -> Option<&crate::cityjson::core::attributes::Attributes<SS>> {
        self.inner.extra()
    }

    pub fn extra_mut(&mut self) -> &mut crate::cityjson::core::attributes::Attributes<SS> {
        self.inner.extra_mut()
    }
}

impl<SS: StringStorage> Display for CityObject<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:#?}")
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
            write!(f, "{ext}")
        } else {
            write!(f, "{self:#?}")
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
                if s.chars().next().is_some_and(|first_char| first_char == '+') {
                    Ok(CityObjectType::Extension(s.to_string()))
                } else {
                    Err(Error::InvalidCityObjectType(s.to_string()))
                }
            }
        }
    }
}
