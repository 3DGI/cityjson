use std::collections::HashMap;

use serde::Deserialize;
use serde_json::value::RawValue;

use crate::de::attributes::RawAttribute;

/// Thin root shell: scalars and small fields are typed; major sections are
/// deferred as `&RawValue` parser boundaries.
#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
pub(crate) struct RawRoot<'a> {
    #[serde(rename = "type", borrow)]
    pub(crate) type_name: &'a str,
    #[serde(default, borrow)]
    pub(crate) version: Option<&'a str>,
    #[serde(default)]
    pub(crate) transform: Option<RawTransform>,
    pub(crate) vertices: Vec<[f64; 3]>,
    #[serde(default, borrow)]
    pub(crate) metadata: Option<&'a RawValue>,
    #[serde(default, borrow)]
    pub(crate) extensions: Option<&'a RawValue>,
    #[serde(rename = "CityObjects", borrow)]
    pub(crate) cityobjects: &'a RawValue,
    #[serde(default, borrow)]
    pub(crate) appearance: Option<&'a RawValue>,
    #[serde(rename = "geometry-templates", default, borrow)]
    pub(crate) geometry_templates: Option<&'a RawValue>,
    #[serde(flatten, borrow)]
    pub(crate) extra: HashMap<&'a str, RawAttribute<'a>>,
}

#[derive(Deserialize)]
pub(crate) struct RawTransform {
    pub(crate) scale: [f64; 3],
    pub(crate) translate: [f64; 3],
}
