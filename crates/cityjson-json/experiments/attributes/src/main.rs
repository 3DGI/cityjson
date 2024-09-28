#![allow(unused_imports, dead_code, unused_assignments, unused_mut)]

mod serde_cityjson {
    use std::borrow::Cow;
    use std::collections::HashMap as Map;

    #[derive(Debug)]
    pub struct CityModel<'cm> {
        pub cityobjects: CityObjects<'cm>,
    }
    pub type CityObjects<'cm> = Map<Cow<'cm, str>, CityObject<'cm>>;
    #[derive(Debug)]
    pub struct CityObject<'cm> {
        pub attributes: Option<Attr<'cm>>,
    }
    pub type Attributes<'cm> = Cow<'cm, serde_json_borrow::Value<'cm>>;

    #[derive(Debug)]
    pub enum Attr<'cm> {
        Borrowed(serde_json_borrow::Value<'cm>),
        Owned(serde_json::Value)
    }
}

use crate::serde_cityjson::{CityModel, CityObjects, CityObject, Attr};
use serde_json;
use std::ops::{Deref, Index};
use std::borrow::Cow;

fn populate_cm<'cm>(mut cm: CityModel<'cm>) -> CityModel<'cm> {
    let mut co = CityObject { attributes: None };
    let attributes_value = serde_json::json!({"key": "value"});
    assert!(attributes_value.is_object());
    co.attributes = Some(Attr::Owned(attributes_value));
    let mut cos = CityObjects::new();
    cos.insert(Cow::from("co-key"), co);
    cm.cityobjects = cos;
    cm
}

fn main() {
    let mut cm = CityModel {
        cityobjects: CityObjects::new(),
    };
    cm = populate_cm(cm);
    dbg!(cm);
}
