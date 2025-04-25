use crate::conversion::attributes::{attributes_to_arrow, map_field};
use crate::error::{Error, Result};
use arrow::array::{
    Array, ArrayRef, BooleanArray, DictionaryArray, Float64Array, Int64Array, ListArray,
    ListBuilder, MapArray, RecordBatch, StringArray, StringBuilder, StringDictionaryBuilder,
    UInt32Array, UInt32Builder, UInt64Array, UnionArray,
};
use arrow::datatypes::{DataType, Field, Int8Type, Schema};
use cityjson::prelude::{
    AttributeValue, Attributes, DefaultResourcePool, OwnedStringStorage, ResourceId32,
    ResourcePool, SemanticTrait, StringStorage,
};
use cityjson::v2_0::{Semantic, SemanticType};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

/// Converts a pool of cityjson-rs Semantics (v2.0) into an Arrow RecordBatch.
pub fn semantics_to_arrow<SS>(
    semantic_pool: &DefaultResourcePool<Semantic<ResourceId32, SS>, ResourceId32>,
) -> Result<RecordBatch>
where
    SS: StringStorage + Default,
    SS::String: AsRef<str> + Eq + Hash, // Constraints from Attributes map keys
{
    let schema = semantics_schema();
    let num_rows = semantic_pool.len();

    // Special case for empty pools
    if num_rows == 0 {
        return Ok(RecordBatch::new_empty(Arc::new(schema)));
    }

    // --- Initialize Builders ---
    // We need capacity hints based on expected data size.
    let mut id_builder = UInt32Builder::with_capacity(num_rows);
    let mut type_builder = StringDictionaryBuilder::<Int8Type>::new(); // TODO: estimate capacity
    let mut extension_builder = StringBuilder::new();
    let mut children_builder = ListBuilder::with_capacity(UInt32Builder::new(), num_rows);
    let mut parent_builder = UInt32Builder::with_capacity(num_rows);

    // For attributes, collect arrays to combine later
    let mut attribute_arrays = Vec::with_capacity(num_rows);

    // --- Iterate and Append Data ---
    for (resource_ref, semantic) in semantic_pool.iter() {
        // ResourceId in pool
        id_builder.append_value(resource_ref.index());

        // Process semantic type with extension
        match semantic.type_semantic() {
            SemanticType::Extension(ext_value) => {
                type_builder.append_value("Extension");
                extension_builder.append_value(ext_value);
            }
            other_type => {
                type_builder.append_value(&other_type.to_string());
                extension_builder.append_null();
            }
        }

        // Children
        if let Some(children_vec) = semantic.children() {
            let indices_builder = children_builder.values();
            // NOTE: Wanted to use `extend` here but that builds an Nullable array
            for child in children_vec {
                indices_builder.append_value(child.index());
            }
            children_builder.append(true);
        } else {
            children_builder.append(false); // Append null list
        }

        // Parent
        parent_builder.append_option(semantic.parent().map(|rr| rr.index()));

        // Attributes
        if let Some(attributes) = semantic.attributes() {
            // Convert these attributes to a MapArray
            let (_, map_array) = attributes_to_arrow(attributes, "attributes")?;
            attribute_arrays.push(Arc::new(map_array) as ArrayRef);
        } else {
            // Create an empty MapArray with the correct structure
            let empty_attributes = Attributes::<OwnedStringStorage, ResourceId32>::new();
            let (_, map_array) = attributes_to_arrow(&empty_attributes, "attributes")?;
            attribute_arrays.push(Arc::new(map_array) as ArrayRef);
        }
    }

    // Concatenate all attribute arrays
    let combined_attributes = arrow::compute::concat(
        &attribute_arrays
            .iter()
            .map(|a| a.as_ref())
            .collect::<Vec<_>>(),
    )?;

    // Create basic arrays
    let arrays: Vec<ArrayRef> = vec![
        Arc::new(id_builder.finish()),
        Arc::new(type_builder.finish()),
        Arc::new(extension_builder.finish()),
        Arc::new(children_builder.finish()),
        Arc::new(parent_builder.finish()),
        combined_attributes,
    ];

    RecordBatch::try_new(Arc::new(schema), arrays).map_err(Error::from)
}

/// Creates the Arrow Schema for a RecordBatch representing the Semantics pool.
/// Assumes RR=ResourceId32 (index stored as u32).
pub fn semantics_schema() -> Schema {
    Schema::new(vec![
        // Resource ID (Required)
        Field::new("id", DataType::UInt32, false),
        // Semantic Type (Required)
        Field::new(
            "type_semantic",
            DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)),
            false,
        ),
        Field::new("extension_value", DataType::Utf8, true),
        // Children Indices (Optional list)
        // TODO: the list items should be not-nullable, but it seems tha the PrimitiveBuilder only builds nullable arrays and I'm lazy now to manually set up the builder as for example in metadata.geographical_extent, but with correct offsets
        Field::new_list(
            "children",
            Field::new_list_field(DataType::UInt32, true),
            true,
        ),
        // Parent Index (Optional)
        Field::new(
            "parent",
            DataType::UInt32, // u32 index
            true,             // Optional parent
        ),
        map_field("attributes"), // Attributes (Optional)
    ])
}

/// Converts an Arrow RecordBatch to a pool of cityjson-rs Semantic instances.
///
/// This function handles the conversion of Arrow data back into cityjson-rs semantics,
/// including proper handling of optional fields like children, parent and attributes.
///
/// # Arguments
///
/// * `batch` - The Arrow RecordBatch containing the semantics data
///
/// # Returns
///
/// A `Result` containing the populated semantic pool or an error
pub fn arrow_to_semantics<SS: StringStorage + Default>(
    batch: &RecordBatch,
) -> Result<DefaultResourcePool<Semantic<ResourceId32, SS>, ResourceId32>>
where
    SS::String: AsRef<str> + From<String> + Eq + Hash,
{
    // Create a new empty semantic pool
    let mut semantic_pool = DefaultResourcePool::<Semantic<ResourceId32, SS>, ResourceId32>::new();

    // If the batch is empty, return the empty pool
    if batch.num_rows() == 0 {
        return Ok(semantic_pool);
    }

    // Extract the required columns
    let id_array = batch
        .column_by_name("id")
        .ok_or_else(|| Error::MissingField("id".to_string()))?
        .as_any()
        .downcast_ref::<UInt32Array>()
        .ok_or_else(|| Error::Conversion("Failed to downcast id array".to_string()))?;

    let type_array = batch
        .column_by_name("type_semantic")
        .ok_or_else(|| Error::MissingField("type_semantic".to_string()))?
        .as_any()
        .downcast_ref::<DictionaryArray<Int8Type>>()
        .ok_or_else(|| Error::Conversion("Failed to downcast type_semantic array".to_string()))?;

    let extension_array = batch
        .column_by_name("extension_value")
        .ok_or_else(|| Error::MissingField("extension_value".to_string()))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| Error::Conversion("Failed to downcast extension_value array".to_string()))?;

    // Get optional columns (don't fail if they're missing)
    let children_array = batch
        .column_by_name("children")
        .and_then(|col| col.as_any().downcast_ref::<ListArray>());

    let parent_array = batch
        .column_by_name("parent")
        .and_then(|col| col.as_any().downcast_ref::<UInt32Array>());

    let attributes_map_array = batch
        .column_by_name("attributes")
        .and_then(|col| col.as_any().downcast_ref::<MapArray>());

    // First pass: Create all semantics without relationships
    let mut original_ids = Vec::with_capacity(batch.num_rows());
    let mut new_ids = Vec::with_capacity(batch.num_rows());

    for i in 0..batch.num_rows() {
        // Extract the semantic type
        let type_id = type_array.keys().value(i);
        let type_values = type_array.values();
        let type_dict = type_values
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("Expected StringArray for type dictionary");
        let type_value = type_dict.value(type_id as usize);

        // Parse the semantic type
        let semantic_type = if type_value == "Extension" {
            // For extension types, we need to get the extension value
            if extension_array.is_null(i) {
                return Err(Error::Conversion(
                    "Extension type without extension_value".to_string(),
                ));
            }
            let extension_value = extension_array.value(i);
            SemanticType::Extension(SS::String::from(extension_value.to_string()))
        } else {
            match type_value {
                "Default" => SemanticType::Default,
                "RoofSurface" => SemanticType::RoofSurface,
                "GroundSurface" => SemanticType::GroundSurface,
                "WallSurface" => SemanticType::WallSurface,
                "ClosureSurface" => SemanticType::ClosureSurface,
                "OuterCeilingSurface" => SemanticType::OuterCeilingSurface,
                "OuterFloorSurface" => SemanticType::OuterFloorSurface,
                "Window" => SemanticType::Window,
                "Door" => SemanticType::Door,
                "InteriorWallSurface" => SemanticType::InteriorWallSurface,
                "CeilingSurface" => SemanticType::CeilingSurface,
                "FloorSurface" => SemanticType::FloorSurface,
                "WaterSurface" => SemanticType::WaterSurface,
                "WaterGroundSurface" => SemanticType::WaterGroundSurface,
                "WaterClosureSurface" => SemanticType::WaterClosureSurface,
                "TrafficArea" => SemanticType::TrafficArea,
                "AuxiliaryTrafficArea" => SemanticType::AuxiliaryTrafficArea,
                "TransportationMarking" => SemanticType::TransportationMarking,
                "TransportationHole" => SemanticType::TransportationHole,
                _ => {
                    return Err(Error::Conversion(format!(
                        "Unknown semantic type: {}",
                        type_value
                    )));
                }
            }
        };

        // Create a new semantic instance
        let mut semantic = Semantic::new(semantic_type);

        // Set attributes if present and if attributes_map_array exists
        if let Some(attributes_array) = attributes_map_array {
            if !attributes_array.is_null(i) {
                // Create a new attributes object
                let mut attributes = Attributes::<SS, ResourceId32>::new();

                // Extract the entries for this row
                let entries = attributes_array.value(i);
                let keys = entries
                    .column_by_name("key")
                    .unwrap()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .unwrap();

                let values = entries
                    .column_by_name("value")
                    .unwrap()
                    .as_any()
                    .downcast_ref::<UnionArray>()
                    .unwrap();

                // Process each entry
                for j in 0..entries.len() {
                    let key = SS::String::from(keys.value(j).to_string());

                    let attr_value = match values.type_id(j) {
                        0 => AttributeValue::Null,
                        1 => {
                            let array = values
                                .child(1)
                                .as_any()
                                .downcast_ref::<BooleanArray>()
                                .ok_or_else(|| {
                                    Error::Conversion("Expected BooleanArray".to_string())
                                })?;
                            AttributeValue::Bool(array.value(values.value_offset(j)))
                        }
                        2 => {
                            let array = values
                                .child(2)
                                .as_any()
                                .downcast_ref::<UInt64Array>()
                                .ok_or_else(|| {
                                    Error::Conversion("Expected UInt64Array".to_string())
                                })?;
                            AttributeValue::Unsigned(array.value(values.value_offset(j)))
                        }
                        3 => {
                            let array = values
                                .child(3)
                                .as_any()
                                .downcast_ref::<Int64Array>()
                                .ok_or_else(|| {
                                    Error::Conversion("Expected Int64Array".to_string())
                                })?;
                            AttributeValue::Integer(array.value(values.value_offset(j)))
                        }
                        4 => {
                            let array = values
                                .child(4)
                                .as_any()
                                .downcast_ref::<Float64Array>()
                                .ok_or_else(|| {
                                    Error::Conversion("Expected Float64Array".to_string())
                                })?;
                            AttributeValue::Float(array.value(values.value_offset(j)))
                        }
                        5 => {
                            let array = values
                                .child(5)
                                .as_any()
                                .downcast_ref::<StringArray>()
                                .ok_or_else(|| {
                                    Error::Conversion("Expected StringArray".to_string())
                                })?;
                            AttributeValue::String(SS::String::from(
                                array.value(values.value_offset(j)).to_string(),
                            ))
                        }
                        type_id => {
                            return Err(Error::Unsupported(format!(
                                "Unsupported attribute type: {}",
                                type_id
                            )));
                        }
                    };

                    attributes.insert(key, attr_value);
                }

                if !attributes.is_empty() {
                    *semantic.attributes_mut() = attributes;
                }
            }
        }

        // Add the semantic to the pool and track IDs
        let original_id = id_array.value(i);
        let new_id = semantic_pool.add(semantic);

        original_ids.push(original_id);
        new_ids.push(new_id);
    }

    // Create ID mapping (original ID → new ResourceId32)
    let id_mapping: HashMap<u32, ResourceId32> = original_ids
        .into_iter()
        .zip(new_ids.iter().cloned())
        .collect();

    // Second pass: Set relationships (only if relationship columns are present)
    for i in 0..batch.num_rows() {
        let new_id = new_ids[i];

        // Set parent if parent array is present and the value is not null
        if let Some(parent_arr) = parent_array {
            if !parent_arr.is_null(i) {
                let parent_original_id = parent_arr.value(i);
                if let Some(parent_new_id) = id_mapping.get(&parent_original_id) {
                    if let Some(semantic) = semantic_pool.get_mut(new_id) {
                        semantic.set_parent(*parent_new_id);
                    }
                }
            }
        }

        // Set children if children array is present and the value is not null
        if let Some(children_arr) = children_array {
            if !children_arr.is_null(i) {
                let children_list = children_arr.value(i);
                let children_values = children_list
                    .as_any()
                    .downcast_ref::<UInt32Array>()
                    .ok_or_else(|| {
                        Error::Conversion("Failed to downcast children values".to_string())
                    })?;

                if children_values.len() > 0 {
                    if let Some(semantic) = semantic_pool.get_mut(new_id) {
                        let children_vec = semantic.children_mut();
                        for j in 0..children_values.len() {
                            let child_original_id = children_values.value(j);
                            if let Some(child_new_id) = id_mapping.get(&child_original_id) {
                                children_vec.push(*child_new_id);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(semantic_pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cityjson::prelude::{
        AttributeValue, DefaultResourcePool, OwnedStringStorage, ResourceId32,
    };
    use cityjson::v2_0::geometry::semantic::{Semantic, SemanticType};

    #[test]
    fn test_semantics_schema() {
        let schema = semantics_schema();
        assert_eq!(schema.fields().len(), 6); // id, type, extension, parent, children, attributes

        // Verify field types
        assert_eq!(
            schema.field_with_name("id").unwrap().data_type(),
            &DataType::UInt32
        );
        assert!(matches!(
            schema.field_with_name("type_semantic").unwrap().data_type(),
            DataType::Dictionary(_, _)
        ));
        assert_eq!(
            schema
                .field_with_name("extension_value")
                .unwrap()
                .data_type(),
            &DataType::Utf8
        );
        assert_eq!(
            schema.field_with_name("parent").unwrap().data_type(),
            &DataType::UInt32
        );
        assert!(matches!(
            schema.field_with_name("children").unwrap().data_type(),
            DataType::List(_)
        ));
        assert!(matches!(
            schema.field_with_name("attributes").unwrap().data_type(),
            DataType::Map(_, _)
        ));
    }

    #[test]
    fn test_empty_semantics_to_arrow() {
        let semantics =
            DefaultResourcePool::<Semantic<ResourceId32, OwnedStringStorage>, ResourceId32>::new();
        let batch = semantics_to_arrow(&semantics).unwrap();

        assert_eq!(batch.num_rows(), 0);
        assert_eq!(batch.num_columns(), 6);
    }

    #[test]
    fn test_semantics_with_data() {
        let mut semantics =
            DefaultResourcePool::<Semantic<ResourceId32, OwnedStringStorage>, ResourceId32>::new();

        // Add a roof semantic
        let mut roof = Semantic::new(SemanticType::RoofSurface);
        roof.attributes_mut().insert(
            "material".to_string(),
            AttributeValue::String("shingles".to_string()),
        );
        let roof_id = semantics.add(roof);

        // Add a wall semantic with parent reference
        let mut wall = Semantic::new(SemanticType::WallSurface);
        wall.attributes_mut()
            .insert("height".to_string(), AttributeValue::Float(3.5));
        wall.set_parent(roof_id);
        let wall_id = semantics.add(wall);

        // Add a custom extension semantic with children
        let mut custom = Semantic::new(SemanticType::Extension("CustomType".to_string()));
        custom.children_mut().push(wall_id);
        semantics.add(custom);

        // Convert to Arrow
        let batch = semantics_to_arrow(&semantics).unwrap();

        // Verify the batch
        assert_eq!(batch.num_rows(), 3);
        assert_eq!(batch.num_columns(), 6);
    }

    #[test]
    fn test_arrow_to_semantics() {
        // Create test data arrays
        let id_array = UInt32Array::from(vec![0, 1, 2]);

        // Create string dictionary for semantic types
        let mut type_builder = StringDictionaryBuilder::<Int8Type>::new();
        type_builder.append_value("RoofSurface");
        type_builder.append_value("WallSurface");
        type_builder.append_value("Extension");
        let type_array = type_builder.finish();

        // Extension values
        let extension_array = StringArray::from(vec![None, None, Some("+CustomType")]);

        // Children relationships: semantic 0 has children [1, 2]
        let mut children_builder = ListBuilder::new(UInt32Builder::new());
        {
            let values_builder = children_builder.values();
            values_builder.append_value(1);
            values_builder.append_value(2);
            children_builder.append(true);
        }
        // No children for semantics 1 and 2
        children_builder.append(false);
        children_builder.append(false);
        let children_array = children_builder.finish();

        // Parent relationships: semantics 1 and 2 have parent 0
        let parent_array = UInt32Array::from(vec![None, Some(0), Some(0)]);

        // Create RecordBatch WITHOUT the attributes field
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("id", DataType::UInt32, false),
                Field::new(
                    "type_semantic",
                    DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new("extension_value", DataType::Utf8, true),
                Field::new(
                    "children",
                    DataType::List(Arc::new(Field::new("item", DataType::UInt32, true))),
                    true,
                ),
                Field::new("parent", DataType::UInt32, true),
            ])),
            vec![
                Arc::new(id_array),
                Arc::new(type_array),
                Arc::new(extension_array),
                Arc::new(children_array),
                Arc::new(parent_array),
            ],
        )
        .unwrap();

        // Convert to semantic pool
        let semantic_pool = arrow_to_semantics::<OwnedStringStorage>(&batch).unwrap();

        // Verify results
        assert_eq!(semantic_pool.len(), 3);

        // Get semantics by ID
        let semantic0 = semantic_pool.get(ResourceId32::new(0, 0)).unwrap();
        let semantic1 = semantic_pool.get(ResourceId32::new(1, 0)).unwrap();
        let semantic2 = semantic_pool.get(ResourceId32::new(2, 0)).unwrap();

        // Check semantic types
        assert_eq!(semantic0.type_semantic(), &SemanticType::RoofSurface);
        assert_eq!(semantic1.type_semantic(), &SemanticType::WallSurface);
        assert!(
            matches!(semantic2.type_semantic(), SemanticType::Extension(s) if s == "+CustomType")
        );

        // Check parent-child relationships
        assert!(semantic0.has_children());
        assert_eq!(semantic0.children().unwrap().len(), 2);
        assert!(
            semantic0
                .children()
                .unwrap()
                .contains(&ResourceId32::new(1, 0))
        );
        assert!(
            semantic0
                .children()
                .unwrap()
                .contains(&ResourceId32::new(2, 0))
        );

        assert!(semantic1.has_parent());
        assert_eq!(semantic1.parent().unwrap(), &ResourceId32::new(0, 0));

        assert!(semantic2.has_parent());
        assert_eq!(semantic2.parent().unwrap(), &ResourceId32::new(0, 0));
    }
}
