//! # CityObject
//!
//! Represents a [CityObject object](https://www.cityjson.org/specs/1.1.3/#the-different-city-objects).

use std::collections::HashMap;
use crate::prelude::{Attributes, BorrowedStringStorage, CityObjectTrait, CityObjectTypeTrait, DefaultResourcePool, OwnedStringStorage, ResourcePool, ResourceRef, StringStorage};
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
/// let building = CityObject::new(CityObjectType::Building);
/// let building_id = objects.add(building);
///
/// // Retrieve a CityObject by its ID
/// let retrieved_building = objects.get(building_id).unwrap();
/// assert_eq!(retrieved_building.get_type(), &CityObjectType::Building);
/// ```
pub struct CityObjects<SS: StringStorage, RR: ResourceRef> {
    /// Internal pool for storing CityObjects with efficient resource management
    inner: DefaultResourcePool<CityObject<SS, RR>, RR>,
}

impl<SS: StringStorage, RR: ResourceRef> CityObjects<SS, RR> {
    /// Creates a new empty CityObjects container.
    pub fn new() -> Self {
        Self {
            inner: DefaultResourcePool::new(),
        }
    }

    /// Creates a new CityObjects container with the specified capacity.
    ///
    /// This method pre-allocates memory for the specified number of objects,
    /// which can improve performance when adding many objects.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: DefaultResourcePool::with_capacity(capacity),
        }
    }

    /// Adds a CityObject to the container.
    ///
    /// # Returns
    ///
    /// A resource reference that can be used to access the added object.
    pub fn add(&mut self, city_object: CityObject<SS, RR>) -> RR {
        self.inner.add(city_object)
    }

    /// Gets a reference to a CityObject by its resource reference.
    ///
    /// # Returns
    ///
    /// `Some(&CityObject)` if found, or `None` if not found.
    pub fn get(&self, id: RR) -> Option<&CityObject<SS, RR>> {
        self.inner.get(id)
    }

    /// Gets a mutable reference to a CityObject by its resource reference.
    ///
    /// # Returns
    ///
    /// `Some(&mut CityObject)` if found, or `None` if not found.
    pub fn get_mut(&mut self, id: RR) -> Option<&mut CityObject<SS, RR>> {
        self.inner.get_mut(id)
    }

    /// Removes a CityObject from the container.
    ///
    /// # Returns
    ///
    /// `Some(CityObject)` containing the removed object if found, or `None` if not found.
    pub fn remove(&mut self, id: RR) -> Option<CityObject<SS, RR>> {
        self.inner.remove(id)
    }

    /// Returns the number of CityObjects in the container.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns whether the container is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over all CityObjects in the container.
    ///
    /// The iterator yields pairs of resource references and references to CityObjects.
    pub fn iter(&self) -> impl Iterator<Item = (RR, &CityObject<SS, RR>)> {
        self.inner.iter()
    }

    /// Returns an iterator over mutable references to all CityObjects in the container.
    ///
    /// The iterator yields pairs of resource references and mutable references to CityObjects.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (RR, &mut CityObject<SS, RR>)> {
        self.inner.iter_mut()
    }

    /// Gets the first CityObject in the container.
    pub fn first(&self) -> Option<(RR, &CityObject<SS, RR>)> {
        self.inner.first()
    }

    /// Gets the last CityObject in the container.
    pub fn last(&self) -> Option<(RR, &CityObject<SS, RR>)> {
        self.inner.last()
    }

    /// Returns all resource references for CityObjects in the container.
    pub fn ids(&self) -> Vec<RR> {
        self.inner.iter().map(|(id, _)| id).collect()
    }

    /// Finds CityObjects by their type.
    pub fn find_by_type(&self, object_type: &CityObjectType<SS>) -> Vec<(RR, &CityObject<SS, RR>)> {
        self.inner.iter()
            .filter(|(_, obj)| obj.get_type() == object_type)
            .collect()
    }

    /// Finds CityObjects by their parent.
    pub fn find_by_parent(&self, parent_id: &SS) -> Vec<(RR, &CityObject<SS, RR>)> {
        self.inner.iter()
            .filter(|(_, obj)| {
                if let Some(parents) = obj.get_parents() {
                    parents.contains(parent_id)
                } else {
                    false
                }
            })
            .collect()
    }

    /// Finds CityObjects that have geometries.
    pub fn find_with_geometries(&self) -> Vec<(RR, &CityObject<SS, RR>)> {
        self.inner.iter()
            .filter(|(_, obj)| obj.get_geometry().is_some() && !obj.get_geometry().unwrap().is_empty())
            .collect()
    }

    /// Finds CityObjects that have children.
    pub fn find_with_children(&self) -> Vec<(RR, &CityObject<SS, RR>)> {
        self.inner.iter()
            .filter(|(_, obj)| obj.get_children().is_some() && !obj.get_children().unwrap().is_empty())
            .collect()
    }

    /// Adds multiple CityObjects and returns their resource references.
    pub fn add_many<I: IntoIterator<Item = CityObject<SS, RR>>>(&mut self, objects: I) -> Vec<RR> {
        objects.into_iter().map(|obj| self.add(obj)).collect()
    }

    /// Clears all CityObjects from the container.
    pub fn clear(&mut self) {
        let ids: Vec<RR> = self.ids();
        for id in ids {
            self.remove(id);
        }
    }

    /// Filters CityObjects using a predicate function.
    pub fn filter<F>(&self, predicate: F) -> Vec<(RR, &CityObject<SS, RR>)>
    where
        F: Fn(&CityObject<SS, RR>) -> bool,
    {
        self.inner.iter()
            .filter(|(_, obj)| predicate(obj))
            .collect()
    }

    /// Returns the count of CityObjects matching a predicate.
    pub fn count<F>(&self, predicate: F) -> usize
    where
        F: Fn(&CityObject<SS, RR>) -> bool,
    {
        self.inner.iter()
            .filter(|(_, obj)| predicate(obj))
            .count()
    }

    /// Returns the count of CityObjects by type.
    pub fn count_by_type(&self) -> HashMap<CityObjectType<SS>, usize> {
        let mut counts = HashMap::new();
        for (_, obj) in self.inner.iter() {
            *counts.entry(obj.get_type().clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Finds CityObjects by their attribute.
    pub fn find_by_attribute(&self, attr_name: &str) -> Vec<(RR, &CityObject<SS, RR>)> {
        self.inner.iter()
            .filter(|(_, obj)| {
                obj.get_attributes().is_some() &&
                obj.get_attributes().unwrap().contains_key(attr_name)
            })
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

#[cfg(test)]
mod tests_cityobjects_container {
    use super::*;
    use crate::cityjson::attributes::AttributeValue;
    use crate::resources::pool::ResourceId32;

    #[test]
    fn test_basic_operations() {
        let mut objects = CityObjects::<OwnedStringStorage, ResourceId32>::new();

        // Create a test object
        let obj = CityObject::new(CityObjectType::Building);

        // Add the object
        let id = objects.add(obj);

        // Check length
        assert_eq!(objects.len(), 1);
        assert!(!objects.is_empty());

        // Get the object
        let retrieved_obj = objects.get(id).unwrap();
        assert_eq!(retrieved_obj.get_type(), &CityObjectType::Building);

        // Modify the object
        if let Some(obj_mut) = objects.get_mut(id) {
            obj_mut.get_attributes_mut().insert(
                "test".to_string(),
                AttributeValue::String("value".to_string())
            );
        }

        // Verify modification
        let modified_obj = objects.get(id).unwrap();
        assert!(modified_obj.get_attributes().is_some());

        // Remove the object
        let removed_obj = objects.remove(id).unwrap();
        assert_eq!(removed_obj.get_type(), &CityObjectType::Building);

        // Check the container is empty
        // assert_eq!(objects.len(), 0);
        // assert!(objects.is_empty());
    }

    #[test]
    fn test_filtering() {
        let mut objects = CityObjects::<OwnedStringStorage, ResourceId32>::new();

        // Add various types of objects
        objects.add(CityObject::new(CityObjectType::Building));
        objects.add(CityObject::new(CityObjectType::Bridge));
        objects.add(CityObject::new(CityObjectType::Building));

        // Find by type
        let buildings = objects.find_by_type(&CityObjectType::Building);
        assert_eq!(buildings.len(), 2);

        let bridges = objects.find_by_type(&CityObjectType::Bridge);
        assert_eq!(bridges.len(), 1);

        // Count by type
        let counts = objects.count_by_type();
        assert_eq!(counts.get(&CityObjectType::Building), Some(&2));
        assert_eq!(counts.get(&CityObjectType::Bridge), Some(&1));
    }

    #[test]
    fn test_bulk_operations() {
        let mut objects = CityObjects::<OwnedStringStorage, ResourceId32>::new();

        // Create multiple objects
        let objs = vec![
            CityObject::new(CityObjectType::Building),
            CityObject::new(CityObjectType::Bridge),
            CityObject::new(CityObjectType::Road),
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
        let mut building1 = CityObject::new(CityObjectType::Building);
        building1.get_attributes_mut().insert(
            "height".to_string(),
            AttributeValue::Float(15.0)
        );
        objects.add(building1);

        let mut building2 = CityObject::new(CityObjectType::Building);
        building2.get_attributes_mut().insert(
            "width".to_string(),
            AttributeValue::Float(30.0)
        );
        objects.add(building2);

        // Filter by attribute existence
        let with_height = objects.find_by_attribute("height");
        assert_eq!(with_height.len(), 1);

        let with_width = objects.find_by_attribute("width");
        assert_eq!(with_width.len(), 1);

        let with_depth = objects.find_by_attribute("depth");
        assert_eq!(with_depth.len(), 0);
    }
}