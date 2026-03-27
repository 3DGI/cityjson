use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use serde::de::{self, DeserializeSeed, MapAccess, Visitor};
use serde::Deserialize;
use serde_json::value::RawValue;

use cityjson::resources::handles::CityObjectHandle;
use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{Attributes, BBox, CityModel, CityObject, CityObjectIdentifier};

use crate::de::attributes::{AttributeValueSeed, OptionalAttributesSeed};
use crate::de::geometry::{import_stream_geometry, GeometryResources, StreamingGeometry};
use crate::de::parse::ParseStringStorage;
use crate::de::profiling::timed;
use crate::de::validation::parse_cityobject_type;
use crate::errors::{Error, Result};

pub(crate) struct StreamingCityObject<'de, SS: StringStorage> {
    pub(crate) type_name: &'de str,
    pub(crate) geographical_extent: Option<[f64; 6]>,
    pub(crate) attributes: Option<Attributes<SS>>,
    pub(crate) parents: Vec<&'de str>,
    pub(crate) children: Vec<&'de str>,
    pub(crate) geometry: Option<Vec<StreamingGeometry<'de>>>,
    pub(crate) extra: Attributes<SS>,
}

impl<'de, SS> Deserialize<'de> for StreamingCityObject<'de, SS>
where
    SS: ParseStringStorage<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(StreamingCityObjectVisitor::<SS>(PhantomData))
    }
}

struct StreamingCityObjectVisitor<SS>(PhantomData<SS>);

impl<'de, SS> Visitor<'de> for StreamingCityObjectVisitor<SS>
where
    SS: ParseStringStorage<'de>,
{
    type Value = StreamingCityObject<'de, SS>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a CityObject")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut type_name = None;
        let mut geographical_extent = None;
        let mut attributes = None;
        let mut parents = None;
        let mut children = None;
        let mut geometry = None;
        let mut extra = Attributes::<SS>::with_capacity(map.size_hint().unwrap_or(0));

        while let Some(key) = map.next_key::<&'de str>()? {
            match key {
                "type" => {
                    if type_name.is_some() {
                        return Err(de::Error::duplicate_field("type"));
                    }
                    type_name = Some(map.next_value()?);
                }
                "geographicalExtent" => {
                    if geographical_extent.is_some() {
                        return Err(de::Error::duplicate_field("geographicalExtent"));
                    }
                    geographical_extent = Some(map.next_value()?);
                }
                "attributes" => {
                    if attributes.is_some() {
                        return Err(de::Error::duplicate_field("attributes"));
                    }
                    attributes = Some(timed("cityobjects.attributes", || {
                        map.next_value_seed(OptionalAttributesSeed::<SS>::new())
                            .map_err(de::Error::custom)
                    })?);
                }
                "parents" => {
                    if parents.is_some() {
                        return Err(de::Error::duplicate_field("parents"));
                    }
                    parents = Some(map.next_value()?);
                }
                "children" => {
                    if children.is_some() {
                        return Err(de::Error::duplicate_field("children"));
                    }
                    children = Some(map.next_value()?);
                }
                "geometry" => {
                    if geometry.is_some() {
                        return Err(de::Error::duplicate_field("geometry"));
                    }
                    geometry = Some(map.next_value()?);
                }
                _ => {
                    let value = timed("cityobjects.extra", || {
                        map.next_value_seed(AttributeValueSeed::<SS>::new())
                            .map_err(de::Error::custom)
                    })?;
                    extra.insert(SS::store(key), value);
                }
            }
        }

        Ok(StreamingCityObject {
            type_name: type_name.ok_or_else(|| de::Error::missing_field("type"))?,
            geographical_extent: geographical_extent.flatten(),
            attributes: attributes.flatten(),
            parents: parents.unwrap_or_default(),
            children: children.unwrap_or_default(),
            geometry: geometry.flatten(),
            extra,
        })
    }
}

struct PendingRelations<'de> {
    source_id: &'de str,
    source_handle: CityObjectHandle,
    parents: Vec<&'de str>,
    children: Vec<&'de str>,
}

pub(crate) fn import_cityobjects<'de, SS>(
    cityobjects: &'de RawValue,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<()>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let mut deserializer = serde_json::Deserializer::from_str(cityobjects.get());
    timed("cityobjects.deserialize", || {
        CityObjectsImportSeed { model, resources }
            .deserialize(&mut deserializer)
            .map_err(Error::from)
    })?;
    deserializer.end().map_err(Error::from)?;
    Ok(())
}

struct CityObjectsImportSeed<'a, SS: StringStorage> {
    model: &'a mut CityModel<u32, SS>,
    resources: &'a GeometryResources,
}

impl<'de, SS> DeserializeSeed<'de> for CityObjectsImportSeed<'_, SS>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(CityObjectsImportVisitor {
            model: self.model,
            resources: self.resources,
        })
    }
}

struct CityObjectsImportVisitor<'a, SS: StringStorage> {
    model: &'a mut CityModel<u32, SS>,
    resources: &'a GeometryResources,
}

impl<'de, SS> Visitor<'de> for CityObjectsImportVisitor<'_, SS>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a CityObjects map")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let capacity = map.size_hint().unwrap_or(0);
        if capacity != 0 {
            self.model
                .cityobjects_mut()
                .reserve(capacity)
                .map_err(de::Error::custom)?;
        }

        let mut handle_by_id = HashMap::with_capacity(capacity);
        let mut pending = Vec::with_capacity(capacity);

        while let Some(id) = map.next_key::<&'de str>()? {
            let imported = timed("cityobjects.import_object", || {
                import_cityobject::<SS>(id, map.next_value()?, self.model, self.resources)
                    .map_err(de::Error::custom)
            })?;
            handle_by_id.insert(id, imported.source_handle);
            pending.push(imported);
        }

        timed("cityobjects.resolve_relations", || {
            resolve_relations(pending, &handle_by_id, self.model).map_err(de::Error::custom)
        })
    }
}

fn import_cityobject<'de, SS>(
    id: &'de str,
    raw_object: StreamingCityObject<'de, SS>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<PendingRelations<'de>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let type_cityobject = timed("cityobjects.parse_type", || {
        parse_cityobject_type::<SS>(raw_object.type_name)
    })?;
    let mut cityobject = CityObject::new(CityObjectIdentifier::new(SS::store(id)), type_cityobject);

    if let Some(extent) = raw_object.geographical_extent {
        cityobject.set_geographical_extent(Some(BBox::from(extent)));
    }
    if let Some(attributes) = raw_object.attributes {
        *cityobject.attributes_mut() = attributes;
    }
    if !raw_object.extra.is_empty() {
        *cityobject.extra_mut() = raw_object.extra;
    }
    if let Some(geometries) = raw_object.geometry {
        if geometries.is_empty() {
            cityobject.clear_geometry();
        } else {
            for geometry in geometries {
                let handle = timed("cityobjects.geometry", || {
                    import_stream_geometry::<SS>(geometry, model, resources)
                })?;
                cityobject.add_geometry(handle);
            }
        }
    }

    let handle = timed("cityobjects.add_object", || {
        model.cityobjects_mut().add(cityobject)
    })?;
    Ok(PendingRelations {
        source_id: id,
        source_handle: handle,
        parents: raw_object.parents,
        children: raw_object.children,
    })
}

fn resolve_relations<'de, SS>(
    pending: Vec<PendingRelations<'de>>,
    handle_by_id: &HashMap<&'de str, CityObjectHandle>,
    model: &mut CityModel<u32, SS>,
) -> Result<()>
where
    SS: StringStorage,
{
    for relation in pending {
        let cityobject = model
            .cityobjects_mut()
            .get_mut(relation.source_handle)
            .ok_or_else(|| {
                Error::InvalidValue(format!(
                    "missing inserted CityObject for '{}'",
                    relation.source_id
                ))
            })?;

        for parent in relation.parents {
            let handle = handle_by_id.get(parent).copied().ok_or_else(|| {
                Error::UnresolvedCityObjectReference {
                    source_id: relation.source_id.to_owned(),
                    target_id: parent.to_owned(),
                    relation: "parent",
                }
            })?;
            cityobject.add_parent(handle);
        }

        for child in relation.children {
            let handle = handle_by_id.get(child).copied().ok_or_else(|| {
                Error::UnresolvedCityObjectReference {
                    source_id: relation.source_id.to_owned(),
                    target_id: child.to_owned(),
                    relation: "child",
                }
            })?;
            cityobject.add_child(handle);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use cityjson::prelude::OwnedStringStorage;
    use cityjson::v2_0::AttributeValue;

    use super::StreamingCityObject;

    #[test]
    fn streaming_cityobject_deserializes_attributes_and_extra() {
        let json = r#"{
            "type": "Building",
            "attributes": {"name": "Main", "height": 12.5},
            "custom:flag": true,
            "custom:tags": ["a", "b"]
        }"#;

        let cityobject: StreamingCityObject<'_, OwnedStringStorage> =
            serde_json::from_str(json).expect("cityobject should deserialize");

        assert_eq!(cityobject.type_name, "Building");
        assert_eq!(
            cityobject
                .attributes
                .as_ref()
                .and_then(|attributes| attributes.get("name")),
            Some(&AttributeValue::String("Main".to_owned()))
        );
        assert_eq!(
            cityobject.extra.get("custom:flag"),
            Some(&AttributeValue::Bool(true))
        );
        assert_eq!(
            cityobject.extra.get("custom:tags"),
            Some(&AttributeValue::Vec(vec![
                AttributeValue::String("a".to_owned()),
                AttributeValue::String("b".to_owned()),
            ]))
        );
    }
}
