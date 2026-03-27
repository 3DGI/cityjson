use serde::ser::{Error as _, SerializeMap, SerializeSeq};
use serde::Serialize;

use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{AttributeValue, Attributes};

use crate::errors::Error;

pub(crate) fn serialize_attributes_entries<M, SS>(
    map: &mut M,
    attributes: &Attributes<SS>,
) -> std::result::Result<(), M::Error>
where
    M: SerializeMap,
    SS: StringStorage,
{
    for (key, value) in attributes.iter() {
        map.serialize_entry(key.as_ref(), &AttributeValueSerializer(value))?;
    }
    Ok(())
}

pub(crate) struct AttributesSerializer<'a, SS>(pub(crate) &'a Attributes<SS>)
where
    SS: StringStorage;

impl<SS> Serialize for AttributesSerializer<'_, SS>
where
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        serialize_attributes_entries(&mut map, self.0)?;
        map.end()
    }
}

pub(crate) struct AttributeValueSerializer<'a, SS>(pub(crate) &'a AttributeValue<SS>)
where
    SS: StringStorage;

impl<SS> Serialize for AttributeValueSerializer<'_, SS>
where
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            AttributeValue::Null => serializer.serialize_unit(),
            AttributeValue::Bool(value) => serializer.serialize_bool(*value),
            AttributeValue::Unsigned(value) => serializer.serialize_u64(*value),
            AttributeValue::Integer(value) => serializer.serialize_i64(*value),
            AttributeValue::Float(value) => serializer.serialize_f64(*value),
            AttributeValue::String(value) => serializer.serialize_str(value.as_ref()),
            AttributeValue::Vec(values) => {
                let mut seq = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    seq.serialize_element(&AttributeValueSerializer(value))?;
                }
                seq.end()
            }
            AttributeValue::Map(values) => {
                let mut map = serializer.serialize_map(Some(values.len()))?;
                for (key, value) in values {
                    map.serialize_entry(key.as_ref(), &AttributeValueSerializer(value))?;
                }
                map.end()
            }
            AttributeValue::Geometry(_) => Err(S::Error::custom(Error::UnsupportedFeature(
                "geometry-valued attributes are not implemented yet",
            ))),
            _ => Err(S::Error::custom(Error::UnsupportedFeature(
                "unknown attribute variant is not implemented yet",
            ))),
        }
    }
}
