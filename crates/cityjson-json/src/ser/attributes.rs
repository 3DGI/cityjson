use serde::Serialize;
use serde::ser::{Error as _, SerializeMap, SerializeSeq};

use cityjson_types::resources::storage::StringStorage;
use cityjson_types::v2_0::{AttributeValue, Attributes};
use cityjson_types::v2_0::{CityModel, VertexRef};

use crate::errors::Error;
use crate::ser::context::WriteContext;
use crate::ser::geometry::GeometrySerializer;

pub(crate) fn serialize_attributes_entries<M, VR, SS>(
    map: &mut M,
    attributes: &Attributes<SS>,
    model: &CityModel<VR, SS>,
    context: &WriteContext,
) -> std::result::Result<(), M::Error>
where
    M: SerializeMap,
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    for (key, value) in attributes.iter() {
        map.serialize_entry(
            key.as_ref(),
            &AttributeValueSerializer {
                value,
                model,
                context,
            },
        )?;
    }
    Ok(())
}

pub(crate) struct AttributesSerializer<'a, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    pub(crate) attributes: &'a Attributes<SS>,
    pub(crate) model: &'a CityModel<VR, SS>,
    pub(crate) context: &'a WriteContext,
}

impl<VR, SS> Serialize for AttributesSerializer<'_, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.attributes.len()))?;
        serialize_attributes_entries(&mut map, self.attributes, self.model, self.context)?;
        map.end()
    }
}

pub(crate) struct AttributeValueSerializer<'a, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    pub(crate) value: &'a AttributeValue<SS>,
    pub(crate) model: &'a CityModel<VR, SS>,
    pub(crate) context: &'a WriteContext,
}

impl<VR, SS> Serialize for AttributeValueSerializer<'_, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.value {
            AttributeValue::Null => serializer.serialize_unit(),
            AttributeValue::Bool(value) => serializer.serialize_bool(*value),
            AttributeValue::Unsigned(value) => serializer.serialize_u64(*value),
            AttributeValue::Integer(value) => serializer.serialize_i64(*value),
            AttributeValue::Float(value) => serializer.serialize_f64(*value),
            AttributeValue::String(value) => serializer.serialize_str(value.as_ref()),
            AttributeValue::Vec(values) => {
                let mut seq = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    seq.serialize_element(&AttributeValueSerializer {
                        value,
                        model: self.model,
                        context: self.context,
                    })?;
                }
                seq.end()
            }
            AttributeValue::Map(values) => {
                let mut map = serializer.serialize_map(Some(values.len()))?;
                for (key, value) in values {
                    map.serialize_entry(
                        key.as_ref(),
                        &AttributeValueSerializer {
                            value,
                            model: self.model,
                            context: self.context,
                        },
                    )?;
                }
                map.end()
            }
            AttributeValue::Geometry(handle) => {
                let geometry = self.model.get_geometry(*handle).ok_or_else(|| {
                    S::Error::custom(Error::InvalidValue(format!(
                        "missing geometry for handle {handle}"
                    )))
                })?;
                GeometrySerializer {
                    model: self.model,
                    geometry,
                    context: self.context,
                }
                .serialize(serializer)
            }
            _ => Err(S::Error::custom(Error::UnsupportedFeature(
                "unknown attribute variant is not implemented yet",
            ))),
        }
    }
}
