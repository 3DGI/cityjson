use std::borrow::Cow;
use std::collections::HashMap;

use cityjson::v2_0::{
    BorrowedAttributeValue, BorrowedAttributes, OwnedAttributeValue, OwnedAttributes,
};
use serde_json::Value as OwnedJsonValue;
use serde_json_borrow::Value as BorrowedJsonValue;

use crate::errors::{Error, Result};

pub(crate) fn owned_attributes_from_json(
    value: &OwnedJsonValue,
    context: &'static str,
) -> Result<OwnedAttributes> {
    let object = value.as_object().ok_or_else(|| {
        Error::InvalidValue(format!("{context} must be a JSON object, got {value}"))
    })?;

    let mut attributes = OwnedAttributes::new();
    for (key, value) in object {
        attributes.insert(
            key.clone(),
            owned_attribute_value_from_json(value, context)?,
        );
    }

    Ok(attributes)
}

pub(crate) fn borrowed_attributes_from_json_owned<'a>(
    value: BorrowedJsonValue<'a>,
    context: &'static str,
) -> Result<BorrowedAttributes<'a>> {
    match value {
        BorrowedJsonValue::Object(values) => {
            let mut attributes = BorrowedAttributes::new();
            for (key, value) in values.into_vec() {
                attributes.insert(
                    cow_to_borrowed_str(key),
                    borrowed_attribute_value_from_json_owned(value, context)?,
                );
            }
            Ok(attributes)
        }
        other => Err(Error::InvalidValue(format!(
            "{context} must be a JSON object, got {other}"
        ))),
    }
}

pub(crate) fn borrowed_attributes_from_map<'a, I>(
    values: I,
    context: &'static str,
) -> Result<BorrowedAttributes<'a>>
where
    I: IntoIterator<Item = (&'a str, BorrowedJsonValue<'a>)>,
{
    let mut attributes = BorrowedAttributes::new();
    for (key, value) in values {
        attributes.insert(
            key,
            borrowed_attribute_value_from_json_owned(value, context)?,
        );
    }
    Ok(attributes)
}

fn owned_attribute_value_from_json(
    value: &OwnedJsonValue,
    context: &'static str,
) -> Result<OwnedAttributeValue> {
    if is_geometry_attribute_object_owned(value) {
        return Err(Error::UnsupportedFeature(
            "geometry-valued attributes are not implemented yet",
        ));
    }

    Ok(match value {
        OwnedJsonValue::Null => OwnedAttributeValue::Null,
        OwnedJsonValue::Bool(value) => OwnedAttributeValue::Bool(*value),
        OwnedJsonValue::Number(value) => {
            if let Some(value) = value.as_u64() {
                OwnedAttributeValue::Unsigned(value)
            } else if let Some(value) = value.as_i64() {
                OwnedAttributeValue::Integer(value)
            } else if let Some(value) = value.as_f64() {
                OwnedAttributeValue::Float(value)
            } else {
                return Err(Error::InvalidValue(format!(
                    "{context} contains an unsupported JSON number"
                )));
            }
        }
        OwnedJsonValue::String(value) => OwnedAttributeValue::String(value.clone()),
        OwnedJsonValue::Array(values) => OwnedAttributeValue::Vec(
            values
                .iter()
                .map(|value| owned_attribute_value_from_json(value, context).map(Box::new))
                .collect::<Result<Vec<_>>>()?,
        ),
        OwnedJsonValue::Object(values) => {
            let mut map = HashMap::with_capacity(values.len());
            for (key, value) in values {
                map.insert(
                    key.clone(),
                    Box::new(owned_attribute_value_from_json(value, context)?),
                );
            }
            OwnedAttributeValue::Map(map)
        }
    })
}

fn borrowed_attribute_value_from_json_owned<'a>(
    value: BorrowedJsonValue<'a>,
    context: &'static str,
) -> Result<BorrowedAttributeValue<'a>> {
    if is_geometry_attribute_object_borrowed(&value) {
        return Err(Error::UnsupportedFeature(
            "geometry-valued attributes are not implemented yet",
        ));
    }

    Ok(match value {
        BorrowedJsonValue::Null => BorrowedAttributeValue::Null,
        BorrowedJsonValue::Bool(value) => BorrowedAttributeValue::Bool(value),
        BorrowedJsonValue::Number(value) => {
            if let Some(value) = value.as_u64() {
                BorrowedAttributeValue::Unsigned(value)
            } else if let Some(value) = value.as_i64() {
                BorrowedAttributeValue::Integer(value)
            } else if let Some(value) = value.as_f64() {
                BorrowedAttributeValue::Float(value)
            } else {
                return Err(Error::InvalidValue(format!(
                    "{context} contains an unsupported JSON number"
                )));
            }
        }
        BorrowedJsonValue::Str(value) => BorrowedAttributeValue::String(cow_to_borrowed_str(value)),
        BorrowedJsonValue::Array(values) => BorrowedAttributeValue::Vec(
            values
                .into_iter()
                .map(|value| borrowed_attribute_value_from_json_owned(value, context).map(Box::new))
                .collect::<Result<Vec<_>>>()?,
        ),
        BorrowedJsonValue::Object(values) => {
            let mut map = HashMap::with_capacity(values.as_vec().len());
            for (key, value) in values.into_vec() {
                map.insert(
                    cow_to_borrowed_str(key),
                    Box::new(borrowed_attribute_value_from_json_owned(value, context)?),
                );
            }
            BorrowedAttributeValue::Map(map)
        }
    })
}

fn cow_to_borrowed_str<'a>(value: Cow<'a, str>) -> &'a str {
    match value {
        Cow::Borrowed(value) => value,
        Cow::Owned(value) => Box::leak(value.into_boxed_str()),
    }
}

fn is_geometry_attribute_object_owned(value: &OwnedJsonValue) -> bool {
    value
        .as_object()
        .is_some_and(|object| object.contains_key("type") && object.contains_key("boundaries"))
}

fn is_geometry_attribute_object_borrowed(value: &BorrowedJsonValue<'_>) -> bool {
    value
        .as_object()
        .is_some_and(|object| object.get("type").is_some() && object.get("boundaries").is_some())
}
