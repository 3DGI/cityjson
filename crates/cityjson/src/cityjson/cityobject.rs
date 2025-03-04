use crate::prelude::{Attributes, BBoxTrait, ResourceRef, StringStorage};
use std::fmt;

pub trait CityObjectTrait<
    SS: StringStorage,
    RR: ResourceRef,
    CoType: CityObjectTypeTrait,
    BBox: BBoxTrait,
>
{
    fn new(type_cityobject: CoType) -> Self;
    fn get_type(&self) -> &CoType;
    fn get_geometry(&self) -> Option<&Vec<RR>>;
    fn get_geometry_mut(&mut self) -> &mut Vec<RR>;
    fn get_attributes(&self) -> Option<&Attributes<SS>>;
    fn get_attributes_mut(&mut self) -> &mut Attributes<SS>;
    fn get_geographical_extent(&self) -> Option<&BBox>;
    fn set_geographical_extent(&mut self, bbox: Option<BBox>);
    fn get_children(&self) -> Option<&Vec<SS>>;
    fn get_children_mut(&mut self) -> &mut Vec<SS>;
    fn get_parents(&self) -> Option<&Vec<SS>>;
    fn get_parents_mut(&mut self) -> &mut Vec<SS>;
    fn get_extra(&self) -> Option<&Attributes<SS>>;
    fn get_extra_mut(&mut self) -> &mut Attributes<SS>;
}

pub trait CityObjectTypeTrait: Default + fmt::Display + Clone {}
