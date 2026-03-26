use std::collections::HashMap;
use std::fmt;

use serde::de::{self, DeserializeSeed, MapAccess, Visitor};
use serde::Deserialize;
use serde_json::value::RawValue;

use cityjson::resources::handles::CityObjectHandle;
use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{BBox, CityModel, CityObject, CityObjectIdentifier};

use crate::de::attributes::{attribute_map, RawAttribute};
use crate::de::geometry::{import_stream_geometry, GeometryResources, StreamingGeometry};
use crate::de::parse::ParseStringStorage;
use crate::de::profiling::timed;
use crate::de::validation::parse_cityobject_type;
use crate::errors::{Error, Result};

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
pub(crate) struct StreamingCityObject<'a> {
    #[serde(rename = "type", borrow)]
    pub(crate) type_name: &'a str,
    #[serde(rename = "geographicalExtent", default)]
    pub(crate) geographical_extent: Option<[f64; 6]>,
    #[serde(default, borrow)]
    pub(crate) attributes: Option<HashMap<&'a str, RawAttribute<'a>>>,
    #[serde(default, borrow)]
    pub(crate) parents: Vec<&'a str>,
    #[serde(default, borrow)]
    pub(crate) children: Vec<&'a str>,
    #[serde(default, borrow)]
    pub(crate) geometry: Option<Vec<StreamingGeometry<'a>>>,
    #[serde(flatten, borrow)]
    pub(crate) extra: HashMap<&'a str, RawAttribute<'a>>,
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
    raw_object: StreamingCityObject<'de>,
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
        *cityobject.attributes_mut() = timed("cityobjects.attributes", || {
            attribute_map::<SS>(attributes, "CityObject.attributes")
        })?;
    }
    if !raw_object.extra.is_empty() {
        *cityobject.extra_mut() = timed("cityobjects.extra", || {
            attribute_map::<SS>(raw_object.extra, "CityObject extra")
        })?;
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
