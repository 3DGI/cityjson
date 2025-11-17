//! Core CityObject structures shared across CityJSON versions

use crate::cityjson::core::attributes::Attributes;
use crate::cityjson::core::metadata::BBox;
use crate::resources::pool::{DefaultResourcePool, ResourcePool, ResourceRef};
use crate::resources::storage::StringStorage;

/// Core CityObjects container structure that contains the data for all CityJSON versions.
/// Version-specific types wrap this core structure and implement methods via macros.
#[derive(Debug, Clone)]
pub struct CityObjectsCore<SS: StringStorage, RR: ResourceRef, CO> {
    pub(crate) inner: DefaultResourcePool<CO, RR>,
    _phantom: std::marker::PhantomData<SS>,
}

impl<SS: StringStorage, RR: ResourceRef, CO> Default for CityObjectsCore<SS, RR, CO> {
    fn default() -> Self {
        Self::new()
    }
}

impl<SS: StringStorage, RR: ResourceRef, CO> CityObjectsCore<SS, RR, CO> {
    pub fn new() -> Self {
        Self {
            inner: DefaultResourcePool::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: DefaultResourcePool::with_capacity(capacity),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn add(&mut self, city_object: CO) -> RR {
        self.inner.add(city_object)
    }

    pub fn get(&self, id: RR) -> Option<&CO> {
        self.inner.get(id)
    }

    pub fn get_mut(&mut self, id: RR) -> Option<&mut CO> {
        self.inner.get_mut(id)
    }

    pub fn remove(&mut self, id: RR) -> Option<CO> {
        self.inner.remove(id)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = (RR, &CO)> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (RR, &mut CO)> {
        self.inner.iter_mut()
    }

    pub fn first(&self) -> Option<(RR, &CO)> {
        self.inner.first()
    }

    pub fn last(&self) -> Option<(RR, &CO)> {
        self.inner.last()
    }

    pub fn ids(&self) -> Vec<RR> {
        self.inner.iter().map(|(id, _)| id).collect()
    }

    pub fn add_many<I: IntoIterator<Item = CO>>(&mut self, objects: I) -> Vec<RR> {
        objects.into_iter().map(|obj| self.add(obj)).collect()
    }

    pub fn clear(&mut self) {
        self.inner.clear()
    }

    pub fn filter<F>(&self, predicate: F) -> Vec<(RR, &CO)>
    where
        F: Fn(&CO) -> bool,
    {
        self.inner
            .iter()
            .filter(|(_, obj)| predicate(obj))
            .collect()
    }
}

/// Core CityObject structure that contains the data for all CityJSON versions.
/// Version-specific types wrap this core structure and implement methods via macros.
#[derive(Debug, Default, Clone)]
pub struct CityObjectCore<SS: StringStorage, RR: ResourceRef, CoType> {
    id: SS::String,
    type_cityobject: CoType,
    geometry: Option<Vec<RR>>,
    attributes: Option<Attributes<SS>>,
    geographical_extent: Option<BBox>,
    children: Option<Vec<RR>>,
    parents: Option<Vec<RR>>,
    extra: Option<Attributes<SS>>,
}

impl<SS: StringStorage, RR: ResourceRef, CoType> CityObjectCore<SS, RR, CoType> {
    pub fn new(id: SS::String, type_cityobject: CoType) -> Self {
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

    pub fn id(&self) -> &SS::String {
        &self.id
    }

    pub fn type_cityobject(&self) -> &CoType {
        &self.type_cityobject
    }

    pub fn geometry(&self) -> Option<&Vec<RR>> {
        self.geometry.as_ref()
    }

    pub fn geometry_mut(&mut self) -> &mut Vec<RR> {
        self.geometry.get_or_insert_with(Vec::new)
    }

    pub fn attributes(&self) -> Option<&Attributes<SS>> {
        self.attributes.as_ref()
    }

    pub fn attributes_mut(&mut self) -> &mut Attributes<SS> {
        self.attributes.get_or_insert_with(Attributes::new)
    }

    pub fn geographical_extent(&self) -> Option<&BBox> {
        self.geographical_extent.as_ref()
    }

    pub fn set_geographical_extent(&mut self, bbox: Option<BBox>) {
        self.geographical_extent = bbox;
    }

    pub fn children(&self) -> Option<&Vec<RR>> {
        self.children.as_ref()
    }

    pub fn children_mut(&mut self) -> &mut Vec<RR> {
        self.children.get_or_insert_with(Vec::new)
    }

    pub fn parents(&self) -> Option<&Vec<RR>> {
        self.parents.as_ref()
    }

    pub fn parents_mut(&mut self) -> &mut Vec<RR> {
        self.parents.get_or_insert_with(Vec::new)
    }

    pub fn extra(&self) -> Option<&Attributes<SS>> {
        self.extra.as_ref()
    }

    pub fn extra_mut(&mut self) -> &mut Attributes<SS> {
        self.extra.get_or_insert_with(Attributes::new)
    }
}
