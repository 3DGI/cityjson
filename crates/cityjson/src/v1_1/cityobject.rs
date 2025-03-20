//! # CityObject
//!
//! Represents a [CityObject object](https://www.cityjson.org/specs/1.1.3/#the-different-city-objects).
//!
//! Properties that are specific to certain type of CityObjects need to be set as `extra`
//! attributes. For example the `address` member of the `Bridge` and `Building` types,
//! or the `children_roles` member of the `CityObjectGroup`.

use crate::cityjson::traits::cityobject::CityObjectsTrait;
use crate::prelude::{
    Attributes, BorrowedStringStorage, CityObjectTrait, CityObjectTypeTrait, DefaultResourcePool,
    OwnedStringStorage, ResourcePool, ResourceRef, StringStorage,
};
use crate::v1_1::BBox;
use std::fmt::{Display, Formatter};

/// A container for efficiently storing and accessing CityObject instances.
///
/// This container provides fast random access, iteration, and mutation capabilities,
/// suitable for managing thousands of CityObjects in a CityJSON model.
///
/// # Type Parameters
///
/// * `SS` - The string storage strategy to use (e.g., `OwnedStringStorage` or `BorrowedStringStorage`)
/// * `RR` - The resource reference type used for identifying objects (e.g., `ResourceId32`)
///
/// # Examples
///
/// ```
/// use cityjson::prelude::*;
/// use cityjson::v1_1::{CityObjects, CityObject, CityObjectType};
///
/// // Create a container for CityObjects
/// let mut objects = CityObjects::<OwnedStringStorage, ResourceId32>::new();
///
/// // Add a building
/// let building = CityObject::new("id-1".to_string(), CityObjectType::Building);
/// let building_id = objects.add(building);
///
/// // Retrieve a CityObject by its ID
/// let retrieved_building = objects.get(building_id).unwrap();
/// assert_eq!(retrieved_building.type_cityobject(), &CityObjectType::Building);
/// ```
#[derive(Debug, Clone)]
pub struct CityObjects<SS: StringStorage, RR: ResourceRef> {
    /// Internal pool for storing CityObjects with efficient resource management
    inner: DefaultResourcePool<CityObject<SS, RR>, RR>,
}

impl<SS: StringStorage, RR: ResourceRef>
    CityObjectsTrait<SS, RR, CityObject<SS, RR>, CityObjectType<SS>, BBox> for CityObjects<SS, RR>
{
    fn new() -> Self {
        Self {
            inner: DefaultResourcePool::new(),
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: DefaultResourcePool::with_capacity(capacity),
        }
    }

    fn add(&mut self, city_object: CityObject<SS, RR>) -> RR {
        self.inner.add(city_object)
    }

    fn get(&self, id: RR) -> Option<&CityObject<SS, RR>> {
        self.inner.get(id)
    }

    fn get_mut(&mut self, id: RR) -> Option<&mut CityObject<SS, RR>> {
        self.inner.get_mut(id)
    }

    fn remove(&mut self, id: RR) -> Option<CityObject<SS, RR>> {
        self.inner.remove(id)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (RR, &'a CityObject<SS, RR>)>
    where
        CityObject<SS, RR>: 'a,
    {
        self.inner.iter()
    }

    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (RR, &'a mut CityObject<SS, RR>)>
    where
        CityObject<SS, RR>: 'a,
    {
        self.inner.iter_mut()
    }

    fn first(&self) -> Option<(RR, &CityObject<SS, RR>)> {
        self.inner.first()
    }

    fn last(&self) -> Option<(RR, &CityObject<SS, RR>)> {
        self.inner.last()
    }

    fn ids(&self) -> Vec<RR> {
        self.inner.iter().map(|(id, _)| id).collect()
    }

    fn add_many<I: IntoIterator<Item = CityObject<SS, RR>>>(&mut self, objects: I) -> Vec<RR> {
        objects.into_iter().map(|obj| self.add(obj)).collect()
    }

    fn clear(&mut self) {
        let ids: Vec<RR> = self.ids();
        for id in ids {
            self.remove(id);
        }
    }

    fn filter<F>(&self, predicate: F) -> Vec<(RR, &CityObject<SS, RR>)>
    where
        F: Fn(&CityObject<SS, RR>) -> bool,
    {
        self.inner
            .iter()
            .filter(|(_, obj)| predicate(obj))
            .collect()
    }
}

impl<SS: StringStorage, RR: ResourceRef> Default for CityObjects<SS, RR> {
    fn default() -> Self {
        Self::new()
    }
}

impl<SS: StringStorage, RR: ResourceRef> Extend<CityObject<SS, RR>> for CityObjects<SS, RR> {
    fn extend<T: IntoIterator<Item = CityObject<SS, RR>>>(&mut self, iter: T) {
        for obj in iter {
            self.add(obj);
        }
    }
}

impl<SS: StringStorage, RR: ResourceRef> FromIterator<CityObject<SS, RR>> for CityObjects<SS, RR> {
    fn from_iter<T: IntoIterator<Item = CityObject<SS, RR>>>(iter: T) -> Self {
        let mut objects = Self::new();
        objects.extend(iter);
        objects
    }
}

/// A CityObjects container using owned strings.
pub type OwnedCityObjects<RR> = CityObjects<OwnedStringStorage, RR>;

/// A CityObjects container using borrowed strings.
pub type BorrowedCityObjects<'a, RR> = CityObjects<BorrowedStringStorage<'a>, RR>;

#[derive(Debug, Default, Clone)]
pub struct CityObject<SS: StringStorage, RR: ResourceRef> {
    id: SS::String,
    type_cityobject: CityObjectType<SS>,
    geometry: Option<Vec<RR>>,
    attributes: Option<Attributes<SS, RR>>,
    geographical_extent: Option<BBox>,
    children: Option<Vec<RR>>,
    parents: Option<Vec<RR>>,
    extra: Option<Attributes<SS, RR>>,
}

impl<SS: StringStorage, RR: ResourceRef> CityObjectTrait<SS, RR, CityObjectType<SS>, BBox>
    for CityObject<SS, RR>
{
    fn new(id: SS::String, type_cityobject: CityObjectType<SS>) -> Self {
        Self {
            id,
            type_cityobject,
            geometry: None,
            attributes: None,
            geographical_extent: None,
            children: None,
            parents: None,
            extra: None,
        }
    }
    fn id(&self) -> &SS::String {
        &self.id
    }
    fn type_cityobject(&self) -> &CityObjectType<SS> {
        &self.type_cityobject
    }
    fn geometry(&self) -> Option<&Vec<RR>> {
        self.geometry.as_ref()
    }
    fn geometry_mut(&mut self) -> &mut Vec<RR> {
        self.geometry.get_or_insert_with(Vec::new)
    }
    fn attributes(&self) -> Option<&Attributes<SS, RR>> {
        self.attributes.as_ref()
    }
    fn attributes_mut(&mut self) -> &mut Attributes<SS, RR> {
        self.attributes.get_or_insert_with(Attributes::new)
    }
    fn geographical_extent(&self) -> Option<&BBox> {
        self.geographical_extent.as_ref()
    }
    fn set_geographical_extent(&mut self, bbox: Option<BBox>) {
        self.geographical_extent = bbox;
    }
    fn children(&self) -> Option<&Vec<RR>> {
        self.children.as_ref()
    }
    fn children_mut(&mut self) -> &mut Vec<RR> {
        self.children.get_or_insert_with(Vec::new)
    }
    fn parents(&self) -> Option<&Vec<RR>> {
        self.parents.as_ref()
    }
    fn parents_mut(&mut self) -> &mut Vec<RR> {
        self.parents.get_or_insert_with(Vec::new)
    }
    fn extra(&self) -> Option<&Attributes<SS, RR>> {
        self.extra.as_ref()
    }
    fn extra_mut(&mut self) -> &mut Attributes<SS, RR> {
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

impl<SS: StringStorage> CityObjectTypeTrait<SS> for CityObjectType<SS> {}

#[cfg(test)]
mod tests_cityobjects_container {
    use super::*;
    use crate::cityjson::attributes::AttributeValue;
    use crate::resources::pool::ResourceId32;

    #[test]
    fn test_basic_operations() {
        let mut objects = CityObjects::<OwnedStringStorage, ResourceId32>::new();

        // Create a test object
        let obj = CityObject::new("id-1".to_string(), CityObjectType::Building);

        // Add the object
        let id = objects.add(obj);

        // Check length
        assert_eq!(objects.len(), 1);
        assert!(!objects.is_empty());

        // Get the object
        let retrieved_obj = objects.get(id).unwrap();
        assert_eq!(retrieved_obj.type_cityobject(), &CityObjectType::Building);

        // Modify the object
        if let Some(obj_mut) = objects.get_mut(id) {
            obj_mut.attributes_mut().insert(
                "test".to_string(),
                AttributeValue::String("value".to_string()),
            );
        }

        // Verify modification
        let modified_obj = objects.get(id).unwrap();
        assert!(modified_obj.attributes().is_some());

        // Remove the object
        let removed_obj = objects.remove(id).unwrap();
        assert_eq!(removed_obj.type_cityobject(), &CityObjectType::Building);

        // Check the container is empty
        // assert_eq!(objects.len(), 0);
        // assert!(objects.is_empty());
    }

    #[test]
    fn test_filtering() {
        let mut objects = CityObjects::<OwnedStringStorage, ResourceId32>::new();

        // Add various types of objects
        objects.add(CityObject::new(
            "id-1".to_string(),
            CityObjectType::Building,
        ));
        objects.add(CityObject::new("id-2".to_string(), CityObjectType::Bridge));
        objects.add(CityObject::new(
            "id-3".to_string(),
            CityObjectType::Building,
        ));
    }

    #[test]
    fn test_bulk_operations() {
        let mut objects = CityObjects::<OwnedStringStorage, ResourceId32>::new();

        // Create multiple objects
        let objs = vec![
            CityObject::new("id-1".to_string(), CityObjectType::Building),
            CityObject::new("id-2".to_string(), CityObjectType::Bridge),
            CityObject::new("id-3".to_string(), CityObjectType::Road),
        ];

        // Add in bulk
        let ids = objects.add_many(objs);
        assert_eq!(ids.len(), 3);
        assert_eq!(objects.len(), 3);

        // Clear all
        // objects.clear();
        // assert_eq!(objects.len(), 0);
    }

    #[test]
    fn test_attribute_filtering() {
        let mut objects = CityObjects::<OwnedStringStorage, ResourceId32>::new();

        // Create objects with attributes
        let mut building1 = CityObject::new("id-1".to_string(), CityObjectType::Building);
        building1
            .attributes_mut()
            .insert("height".to_string(), AttributeValue::Float(15.0));
        objects.add(building1);

        let mut building2 = CityObject::new("id-2".to_string(), CityObjectType::Building);
        building2
            .attributes_mut()
            .insert("width".to_string(), AttributeValue::Float(30.0));
        objects.add(building2);
    }
}
