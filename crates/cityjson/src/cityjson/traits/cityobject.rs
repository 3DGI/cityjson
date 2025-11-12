use crate::prelude::{Attributes, ResourceRef, StringStorage};
use std::fmt;

pub trait CityObjectsTrait<SS, RR, Co, CoType, BBox>
where
    SS: StringStorage,
    RR: ResourceRef,
    Co: CityObjectTrait<SS, RR, CoType, BBox>,
    CoType: CityObjectTypeTrait<SS>,
{
    /// Creates a new empty CityObjects container.
    fn new() -> Self;
    /// Creates a new CityObjects container with the specified capacity.
    ///
    /// This method pre-allocates memory for the specified number of objects,
    /// which can improve performance when adding many objects.
    fn with_capacity(capacity: usize) -> Self;
    /// Adds a CityObject to the container.
    ///
    /// # Returns
    ///
    /// A resource reference that can be used to access the added object.
    fn add(&mut self, city_object: Co) -> RR;
    /// Gets a reference to a CityObject by its resource reference.
    ///
    /// # Returns
    ///
    /// `Some(&CityObject)` if found, or `None` if not found.
    fn get(&self, id: RR) -> Option<&Co>;
    /// Gets a mutable reference to a CityObject by its resource reference.
    ///
    /// # Returns
    ///
    /// `Some(&mut CityObject)` if found, or `None` if not found.
    fn get_mut(&mut self, id: RR) -> Option<&mut Co>;
    /// Removes a CityObject from the container.
    ///
    /// # Returns
    ///
    /// `Some(CityObject)` containing the removed object if found, or `None` if not found.
    fn remove(&mut self, id: RR) -> Option<Co>;
    /// Returns the number of CityObjects in the container.
    fn len(&self) -> usize;
    /// Returns whether the container is empty.
    fn is_empty(&self) -> bool;
    /// Returns an iterator over all CityObjects in the container.
    ///
    /// The iterator yields pairs of resource references and references to CityObjects.
    fn iter<'a>(&'a self) -> impl Iterator<Item = (RR, &'a Co)>
    where
        Co: 'a;
    /// Returns an iterator over mutable references to all CityObjects in the container.
    ///
    /// The iterator yields pairs of resource references and mutable references to CityObjects.
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = (RR, &'a mut Co)>
    where
        Co: 'a;
    /// Gets the first CityObject in the container.
    fn first(&self) -> Option<(RR, &Co)>;
    /// Gets the last CityObject in the container.
    fn last(&self) -> Option<(RR, &Co)>;
    /// Returns all resource references for CityObjects in the container.
    fn ids(&self) -> Vec<RR>;
    /// Adds multiple CityObjects and returns their resource references.
    fn add_many<I: IntoIterator<Item = Co>>(&mut self, objects: I) -> Vec<RR>;
    /// Clears all CityObjects from the container.
    fn clear(&mut self);
    /// Filters CityObjects using a predicate function.
    fn filter<F>(&self, predicate: F) -> Vec<(RR, &Co)>
    where
        F: Fn(&Co) -> bool;
}

pub trait CityObjectTrait<SS: StringStorage, RR: ResourceRef, CoType: CityObjectTypeTrait<SS>, BBox>
{
    fn new(id: SS::String, type_cityobject: CoType) -> Self;
    fn id(&self) -> &SS::String;
    fn type_cityobject(&self) -> &CoType;
    fn geometry(&self) -> Option<&Vec<RR>>;
    fn geometry_mut(&mut self) -> &mut Vec<RR>;
    fn attributes(&self) -> Option<&Attributes<SS, RR>>;
    fn attributes_mut(&mut self) -> &mut Attributes<SS, RR>;
    fn geographical_extent(&self) -> Option<&BBox>;
    fn set_geographical_extent(&mut self, bbox: Option<BBox>);
    fn children(&self) -> Option<&Vec<RR>>;
    fn children_mut(&mut self) -> &mut Vec<RR>;
    fn parents(&self) -> Option<&Vec<RR>>;
    fn parents_mut(&mut self) -> &mut Vec<RR>;
    fn extra(&self) -> Option<&Attributes<SS, RR>>;
    fn extra_mut(&mut self) -> &mut Attributes<SS, RR>;
}

pub trait CityObjectTypeTrait<SS: StringStorage>: Default + fmt::Display + Clone {}
