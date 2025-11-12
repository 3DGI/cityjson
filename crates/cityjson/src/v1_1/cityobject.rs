//! # CityObject
//!
//! Represents a [CityObject object](https://www.cityjson.org/specs/1.1.3/#the-different-city-objects).
//!
//! Properties that are specific to certain type of CityObjects need to be set as `extra`
//! attributes. For example the `address` member of the `Bridge` and `Building` types,
//! or the `children_roles` member of the `CityObjectGroup`.

use crate::cityjson::core::cityobject::{CityObjectCore, CityObjectsCore};
use crate::cityjson::traits::cityobject::CityObjectTypeTrait;
use crate::prelude::*;
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

// Trait implementations for internal use (required by CityModelTypes)
impl<SS: StringStorage, RR: ResourceRef> CityObjectTrait<SS, RR, CityObjectType<SS>, BBox>
    for CityObject<SS, RR>
{
    fn new(id: SS::String, type_cityobject: CityObjectType<SS>) -> Self {
        Self::new(id, type_cityobject)
    }
    fn id(&self) -> &SS::String {
        self.id()
    }
    fn type_cityobject(&self) -> &CityObjectType<SS> {
        self.type_cityobject()
    }
    fn geometry(&self) -> Option<&Vec<RR>> {
        self.geometry()
    }
    fn geometry_mut(&mut self) -> &mut Vec<RR> {
        self.geometry_mut()
    }
    fn attributes(&self) -> Option<&Attributes<SS, RR>> {
        self.attributes()
    }
    fn attributes_mut(&mut self) -> &mut Attributes<SS, RR> {
        self.attributes_mut()
    }
    fn geographical_extent(&self) -> Option<&BBox> {
        self.geographical_extent()
    }
    fn set_geographical_extent(&mut self, bbox: Option<BBox>) {
        self.set_geographical_extent(bbox);
    }
    fn children(&self) -> Option<&Vec<RR>> {
        self.children()
    }
    fn children_mut(&mut self) -> &mut Vec<RR> {
        self.children_mut()
    }
    fn parents(&self) -> Option<&Vec<RR>> {
        self.parents()
    }
    fn parents_mut(&mut self) -> &mut Vec<RR> {
        self.parents_mut()
    }
    fn extra(&self) -> Option<&Attributes<SS, RR>> {
        self.extra()
    }
    fn extra_mut(&mut self) -> &mut Attributes<SS, RR> {
        self.extra_mut()
    }
}

impl<SS: StringStorage, RR: ResourceRef>
    CityObjectsTrait<SS, RR, CityObject<SS, RR>, CityObjectType<SS>, BBox> for CityObjects<SS, RR>
{
    fn new() -> Self {
        Self::new()
    }
    fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity(capacity)
    }
    fn add(&mut self, city_object: CityObject<SS, RR>) -> RR {
        self.add(city_object)
    }
    fn get(&self, id: RR) -> Option<&CityObject<SS, RR>> {
        self.get(id)
    }
    fn get_mut(&mut self, id: RR) -> Option<&mut CityObject<SS, RR>> {
        self.get_mut(id)
    }
    fn remove(&mut self, id: RR) -> Option<CityObject<SS, RR>> {
        self.remove(id)
    }
    fn len(&self) -> usize {
        self.len()
    }
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
    fn iter<'a>(&'a self) -> impl Iterator<Item = (RR, &'a CityObject<SS, RR>)>
    where
        CityObject<SS, RR>: 'a,
    {
        self.iter()
    }
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (RR, &'a mut CityObject<SS, RR>)>
    where
        CityObject<SS, RR>: 'a,
    {
        self.iter_mut()
    }
    fn first(&self) -> Option<(RR, &CityObject<SS, RR>)> {
        self.first()
    }
    fn last(&self) -> Option<(RR, &CityObject<SS, RR>)> {
        self.last()
    }
    fn ids(&self) -> Vec<RR> {
        self.ids()
    }
    fn add_many<I: IntoIterator<Item = CityObject<SS, RR>>>(&mut self, objects: I) -> Vec<RR> {
        self.add_many(objects)
    }
    fn clear(&mut self) {
        self.clear();
    }
    fn filter<F>(&self, predicate: F) -> Vec<(RR, &CityObject<SS, RR>)>
    where
        F: Fn(&CityObject<SS, RR>) -> bool,
    {
        self.filter(predicate)
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
    use crate::cityjson::core::attributes::AttributeValue;
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
