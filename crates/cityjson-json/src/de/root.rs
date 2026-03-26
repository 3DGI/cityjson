use std::collections::HashMap;
use std::fmt;

use serde::de::{self, DeserializeSeed, MapAccess, Visitor};
use serde::Deserialize;

use crate::de::attributes::RawAttribute;
use crate::de::cityobjects::{BufferedCityObject, CityObjectsBufferSeed};
use crate::de::sections::{
    RawAppearanceSection, RawExtension, RawGeometryTemplatesSection, RawMetadataSection,
};
use crate::errors::{Error, Result};

pub(crate) struct ParsedRoot<'a> {
    pub(crate) type_name: &'a str,
    pub(crate) version: Option<&'a str>,
    pub(crate) transform: Option<RawTransform>,
    pub(crate) vertices: Vec<[f64; 3]>,
    pub(crate) metadata: Option<RawMetadataSection<'a>>,
    pub(crate) extensions: Option<HashMap<&'a str, RawExtension<'a>>>,
    pub(crate) cityobjects: Vec<BufferedCityObject<'a>>,
    pub(crate) appearance: Option<RawAppearanceSection<'a>>,
    pub(crate) geometry_templates: Option<RawGeometryTemplatesSection<'a>>,
    pub(crate) extra: HashMap<&'a str, RawAttribute<'a>>,
}

#[derive(Deserialize)]
pub(crate) struct RawTransform {
    pub(crate) scale: [f64; 3],
    pub(crate) translate: [f64; 3],
}

pub(crate) fn parse_root(input: &str) -> Result<ParsedRoot<'_>> {
    let mut deserializer = serde_json::Deserializer::from_str(input);
    let root = RootSeed
        .deserialize(&mut deserializer)
        .map_err(Error::from)?;
    deserializer.end().map_err(Error::from)?;
    Ok(root)
}

struct RootSeed;

impl<'de> DeserializeSeed<'de> for RootSeed {
    type Value = ParsedRoot<'de>;

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(RootVisitor)
    }
}

struct RootVisitor;

impl<'de> Visitor<'de> for RootVisitor {
    type Value = ParsedRoot<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a CityJSON root object")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut type_name = None;
        let mut version = None;
        let mut transform = None;
        let mut vertices = None;
        let mut metadata = None;
        let mut extensions = None;
        let mut cityobjects = None;
        let mut appearance = None;
        let mut geometry_templates = None;
        let mut extra = HashMap::with_capacity(map.size_hint().unwrap_or(0));

        while let Some(key) = map.next_key::<&'de str>()? {
            match key {
                "type" => set_once(&mut type_name, "type", map.next_value()?)?,
                "version" => set_once(&mut version, "version", map.next_value()?)?,
                "transform" => set_once(&mut transform, "transform", map.next_value()?)?,
                "vertices" => set_once(&mut vertices, "vertices", map.next_value()?)?,
                "metadata" => set_once(&mut metadata, "metadata", map.next_value()?)?,
                "extensions" => set_once(&mut extensions, "extensions", map.next_value()?)?,
                "CityObjects" => set_once(
                    &mut cityobjects,
                    "CityObjects",
                    map.next_value_seed(CityObjectsBufferSeed)?,
                )?,
                "appearance" => set_once(&mut appearance, "appearance", map.next_value()?)?,
                "geometry-templates" => set_once(
                    &mut geometry_templates,
                    "geometry-templates",
                    map.next_value()?,
                )?,
                _ => {
                    extra.insert(key, map.next_value()?);
                }
            }
        }

        Ok(ParsedRoot {
            type_name: type_name.ok_or_else(|| de::Error::missing_field("type"))?,
            version,
            transform,
            vertices: vertices.ok_or_else(|| de::Error::missing_field("vertices"))?,
            metadata,
            extensions,
            cityobjects: cityobjects.ok_or_else(|| de::Error::missing_field("CityObjects"))?,
            appearance,
            geometry_templates,
            extra,
        })
    }
}

fn set_once<T, E>(slot: &mut Option<T>, field: &'static str, value: T) -> std::result::Result<(), E>
where
    E: de::Error,
{
    if slot.is_some() {
        return Err(de::Error::duplicate_field(field));
    }
    *slot = Some(value);
    Ok(())
}
