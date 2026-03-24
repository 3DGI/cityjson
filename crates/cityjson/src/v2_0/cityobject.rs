//! City objects — the features in a `CityJSON` dataset.
//!
//! Each entry in the `CityObjects` map is a [`CityObject`] with a string ID, a type, optional
//! geometry references, optional attributes, and optional parent/child relationships.
//!
//! **1st-level types** can exist without a parent:
//!
//! `Bridge`, `Building`, `CityFurniture`, `CityObjectGroup`, `GenericCityObject`, `LandUse`,
//! `OtherConstruction`, `PlantCover`, `SolitaryVegetationObject`, `TINRelief`, `Road`,
//! `Railway`, `TransportSquare`, `Waterway`, `WaterBody`, `Tunnel`
//!
//! **2nd-level types** must reference at least one parent:
//!
//! `BridgeConstructiveElement`, `BridgeFurniture`, `BridgeInstallation`, `BridgePart`,
//! `BridgeRoom`, `BuildingConstructiveElement`, `BuildingFurniture`, `BuildingInstallation`,
//! `BuildingPart`, `BuildingRoom`, `BuildingStorey`, `BuildingUnit`,
//! `TunnelConstructiveElement`, `TunnelFurniture`, `TunnelHollowSpace`, `TunnelInstallation`,
//! `TunnelPart`
//!
//! ```rust
//! use cityjson::CityModelType;
//! use cityjson::v2_0::{
//!     CityObject, CityObjectIdentifier, CityObjectType, OwnedAttributeValue, OwnedCityModel,
//! };
//!
//! let mut model = OwnedCityModel::new(CityModelType::CityJSON);
//!
//! let mut building = CityObject::new(
//!     CityObjectIdentifier::new("building-001".to_string()),
//!     CityObjectType::Building,
//! );
//! building
//!     .attributes_mut()
//!     .insert("measuredHeight".to_string(), OwnedAttributeValue::Float(15.3));
//!
//! let handle = model.cityobjects_mut().add(building).unwrap();
//! assert!(model.cityobjects().get(handle).is_some());
//! ```

use crate::backend::default::cityobject::{CityObjectCore, CityObjectsCore};
use crate::error::{Error, Result};
use crate::resources::handles::{CityObjectHandle, GeometryHandle, cast_handle_slice};
use crate::resources::id::ResourceId32;
use crate::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub use crate::cityjson::core::cityobject::CityObjectIdentifier;

/// The ordered collection of city objects in a [`CityModel`](super::citymodel::CityModel).
///
/// Wraps a resource pool keyed by [`CityObjectHandle`].
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
    pub fn add(&mut self, city_object: CityObject<SS>) -> Result<CityObjectHandle> {
        self.inner.add(city_object).map(CityObjectHandle::from_raw)
    }

    #[must_use]
    pub fn get(&self, id: CityObjectHandle) -> Option<&CityObject<SS>> {
        self.inner.get(id.to_raw())
    }

    pub fn get_mut(&mut self, id: CityObjectHandle) -> Option<&mut CityObject<SS>> {
        self.inner.get_mut(id.to_raw())
    }

    pub fn remove(&mut self, id: CityObjectHandle) -> Option<CityObject<SS>> {
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

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (CityObjectHandle, &'a CityObject<SS>)>
    where
        CityObject<SS>: 'a,
    {
        self.inner
            .iter()
            .map(|(id, value)| (CityObjectHandle::from_raw(id), value))
    }

    pub fn iter_mut<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = (CityObjectHandle, &'a mut CityObject<SS>)>
    where
        CityObject<SS>: 'a,
    {
        self.inner
            .iter_mut()
            .map(|(id, value)| (CityObjectHandle::from_raw(id), value))
    }

    #[must_use]
    pub fn first(&self) -> Option<(CityObjectHandle, &CityObject<SS>)> {
        self.inner
            .first()
            .map(|(id, value)| (CityObjectHandle::from_raw(id), value))
    }

    #[must_use]
    pub fn last(&self) -> Option<(CityObjectHandle, &CityObject<SS>)> {
        self.inner
            .last()
            .map(|(id, value)| (CityObjectHandle::from_raw(id), value))
    }

    pub fn ids(&self) -> impl Iterator<Item = CityObjectHandle> + '_ {
        self.inner.ids().map(CityObjectHandle::from_raw)
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
    ) -> Result<Vec<CityObjectHandle>> {
        self.inner
            .add_many(objects)
            .map(|ids| ids.into_iter().map(CityObjectHandle::from_raw).collect())
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn filter<F>(
        &self,
        predicate: F,
    ) -> impl Iterator<Item = (CityObjectHandle, &CityObject<SS>)>
    where
        F: Fn(&CityObject<SS>) -> bool,
    {
        self.inner
            .filter(predicate)
            .map(|(id, value)| (CityObjectHandle::from_raw(id), value))
    }
}

impl<SS: StringStorage> Default for CityObjects<SS> {
    fn default() -> Self {
        Self::new()
    }
}

pub type OwnedCityObjects = CityObjects<OwnedStringStorage>;
pub type BorrowedCityObjects<'a> = CityObjects<BorrowedStringStorage<'a>>;

/// A single city object.
///
/// Corresponds to one entry in the `CityObjects` JSON map. Holds the object type, geometry
/// handle references, attributes, optional bounding box, and parent/child handle lists.
///
/// Geometry handles must reference geometries already stored in the [`CityModel`](super::citymodel::CityModel);
/// use [`CityObject::add_geometry`] to attach them after insertion.
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

    pub fn id(&self) -> &str
    where
        SS::String: AsRef<str>,
    {
        self.inner.id().as_ref()
    }

    pub fn type_cityobject(&self) -> &CityObjectType<SS> {
        self.inner.type_cityobject()
    }

    pub fn geometry(&self) -> Option<&[GeometryHandle]> {
        self.inner
            .geometry()
            .map(|items| cast_handle_slice::<GeometryHandle>(items.as_slice()))
    }

    pub fn add_geometry(&mut self, geometry_ref: GeometryHandle) {
        self.inner.geometry_mut().push(geometry_ref.to_raw());
    }

    pub fn clear_geometry(&mut self) {
        self.inner.geometry_mut().clear();
    }

    pub fn attributes(&self) -> Option<&crate::v2_0::attributes::Attributes<SS>> {
        self.inner.attributes()
    }

    pub fn attributes_mut(&mut self) -> &mut crate::v2_0::attributes::Attributes<SS> {
        self.inner.attributes_mut()
    }

    pub fn geographical_extent(&self) -> Option<&crate::v2_0::metadata::BBox> {
        self.inner.geographical_extent()
    }

    pub fn set_geographical_extent(&mut self, bbox: Option<crate::v2_0::metadata::BBox>) {
        self.inner.set_geographical_extent(bbox);
    }

    pub fn children(&self) -> Option<&[CityObjectHandle]> {
        self.inner
            .children()
            .map(|items| cast_handle_slice::<CityObjectHandle>(items.as_slice()))
    }

    pub fn add_child(&mut self, child: CityObjectHandle) {
        self.inner.children_mut().push(child.to_raw());
    }

    pub fn clear_children(&mut self) {
        self.inner.children_mut().clear();
    }

    pub fn parents(&self) -> Option<&[CityObjectHandle]> {
        self.inner
            .parents()
            .map(|items| cast_handle_slice::<CityObjectHandle>(items.as_slice()))
    }

    pub fn add_parent(&mut self, parent: CityObjectHandle) {
        self.inner.parents_mut().push(parent.to_raw());
    }

    pub fn clear_parents(&mut self) {
        self.inner.parents_mut().clear();
    }

    pub fn extra(&self) -> Option<&crate::v2_0::attributes::Attributes<SS>> {
        self.inner.extra()
    }

    pub fn extra_mut(&mut self) -> &mut crate::v2_0::attributes::Attributes<SS> {
        self.inner.extra_mut()
    }
}

impl<SS: StringStorage> Display for CityObject<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:#?}")
    }
}

/// The type of a city object, as defined in the `CityJSON` specification.
///
/// 1st-level types (e.g. `Building`, `Road`) can exist independently.
/// 2nd-level types (e.g. `BuildingPart`, `BuildingRoom`) require a `parents` reference.
///
/// Extension types are represented by `Extension(name)` where `name` starts with `"+"`.
/// `FromStr` accepts all standard names plus `"+"` prefixed extension names.
#[derive(Debug, Default, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[non_exhaustive]
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
