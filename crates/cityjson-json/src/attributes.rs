use serde::{Deserialize, Serialize, Serializer};
use serde_json_borrow::{OwnedValue, Value};
use std::fmt::{Display, Formatter};

/// Attributes of CityModel, CityObjects, Semantics.
/// Borrowed from the input data when deserialized. The deserialized value
/// is [Value].
/// Can own its value, which is then [serde_json::Value].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Attributes<'cm> {
    Borrowed(Value<'cm>),
    Owned(OwnedValue),
}

impl Default for Attributes<'_> {
    fn default() -> Self {
        Self::Owned(OwnedValue::from_str("").unwrap())
    }
}

impl<'cm> Attributes<'cm> {
   pub fn as_value(&self) -> &Value {
       match self {
           Attributes::Borrowed(value) => value,
           Attributes::Owned(value) => value.get_value(),
       }
   }
}

impl Display for Attributes<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Attributes::Borrowed(v) => {
                write!(f, "{:?}", v)
            }
            Attributes::Owned(v) => {
                write!(f, "{:?}", v)
            }
        }
    }
}

pub fn deserialize_attributes<'de: 'cm, 'cm, D>(
    deserializer: D,
) -> Result<Option<Attributes<'cm>>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s = Value::deserialize(deserializer)?;
    Ok((!s.is_null()).then_some(Attributes::Borrowed(s)))
}

pub fn serialize_attributes<S>(
    attributes: &Option<Attributes>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // We unwrap here, because the attributes are always set to
    // 'skip_serializing_if = "Option::is_none"'.
    let a = attributes.as_ref().unwrap();
    match a {
        Attributes::Borrowed(a) => a.serialize(serializer),
        Attributes::Owned(a) => a.serialize(serializer),
    }
}
