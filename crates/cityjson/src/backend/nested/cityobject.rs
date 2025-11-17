//! CityObject type for the nested backend.
//!

use crate::backend::nested::attributes::Attributes;
use crate::backend::nested::geometry::Geometry;
use crate::prelude::{BBox, StringStorage};
use crate::v2_0::CityObjectType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CityObject<SS: StringStorage> {
    pub type_co: CityObjectType<SS>,

    pub geometry: Option<Vec<Geometry<SS>>>,

    pub attributes: Option<Attributes<SS>>,

    pub geographical_extent: Option<BBox>,

    pub children: Option<Vec<String>>,

    pub parents: Option<Vec<String>>,

    pub extra: Option<Attributes<SS>>,
}

pub type CityObjects<SS> = HashMap<String, CityObject<SS>>;
