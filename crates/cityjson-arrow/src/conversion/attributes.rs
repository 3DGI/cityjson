use arrow::array::{
    Array, ArrayData, ArrayRef, BooleanArray, Float64Array, Int64Array, MapArray, NullArray,
    StringArray, StructArray, UInt64Array, UnionArray,
};
use arrow::buffer::{Buffer, ScalarBuffer};
use arrow::datatypes::{DataType, Field, Fields, Schema, UnionFields, UnionMode};
use cityjson::prelude::{AttributeValue, Attributes, OwnedStringStorage, ResourceRef, StringStorage};
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

    let union_fields = union_fields();
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
                DataType::Union(union_fields, UnionMode::Dense),
                true,
            ),
        ]),
        vec![
            Arc::new(keys_array) as ArrayRef,
            Arc::new(union_array) as ArrayRef,
        ],
        None,
    )?;

    // The map itself is represented as a Map type
    let map_field = map_field(map_field_name);
    // A MapArray is represented as a ListArray whose values are the map entries (a StructArray).
    // For one record (one map), the offsets buffer is [0, num_entries].
    let num_entries = struct_array.len() as i32;
    let map_offsets = vec![0, num_entries];
    let map_offsets_buffer = Buffer::from_slice_ref(&map_offsets);

    let map_data = ArrayData::builder(map_field.data_type().clone())
        .len(1) // one row (one map)
        .add_buffer(map_offsets_buffer)
        .add_child_data(struct_array.to_data().clone())
        .build()?;
    let map_array = MapArray::from(map_data);

    let schema = Schema::new(vec![map_field]);
    Ok((schema, map_array))
}

pub(crate) fn map_field(map_field_name: &str) -> Field {
    Field::new(
        map_field_name,
        DataType::Map(Arc::new(map_entry_field(union_type())), false),
        true,
    )
}

fn map_entry_field(value_type: DataType) -> Field {
    // Define the map entry field: a struct with key and value
    Field::new(
        "entries",
        DataType::Struct(Fields::from(vec![
            Field::new("key", DataType::Utf8, false),
            Field::new("value", value_type, true),
        ])),
        false,
    )
}

pub fn union_type() -> DataType {
    // Define the union fields for attribute values
    let union_fields = union_fields();
    let union_mode = UnionMode::Dense;
    let value_type = DataType::Union(union_fields, union_mode);
    value_type
}

pub fn union_fields() -> UnionFields {
    let union_fields = UnionFields::from_iter(vec![
        (0i8, Arc::new(Field::new("null", DataType::Null, true))),
        (1i8, Arc::new(Field::new("bool", DataType::Boolean, true))),
        (2i8, Arc::new(Field::new("uint", DataType::UInt64, true))),
        (3i8, Arc::new(Field::new("int", DataType::Int64, true))),
        (4i8, Arc::new(Field::new("float", DataType::Float64, true))),
        (5i8, Arc::new(Field::new("string", DataType::Utf8, true))),
    ]);
    union_fields
}


/// Converts an Arrow MapArray to a cityjson-rs OwnedAttributes container.
///
/// This function extracts key-value pairs from an Arrow MapArray and converts them
/// to a cityjson-rs Attributes container with owned strings.
///
/// # Parameters
///
/// * `map_array` - The Arrow MapArray containing attribute data
///
/// # Returns
///
/// A Result containing the converted Attributes container or an error
pub fn arrow_to_attributes_owned<RR: ResourceRef>(
    map_array: &MapArray,
) -> Result<Attributes<OwnedStringStorage, RR>> {
    let mut attributes = Attributes::<OwnedStringStorage, RR>::new();

    // Handle empty map
    if map_array.len() == 0 {
        return Ok(attributes);
    }

    // Get the entries struct array
    let entries = map_array.entries();

    // Get the keys and values arrays
    let keys = entries.column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| Error::Conversion("Expected StringArray for keys".to_string()))?;

    let values = entries.column(1)
        .as_any()
        .downcast_ref::<UnionArray>()
        .ok_or_else(|| Error::Conversion("Expected UnionArray for values".to_string()))?;

    // Process each entry
    for i in 0..entries.len() {
        let key = keys.value(i).to_string();
        let attr_value = convert_union_value_to_owned_attribute_value(values, i)?;
        attributes.insert(key, attr_value);
    }

    Ok(attributes)
}

/// Converts a UnionArray value to an AttributeValue with owned strings.
///
/// This function handles the different types of values that can be stored in the UnionArray
/// and converts them to the appropriate AttributeValue variant.
fn convert_union_value_to_owned_attribute_value<RR: ResourceRef>(
    union_array: &UnionArray,
    index: usize,
) -> Result<AttributeValue<OwnedStringStorage, RR>> {
    let type_id = union_array.type_id(index);
    let value_offset = union_array.value_offset(index);

    match type_id {
        0 => Ok(AttributeValue::Null),
        1 => { // Boolean
            let array = union_array.child(1)
                .as_any()
                .downcast_ref::<BooleanArray>()
                .ok_or_else(|| Error::Conversion("Expected BooleanArray".to_string()))?;
            Ok(AttributeValue::Bool(array.value(value_offset)))
        },
        2 => { // Unsigned
            let array = union_array.child(2)
                .as_any()
                .downcast_ref::<UInt64Array>()
                .ok_or_else(|| Error::Conversion("Expected UInt64Array".to_string()))?;
            Ok(AttributeValue::Unsigned(array.value(value_offset)))
        },
        3 => { // Integer
            let array = union_array.child(3)
                .as_any()
                .downcast_ref::<Int64Array>()
                .ok_or_else(|| Error::Conversion("Expected Int64Array".to_string()))?;
            Ok(AttributeValue::Integer(array.value(value_offset)))
        },
        4 => { // Float
            let array = union_array.child(4)
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| Error::Conversion("Expected Float64Array".to_string()))?;
            Ok(AttributeValue::Float(array.value(value_offset)))
        },
        5 => { // String
            let array = union_array.child(5)
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| Error::Conversion("Expected StringArray".to_string()))?;
            Ok(AttributeValue::String(array.value(value_offset).to_string()))
        },
        _ => Err(Error::Unsupported(format!("Nested types are not supported (type_id: {})", type_id))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{MapArray, RecordBatch, StringArray};
    use cityjson::prelude::{AttributeValue, OwnedAttributes, ResourceId32};
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

    #[test]
    fn test_arrow_to_attributes_owned() {
        // Create test data for attributes with various primitive types
        let keys = StringArray::from(vec![
            "null_value", "bool_value", "uint_value",
            "int_value", "float_value", "string_value"
        ]);

        // Create child arrays for the union
        let null_array = Arc::new(NullArray::new(1)) as ArrayRef;
        let bool_array = Arc::new(BooleanArray::from(vec![true])) as ArrayRef;
        let uint_array = Arc::new(UInt64Array::from(vec![42u64])) as ArrayRef;
        let int_array = Arc::new(Int64Array::from(vec![-42i64])) as ArrayRef;
        let float_array = Arc::new(Float64Array::from(vec![3.14159])) as ArrayRef;
        let string_array = Arc::new(StringArray::from(vec!["test"])) as ArrayRef;

        // Define union fields
        let union_fields = UnionFields::from_iter(vec![
            (0i8, Arc::new(Field::new("null", DataType::Null, true))),
            (1i8, Arc::new(Field::new("bool", DataType::Boolean, true))),
            (2i8, Arc::new(Field::new("uint", DataType::UInt64, true))),
            (3i8, Arc::new(Field::new("int", DataType::Int64, true))),
            (4i8, Arc::new(Field::new("float", DataType::Float64, true))),
            (5i8, Arc::new(Field::new("string", DataType::Utf8, true))),
        ]);

        let type_ids_vec = vec![0i8, 1, 2, 3, 4, 5];
        let offsets_vec = vec![0, 0, 0, 0, 0, 0];
        
        // Create the union array
        let union_array = UnionArray::try_new(
            union_fields.clone(),
            ScalarBuffer::from(type_ids_vec),
            Some(ScalarBuffer::from(offsets_vec)),
            vec![null_array, bool_array, uint_array, int_array, float_array, string_array],
        ).unwrap();

        // Create the struct array for entries
        let struct_array = arrow::array::StructArray::try_new(
            Fields::from(vec![
                Field::new("key", DataType::Utf8, false),
                Field::new("value", DataType::Union(union_fields, UnionMode::Dense), true),
            ]),
            vec![
                Arc::new(keys) as ArrayRef,
                Arc::new(union_array) as ArrayRef,
            ],
            None,
        ).unwrap();

        // Create the map array
        let map_data = arrow::array::ArrayData::builder(map_field("test").data_type().clone())
            .len(1)
            .add_buffer(Buffer::from_slice_ref(&[0, 6])) // offsets: [0, 6]
            .add_child_data(struct_array.to_data().clone())
            .build()
            .unwrap();
        let map_array = MapArray::from(map_data);

        // Convert to Attributes
        let attributes = arrow_to_attributes_owned::<ResourceId32>(&map_array).unwrap();

        // Verify the results
        assert_eq!(attributes.len(), 6);
        assert!(matches!(attributes.get("null_value").unwrap(), AttributeValue::Null));
        assert!(matches!(attributes.get("bool_value").unwrap(), AttributeValue::Bool(true)));
        assert!(matches!(attributes.get("uint_value").unwrap(), AttributeValue::Unsigned(42)));
        assert!(matches!(attributes.get("int_value").unwrap(), AttributeValue::Integer(-42)));
        assert!(matches!(attributes.get("float_value").unwrap(), AttributeValue::Float(val) if *val == 3.14159));
        assert!(matches!(attributes.get("string_value").unwrap(), AttributeValue::String(val) if val == "test"));
    }
}
