use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use cityjson::v2_0::{AttributeValue, Attributes};

use crate::de::parse::ParseStringStorage;
use crate::errors::{Error, Result};

/// A typed recursive enum for JSON attribute values, borrowing strings from the
/// original input where possible.
///
/// `String` holds a `Cow<'de, str>` so that:
/// - unescaped strings are borrowed from the input (`Cow::Borrowed`)
/// - escaped strings are owned (`Cow::Owned`)
///
/// Object keys use `&'de str` (unescaped JSON object keys only).
#[derive(Debug)]
pub(crate) enum RawAttribute<'de> {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(Cow<'de, str>),
    Array(Vec<RawAttribute<'de>>),
    Object(HashMap<&'de str, RawAttribute<'de>>),
}

struct RawAttributeVisitor<'de>(PhantomData<&'de ()>);

impl<'de> Visitor<'de> for RawAttributeVisitor<'de> {
    type Value = RawAttribute<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any JSON value")
    }

    fn visit_unit<E: de::Error>(self) -> std::result::Result<Self::Value, E> {
        Ok(RawAttribute::Null)
    }

    fn visit_none<E: de::Error>(self) -> std::result::Result<Self::Value, E> {
        Ok(RawAttribute::Null)
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> std::result::Result<Self::Value, E> {
        Ok(RawAttribute::Bool(v))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<Self::Value, E> {
        Ok(RawAttribute::Number(v.into()))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<Self::Value, E> {
        Ok(RawAttribute::Number(v.into()))
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> std::result::Result<Self::Value, E> {
        Ok(RawAttribute::Number(
            serde_json::Number::from_f64(v)
                .ok_or_else(|| de::Error::custom("non-finite float in attribute"))?,
        ))
    }

    fn visit_borrowed_str<E: de::Error>(self, v: &'de str) -> std::result::Result<Self::Value, E> {
        Ok(RawAttribute::String(Cow::Borrowed(v)))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<Self::Value, E> {
        Ok(RawAttribute::String(Cow::Owned(v.to_owned())))
    }

    fn visit_string<E: de::Error>(self, v: String) -> std::result::Result<Self::Value, E> {
        Ok(RawAttribute::String(Cow::Owned(v)))
    }

    fn visit_seq<A: SeqAccess<'de>>(
        self,
        mut seq: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(elem) = seq.next_element::<RawAttribute<'de>>()? {
            vec.push(elem);
        }
        Ok(RawAttribute::Array(vec))
    }

    fn visit_map<A: MapAccess<'de>>(
        self,
        mut map: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let mut hm = HashMap::with_capacity(map.size_hint().unwrap_or(0));
        while let Some(k) = map.next_key::<&'de str>()? {
            let v = map.next_value::<RawAttribute<'de>>()?;
            hm.insert(k, v);
        }
        Ok(RawAttribute::Object(hm))
    }
}

impl<'de> Deserialize<'de> for RawAttribute<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        deserializer.deserialize_any(RawAttributeVisitor(PhantomData))
    }
}

pub(crate) struct AttributeValueSeed<SS>(PhantomData<SS>);

impl<SS> AttributeValueSeed<SS> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'de, SS> DeserializeSeed<'de> for AttributeValueSeed<SS>
where
    SS: ParseStringStorage<'de>,
{
    type Value = AttributeValue<SS>;

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(AttributeValueVisitor::<SS>(PhantomData))
    }
}

struct AttributeValueVisitor<SS>(PhantomData<SS>);

impl<'de, SS> Visitor<'de> for AttributeValueVisitor<SS>
where
    SS: ParseStringStorage<'de>,
{
    type Value = AttributeValue<SS>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any JSON value")
    }

    fn visit_unit<E: de::Error>(self) -> std::result::Result<Self::Value, E> {
        Ok(AttributeValue::Null)
    }

    fn visit_none<E: de::Error>(self) -> std::result::Result<Self::Value, E> {
        Ok(AttributeValue::Null)
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> std::result::Result<Self::Value, E> {
        Ok(AttributeValue::Bool(value))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> std::result::Result<Self::Value, E> {
        Ok(AttributeValue::Integer(value))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> std::result::Result<Self::Value, E> {
        Ok(AttributeValue::Unsigned(value))
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> std::result::Result<Self::Value, E> {
        if value.is_finite() {
            Ok(AttributeValue::Float(value))
        } else {
            Err(de::Error::custom("non-finite float in attribute"))
        }
    }

    fn visit_borrowed_str<E: de::Error>(
        self,
        value: &'de str,
    ) -> std::result::Result<Self::Value, E> {
        Ok(AttributeValue::String(SS::store(value)))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> std::result::Result<Self::Value, E> {
        Ok(AttributeValue::String(
            SS::store_cow(Cow::Owned(value.to_owned())).map_err(de::Error::custom)?,
        ))
    }

    fn visit_string<E: de::Error>(self, value: String) -> std::result::Result<Self::Value, E> {
        Ok(AttributeValue::String(
            SS::store_cow(Cow::Owned(value)).map_err(de::Error::custom)?,
        ))
    }

    fn visit_seq<A: SeqAccess<'de>>(
        self,
        mut seq: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let mut values = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(value) = seq.next_element_seed(AttributeValueSeed::<SS>::new())? {
            values.push(value);
        }
        Ok(AttributeValue::Vec(values))
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> std::result::Result<Self::Value, A::Error> {
        Ok(AttributeValue::Map(read_attributes_map::<A, SS>(map)?))
    }
}

pub(crate) struct AttributesSeed<SS>(PhantomData<SS>);

impl<SS> AttributesSeed<SS> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'de, SS> DeserializeSeed<'de> for AttributesSeed<SS>
where
    SS: ParseStringStorage<'de>,
{
    type Value = Attributes<SS>;

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(AttributesVisitor::<SS>(PhantomData))
    }
}

struct AttributesVisitor<SS>(PhantomData<SS>);

impl<'de, SS> Visitor<'de> for AttributesVisitor<SS>
where
    SS: ParseStringStorage<'de>,
{
    type Value = Attributes<SS>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a JSON object")
    }

    fn visit_map<A>(self, map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let values = read_attributes_map::<A, SS>(map)?;
        Ok(Attributes::from(values))
    }
}

pub(crate) struct OptionalAttributesSeed<SS>(PhantomData<SS>);

impl<SS> OptionalAttributesSeed<SS> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'de, SS> DeserializeSeed<'de> for OptionalAttributesSeed<SS>
where
    SS: ParseStringStorage<'de>,
{
    type Value = Option<Attributes<SS>>;

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(OptionalAttributesVisitor::<SS>(PhantomData))
    }
}

struct OptionalAttributesVisitor<SS>(PhantomData<SS>);

impl<'de, SS> Visitor<'de> for OptionalAttributesVisitor<SS>
where
    SS: ParseStringStorage<'de>,
{
    type Value = Option<Attributes<SS>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("null or a JSON object")
    }

    fn visit_none<E: de::Error>(self) -> std::result::Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_unit<E: de::Error>(self) -> std::result::Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        AttributesSeed::<SS>::new()
            .deserialize(deserializer)
            .map(Some)
    }
}

fn read_attributes_map<'de, A, SS>(
    mut map: A,
) -> std::result::Result<HashMap<SS::String, AttributeValue<SS>>, A::Error>
where
    A: MapAccess<'de>,
    SS: ParseStringStorage<'de>,
{
    let mut values = HashMap::with_capacity(map.size_hint().unwrap_or(0));
    while let Some(key) = map.next_key::<&'de str>()? {
        let value = map.next_value_seed(AttributeValueSeed::<SS>::new())?;
        values.insert(SS::store(key), value);
    }
    Ok(values)
}

/// Convert a `RawAttribute<'de>` into a typed `AttributeValue<SS>`.
///
/// For borrowed mode: string values that required allocation (escaped strings in JSON)
/// will return an error.
pub(crate) fn attribute_value<'de, SS>(
    raw: RawAttribute<'de>,
    context: &'static str,
) -> Result<AttributeValue<SS>>
where
    SS: ParseStringStorage<'de>,
{
    Ok(match raw {
        RawAttribute::Null => AttributeValue::Null,
        RawAttribute::Bool(b) => AttributeValue::Bool(b),
        RawAttribute::Number(n) => {
            if let Some(v) = n.as_u64() {
                AttributeValue::Unsigned(v)
            } else if let Some(v) = n.as_i64() {
                AttributeValue::Integer(v)
            } else if let Some(v) = n.as_f64() {
                AttributeValue::Float(v)
            } else {
                return Err(Error::InvalidValue(format!(
                    "{context} contains an unsupported JSON number"
                )));
            }
        }
        RawAttribute::String(cow) => AttributeValue::String(SS::store_cow(cow)?),
        RawAttribute::Array(values) => AttributeValue::Vec(
            values
                .into_iter()
                .map(|v| attribute_value::<SS>(v, context))
                .collect::<Result<Vec<_>>>()?,
        ),
        RawAttribute::Object(map) => {
            let mut result = HashMap::with_capacity(map.len());
            for (k, v) in map {
                result.insert(SS::store(k), attribute_value::<SS>(v, context)?);
            }
            AttributeValue::Map(result)
        }
    })
}

/// Convert a `HashMap<&'de str, RawAttribute<'de>>` into a typed `Attributes<SS>`.
pub(crate) fn attribute_map<'de, SS>(
    raw: HashMap<&'de str, RawAttribute<'de>>,
    context: &'static str,
) -> Result<Attributes<SS>>
where
    SS: ParseStringStorage<'de>,
{
    let mut attrs = Attributes::<SS>::with_capacity(raw.len());
    for (k, v) in raw {
        attrs.insert(SS::store(k), attribute_value::<SS>(v, context)?);
    }
    Ok(attrs)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use cityjson::prelude::OwnedStringStorage;
    use cityjson::v2_0::AttributeValue;

    use super::AttributesSeed;
    use serde::de::DeserializeSeed;

    #[test]
    fn attributes_seed_parses_nested_values() {
        let json = r#"{
            "name": "Building A",
            "levels": [1, 2, 3],
            "meta": {"kind": "residential", "active": true}
        }"#;

        let mut deserializer = serde_json::Deserializer::from_str(json);
        let attributes = AttributesSeed::<OwnedStringStorage>::new()
            .deserialize(&mut deserializer)
            .expect("attributes should deserialize");
        deserializer
            .end()
            .expect("deserializer should be exhausted");

        assert_eq!(
            attributes.get("name"),
            Some(&AttributeValue::String("Building A".to_owned()))
        );
        assert_eq!(
            attributes.get("levels"),
            Some(&AttributeValue::Vec(vec![
                AttributeValue::Unsigned(1),
                AttributeValue::Unsigned(2),
                AttributeValue::Unsigned(3),
            ]))
        );
        assert_eq!(
            attributes.get("meta"),
            Some(&AttributeValue::Map(HashMap::from([
                (
                    "kind".to_owned(),
                    AttributeValue::String("residential".to_owned()),
                ),
                ("active".to_owned(), AttributeValue::Bool(true)),
            ])))
        );
    }
}
