//! CityObject type for the nested backend.
//!

use crate::backend::nested::attributes::Attributes;
use crate::backend::nested::geometry::Geometry;
use crate::prelude::{BBox, StringStorage};
use crate::v2_0::CityObjectType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CityObject<SS: StringStorage> {
    type_co: CityObjectType<SS>,
    geometry: Option<Vec<Geometry<SS>>>,
    attributes: Option<Attributes<SS>>,
    geographical_extent: Option<BBox>,
    children: Option<Vec<String>>,
    parents: Option<Vec<String>>,
    extra: Option<Attributes<SS>>,
}

impl<SS: StringStorage> CityObject<SS> {
    // Constructor
    pub fn new(type_co: CityObjectType<SS>) -> Self {
        Self {
            type_co,
            geometry: None,
            attributes: None,
            geographical_extent: None,
            children: None,
            parents: None,
            extra: None,
        }
    }

    // Getters
    pub fn type_cityobject(&self) -> &CityObjectType<SS> {
        &self.type_co
    }

    pub fn geometry(&self) -> Option<&Vec<Geometry<SS>>> {
        self.geometry.as_ref()
    }

    pub fn attributes(&self) -> Option<&Attributes<SS>> {
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

    pub fn extra(&self) -> Option<&Attributes<SS>> {
        self.extra.as_ref()
    }

    // Mutators (auto-initialize Options)
    pub fn geometry_mut(&mut self) -> &mut Vec<Geometry<SS>> {
        self.geometry.get_or_insert_with(Vec::new)
    }

    pub fn attributes_mut(&mut self) -> &mut Attributes<SS> {
        self.attributes.get_or_insert_with(Attributes::new)
    }

    pub fn children_mut(&mut self) -> &mut Vec<String> {
        self.children.get_or_insert_with(Vec::new)
    }

    pub fn parents_mut(&mut self) -> &mut Vec<String> {
        self.parents.get_or_insert_with(Vec::new)
    }

    pub fn extra_mut(&mut self) -> &mut Attributes<SS> {
        self.extra.get_or_insert_with(Attributes::new)
    }

    pub fn set_geographical_extent(&mut self, bbox: Option<BBox>) {
        self.geographical_extent = bbox;
    }
}

pub type CityObjects<SS> = HashMap<String, CityObject<SS>>;
