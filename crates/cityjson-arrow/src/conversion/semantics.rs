use crate::conversion::attributes::{attributes_to_arrow, map_field};
use crate::error::{Error, Result};
use arrow::array::{
    ArrayRef, ListBuilder, RecordBatch, StringBuilder, StringDictionaryBuilder, UInt32Builder,
};
use arrow::datatypes::{DataType, Field, Int8Type, Schema};
use cityjson::prelude::{
    Attributes, DefaultResourcePool, OwnedStringStorage, ResourceId32, ResourcePool, SemanticTrait,
    StringStorage,
};
use cityjson::v2_0::{Semantic, SemanticType};
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
                extension_builder.append_value(ext_value.as_str());
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
}
