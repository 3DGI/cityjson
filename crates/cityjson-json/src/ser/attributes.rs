use serde_json::{Map, Number, Value};

use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{AttributeValue, Attributes};

use crate::errors::{Error, Result};

pub(crate) fn attributes_to_json_map<SS>(attributes: &Attributes<SS>) -> Result<Map<String, Value>>
where
    SS: StringStorage,
{
    let mut map = Map::with_capacity(attributes.len());
    for (key, value) in attributes.iter() {
        map.insert(key.as_ref().to_owned(), attribute_value_to_json(value)?);
    }
    Ok(map)
}

fn attribute_value_to_json<SS>(value: &AttributeValue<SS>) -> Result<Value>
where
    SS: StringStorage,
{
    Ok(match value {
        AttributeValue::Null => Value::Null,
        AttributeValue::Bool(value) => Value::Bool(*value),
        AttributeValue::Unsigned(value) => Value::Number(Number::from(*value)),
        AttributeValue::Integer(value) => Value::Number(Number::from(*value)),
        AttributeValue::Float(value) => Value::Number(
            Number::from_f64(*value)
                .ok_or_else(|| Error::InvalidValue(format!("cannot serialize float '{value}'")))?,
        ),
        AttributeValue::String(value) => Value::String(value.as_ref().to_owned()),
        AttributeValue::Vec(values) => Value::Array(
            values
                .iter()
                .map(|value| attribute_value_to_json(value))
                .collect::<Result<Vec<_>>>()?,
        ),
        AttributeValue::Map(values) => {
            let mut map = Map::with_capacity(values.len());
            for (key, value) in values {
                map.insert(key.as_ref().to_owned(), attribute_value_to_json(value)?);
            }
            Value::Object(map)
        }
        AttributeValue::Geometry(_) => {
            return Err(Error::UnsupportedFeature(
                "geometry-valued attributes are not implemented yet",
            ));
        }
        _ => {
            return Err(Error::UnsupportedFeature(
                "unknown attribute variant is not implemented yet",
            ));
        }
    })
}
