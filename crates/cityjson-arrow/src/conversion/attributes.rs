use arrow::array::{
    Array, ArrayData, ArrayRef, BooleanArray, Float64Array, Int64Array, MapArray, NullArray,
    StringArray, StructArray, UInt64Array, UnionArray,
};
use arrow::buffer::{Buffer, ScalarBuffer};
use arrow::datatypes::{DataType, Field, Fields, Schema, UnionFields, UnionMode};
use cityjson::prelude::{AttributeValue, Attributes, ResourceRef, StringStorage};
use std::sync::Arc;

use crate::error::{Error, Result};

/// Converts [Attributes] to an Arrow MapArray.
///
/// ## Arguments
/// - `map_field_name` is either "attributes" or "extra", depending on wether the
/// attributes are used as object attributes or extra properties.
///
/// ## Returns
/// A tuple containing the schema of the Map and the MapArray.
pub fn attributes_to_arrow<SS: StringStorage, RR: ResourceRef>(
    attributes: &Attributes<SS, RR>,
    map_field_name: &str,
) -> Result<(Schema, MapArray)> {
    // Define the union fields: each field gets a unique type id.
    let union_fields = UnionFields::from_iter(vec![
        (0i8, Arc::new(Field::new("null", DataType::Null, true))),
        (1i8, Arc::new(Field::new("bool", DataType::Boolean, true))),
        (2i8, Arc::new(Field::new("uint", DataType::UInt64, true))),
        (3i8, Arc::new(Field::new("int", DataType::Int64, true))),
        (4i8, Arc::new(Field::new("float", DataType::Float64, true))),
        (5i8, Arc::new(Field::new("string", DataType::Utf8, true))),
    ]);
    let union_mode = UnionMode::Dense;

    // ----- Step 1: Accumulate keys and union components -----
    let mut keys: Vec<&str> = Vec::with_capacity(attributes.len());
    // For the union array in dense mode, we need:
    // - A vector of type IDs (one per attribute)
    // - A vector of offsets (one per attribute) indicating the position in the corresponding child array
    let mut union_type_ids: Vec<i8> = Vec::with_capacity(attributes.len());
    let mut union_offsets: Vec<i32> = Vec::with_capacity(attributes.len());
    // For each child type, we accumulate the actual values.
    let mut null_count = 0;
    let mut bool_values: Vec<bool> = Vec::new();
    let mut uint_values: Vec<u64> = Vec::new();
    let mut int_values: Vec<i64> = Vec::new();
    let mut float_values: Vec<f64> = Vec::new();
    let mut string_values: Vec<&str> = Vec::new();

    // Iterate over each attribute, record the key and update the union accumulators.
    for (key, value) in attributes.iter() {
        keys.push(key.as_ref());
        match value {
            AttributeValue::Null => {
                union_type_ids.push(0);
                union_offsets.push(null_count);
                null_count += 1;
            }
            AttributeValue::Bool(v) => {
                union_type_ids.push(1);
                union_offsets.push(bool_values.len() as i32);
                bool_values.push(*v);
            }
            AttributeValue::Unsigned(v) => {
                union_type_ids.push(2);
                union_offsets.push(uint_values.len() as i32);
                uint_values.push(*v);
            }
            AttributeValue::Integer(v) => {
                union_type_ids.push(3);
                union_offsets.push(int_values.len() as i32);
                int_values.push(*v);
            }
            AttributeValue::Float(v) => {
                union_type_ids.push(4);
                union_offsets.push(float_values.len() as i32);
                float_values.push(*v);
            }
            AttributeValue::String(v) => {
                union_type_ids.push(5);
                union_offsets.push(string_values.len() as i32);
                string_values.push(v.as_ref());
            }
            // For nested vector or map values inside a vector, we return an error.
            _ => {
                return Err(Error::Unsupported(
                    "Nested types are not supported".to_string(),
                ));
            }
        }
    }

    // ----- Step 2: Build the child arrays for the union -----
    let null_array = Arc::new(NullArray::new(null_count as usize)) as ArrayRef;
    let bool_array = Arc::new(BooleanArray::from(bool_values)) as ArrayRef;
    let uint_array = Arc::new(UInt64Array::from(uint_values)) as ArrayRef;
    let int_array = Arc::new(Int64Array::from(int_values)) as ArrayRef;
    let float_array = Arc::new(Float64Array::from(float_values)) as ArrayRef;
    let string_array = Arc::new(StringArray::from(string_values)) as ArrayRef;

    let children = vec![
        null_array,
        bool_array,
        uint_array,
        int_array,
        float_array,
        string_array,
    ];

    // Build buffers for the union array.
    let type_ids_buffer = ScalarBuffer::from(union_type_ids.clone());
    let offsets_buffer = ScalarBuffer::from(union_offsets);

    // Create the union array from the buffers and children.
    let union_array = UnionArray::try_new(
        union_fields.clone(),
        type_ids_buffer,
        Some(offsets_buffer),
        children,
    )?;

    // ----- Step 3: Build the StructArray for map entries -----
    // Build the keys array.
    let keys_array = StringArray::from(keys);
    // Create a struct array with fields "key" and "value".
    let struct_array = StructArray::try_new(
        Fields::from(vec![
            Field::new("key", DataType::Utf8, false),
            Field::new(
                "value",
                DataType::Union(union_fields.clone(), UnionMode::Dense),
                true,
            ),
        ]),
        vec![
            Arc::new(keys_array) as ArrayRef,
            Arc::new(union_array) as ArrayRef,
        ],
        None,
    )?;

    // ----- Step 4: Assemble the MapArray -----
    let value_type = DataType::Union(union_fields, union_mode);
    // Define the map entry field: a struct with key and value.
    let map_entry_field = Arc::new(Field::new(
        "entries",
        DataType::Struct(Fields::from(vec![
            Field::new("key", DataType::Utf8, false),
            Field::new("value", value_type.clone(), true),
        ])),
        false,
    ));

    // A MapArray is represented as a ListArray whose values are the map entries (a StructArray).
    // For one record (one map), the offsets buffer is [0, num_entries].
    let num_entries = struct_array.len() as i32;
    let map_offsets = vec![0, num_entries];
    let map_offsets_buffer = Buffer::from_slice_ref(&map_offsets);

    let map_data = ArrayData::builder(DataType::Map(map_entry_field.clone(), false))
        .len(1) // one row (one map)
        .add_buffer(map_offsets_buffer)
        .add_child_data(struct_array.to_data().clone())
        .build()?;
    let map_array = MapArray::from(map_data);

    // The map itself is represented as a Map type.
    let map_field = Arc::new(Field::new(
        map_field_name,
        DataType::Map(map_entry_field.clone(), false),
        true,
    ));
    let schema = Schema::new(vec![map_field]);

    Ok((schema, map_array))
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{MapArray, RecordBatch, StringArray};
    use cityjson::prelude::{AttributeValue, OwnedAttributes};
    use std::collections::HashSet;

    #[test]
    fn test_attributes_to_arrow_conversion() {
        // Create a set of test attributes.
        let mut attributes = OwnedAttributes::new();
        attributes.insert("null_value".to_string(), AttributeValue::Null);
        attributes.insert("bool_value".to_string(), AttributeValue::Bool(true));
        attributes.insert("uint_value".to_string(), AttributeValue::Unsigned(100));
        attributes.insert("int_value".to_string(), AttributeValue::Integer(-42));
        attributes.insert(
            "float_value".to_string(),
            AttributeValue::Float(std::f64::consts::E),
        );
        attributes.insert(
            "string_value".to_string(),
            AttributeValue::String("test".to_string()),
        );

        // Convert attributes to an Arrow RecordBatch.
        let (schema, map_array) = attributes_to_arrow(&attributes, "attributes")
            .expect("Failed to convert attributes to Arrow");
        let record_batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(map_array)])
            .expect("Failed to create record batch");

        // Verify that the RecordBatch contains one column and one row.
        assert_eq!(record_batch.num_columns(), 1);
        assert_eq!(record_batch.num_rows(), 1);

        // Downcast the first column to a MapArray.
        let map_array = record_batch
            .column(0)
            .as_any()
            .downcast_ref::<MapArray>()
            .expect("Expected a MapArray");

        // Retrieve the entries array from the MapArray.
        let entries = map_array.entries();

        // The keys are stored as a StringArray in the first field of the struct.
        let keys = entries
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("Expected keys to be a StringArray");

        // We expect six keys, one for each attribute inserted.
        assert_eq!(keys.len(), 6);

        // Check that the keys match the expected set.
        let actual_keys: HashSet<_> = (0..keys.len()).map(|i| keys.value(i)).collect();
        let expected_keys: HashSet<_> = [
            "null_value",
            "bool_value",
            "uint_value",
            "int_value",
            "float_value",
            "string_value",
        ]
        .iter()
        .cloned()
        .collect();
        assert_eq!(actual_keys, expected_keys);
    }
}
