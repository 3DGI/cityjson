use arrow::array::{
    Array, ArrayRef, BooleanBuilder, Float64Builder, Int64Builder, MapBuilder,
    StringBuilder, UInt64Builder, UnionBuilder, make_union_builder
};
use arrow::datatypes::{BooleanType, DataType, Field, FieldRef, Fields, Float64Type, Int64Type, Schema, UInt64Type, UnionFields, UnionMode, Utf8Type};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatch;
use cityjson::prelude::{AttributeValue, Attributes, ResourceRef, StringStorage};
use std::collections::HashMap;
use std::sync::Arc;

/// Converts `Attributes` to an Arrow RecordBatch containing a MapArray.
///
/// This implementation uses the UnionBuilder from the Arrow crate to simplify
/// handling different value types.
pub fn attributes_to_arrow<SS: StringStorage, RR: ResourceRef>(
    attrs: &Attributes<SS, RR>
) -> Result<RecordBatch, ArrowError> {
    // Define field for the union array
    let union_fields: Vec<(usize, FieldRef)> = vec![
        (0, Arc::new(Field::new("null", DataType::Null, true))),
        (1, Arc::new(Field::new("bool", DataType::Boolean, true))),
        (2, Arc::new(Field::new("uint", DataType::UInt64, true))),
        (3, Arc::new(Field::new("int", DataType::Int64, true))),
        (4, Arc::new(Field::new("float", DataType::Float64, true))),
        (5, Arc::new(Field::new("string", DataType::Utf8, true))),
    ];

    // Create the union data type
    let value_type = DataType::Union(
        UnionFields::from_iter(union_fields.iter()),
        UnionMode::Dense
    );

    // Define the map entry field
    let map_entry_field = Arc::new(Field::new(
        "entries",
        DataType::Struct(Fields::from(vec![
            Field::new("key", DataType::Utf8, false),
            Field::new("value", value_type.clone(), true),
        ])),
        false
    ));

    // Create the map field
    let map_field = Arc::new(Field::new(
        "attributes",
        DataType::Map(map_entry_field.clone(), false),
        false
    ));

    // Create schema
    let schema = Schema::new(vec![map_field.clone()]);

    // Create map builder
    let key_builder = StringBuilder::new();
    let value_builder = UnionBuilder::with_capacity_dense(attrs.len());
    let mut map_builder = MapBuilder::with_capacity(None, key_builder, value_builder, attrs.len());

    // Add attributes to the map builder
    for (key, value) in attrs.iter() {
        map_builder.keys().append_value(key.as_ref());
        append_attribute_value(map_builder.values(), value);
    }

    // Finalize the array
    let map_array = map_builder.finish();

    // Create record batch
    let record_batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![Arc::new(map_array)]
    )?;

    Ok(record_batch)
}

/// Appends an AttributeValue to a UnionBuilder.
fn append_attribute_value<SS: StringStorage, RR: ResourceRef>(
    builder: &mut UnionBuilder,
    value: &AttributeValue<SS, RR>
) -> Result<(), ArrowError> {
    match value {
        AttributeValue::Null => {
            Ok(builder.append_null("null")?)
        },
        AttributeValue::Bool(v) => {
            Ok(builder.append::<BooleanType>("bool", v)?)
        },
        AttributeValue::Unsigned(v) => {
            Ok(builder.append::<UInt64Type>("uint", *v)?)
        },
        AttributeValue::Integer(v) => {
            Ok(builder.append::<Int64Type>("int", *v)?)
        },
        AttributeValue::Float(v) => {
            Ok(builder.append::<Float64Type>("float", *v)?)
        },
        AttributeValue::String(v) => {
            Ok(builder.append::<Utf8Type>("string", v)?)
        },
        AttributeValue::Vec(vec_values) => {
            // For vectors, serialize to a string representation
            let mut item_strs = Vec::with_capacity(vec_values.len());
            for item in vec_values.iter() {
                match item.as_ref() {
                    AttributeValue::Null => item_strs.push("null".to_string()),
                    AttributeValue::Bool(b) => item_strs.push(b.to_string()),
                    AttributeValue::Unsigned(u) => item_strs.push(u.to_string()),
                    AttributeValue::Integer(i) => item_strs.push(i.to_string()),
                    AttributeValue::Float(f) => item_strs.push(f.to_string()),
                    AttributeValue::String(s) => item_strs.push(format!("\"{}\"", s)),
                    _ => item_strs.push("complex_item".to_string()),
                }
            }
            let vec_str = format!("[{}]", item_strs.join(", "));
            builder.append::<StringBuilder>(5, |b| b.append_value(&vec_str));
        },
        AttributeValue::Map(map_values) => {
            // For maps, serialize to a string representation
            let mut entry_strs = Vec::with_capacity(map_values.len());
            for (k, v) in map_values.iter() {
                let value_str = match v.as_ref() {
                    AttributeValue::Null => "null".to_string(),
                    AttributeValue::Bool(b) => b.to_string(),
                    AttributeValue::Unsigned(u) => u.to_string(),
                    AttributeValue::Integer(i) => i.to_string(),
                    AttributeValue::Float(f) => f.to_string(),
                    AttributeValue::String(s) => format!("\"{}\"", s),
                    _ => "complex_value".to_string(),
                };
                entry_strs.push(format!("\"{}\": {}", k, value_str));
            }
            let map_str = format!("{{{}}}", entry_strs.join(", "));
            builder.append::<StringBuilder>(5, |b| b.append_value(&map_str));
        },
        AttributeValue::Geometry(geom_id) => {
            // For geometry, convert resource ID to string
            let geom_str = format!("Geometry {}", geom_id);
            builder.append::<StringBuilder>(5, |b| b.append_value(&geom_str));
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cityjson::prelude::{OwnedAttributes, ResourceId32};

    #[test]
    fn test_attributes_to_arrow() {
        // Create test attributes
        let mut attrs = OwnedAttributes::new();
        attrs.insert("null_value".to_string(), AttributeValue::Null);
        attrs.insert("bool_value".to_string(), AttributeValue::Bool(true));
        attrs.insert("uint_value".to_string(), AttributeValue::Unsigned(42));
        attrs.insert("int_value".to_string(), AttributeValue::Integer(-100));
        attrs.insert("float_value".to_string(), AttributeValue::Float(3.14159));
        attrs.insert("string_value".to_string(), AttributeValue::String("hello".to_string()));

        // Create a vector attribute (nested)
        let vec_items = vec![
            Box::new(AttributeValue::Integer(1)),
            Box::new(AttributeValue::Integer(2)),
            Box::new(AttributeValue::Integer(3)),
        ];
        attrs.insert("vec_value".to_string(), AttributeValue::Vec(vec_items));

        // Create a map attribute (nested)
        let mut map_items = HashMap::new();
        map_items.insert("key1".to_string(), Box::new(AttributeValue::String("value1".to_string())));
        map_items.insert("key2".to_string(), Box::new(AttributeValue::Integer(42)));
        attrs.insert("map_value".to_string(), AttributeValue::Map(map_items));

        // Convert to Arrow
        let result = attributes_to_arrow(&attrs);
        assert!(result.is_ok(), "Conversion to Arrow failed: {:?}", result.err());

        let batch = result.unwrap();

        // Verify structure
        assert_eq!(batch.num_columns(), 1);
        assert_eq!(batch.num_rows(), 1);

        // Verify the array is a map array
        let map_array = batch.column(0)
            .as_any()
            .downcast_ref::<arrow::array::MapArray>()
            .expect("Expected MapArray");

        // Verify the keys
        let entries = map_array.entries_array();
        let keys = entries.column(0);
        let string_keys = keys.as_any().downcast_ref::<arrow::array::StringArray>().unwrap();

        // Check key count (should have 8 keys)
        assert_eq!(string_keys.len(), 8);

        // Verify values are stored in a union array
        let values = entries.column(1);
        assert!(values.as_any().downcast_ref::<arrow::array::UnionArray>().is_some());
    }

    #[test]
    fn test_empty_attributes() {
        // Test with empty attributes
        let attrs = OwnedAttributes::new();

        let result = attributes_to_arrow(&attrs);
        assert!(result.is_ok());

        let batch = result.unwrap();
        assert_eq!(batch.num_rows(), 1);

        let map_array = batch.column(0)
            .as_any()
            .downcast_ref::<arrow::array::MapArray>()
            .expect("Expected MapArray");

        assert_eq!(map_array.value_length(0), 0);
    }
}