use crate::conversion::attributes::{attributes_to_arrow, map_field};
use crate::error::{Error, Result};
use arrow::array::{
    ArrayRef, FixedSizeListBuilder, Float64Builder, ListBuilder, RecordBatch, StringBuilder,
    StringDictionaryBuilder, UInt32Builder,
};
use arrow::datatypes::{DataType, Field, Int8Type, Schema};
use cityjson::prelude::{
    Attributes, BBoxTrait, CityObjectTrait, CityObjectsTrait, OwnedStringStorage, ResourceId32,
    StringStorage,
};
use cityjson::v2_0::{CityObjectType, CityObjects};
use std::hash::Hash;
use std::sync::Arc;

pub fn cityobjects_to_arrow<SS>(cityobjects: &CityObjects<SS, ResourceId32>) -> Result<RecordBatch>
where
    SS: StringStorage + Default,
    SS::String: AsRef<str> + Eq + Hash, // Constraints from Attributes map keys
{
    let schema = cityobjects_schema();
    let num_rows = cityobjects.len();

    // Special case for empty pools
    if num_rows == 0 {
        return Ok(RecordBatch::new_empty(Arc::new(schema)));
    }

    // --- Initialize Builders ---
    // We need capacity hints based on expected data size.
    let mut id_builder = UInt32Builder::with_capacity(num_rows);
    let mut type_builder = StringDictionaryBuilder::<Int8Type>::new(); // TODO: estimate capacity
    let mut extension_builder = StringBuilder::new();
    let mut geometries_builder = ListBuilder::with_capacity(UInt32Builder::new(), num_rows);
    let mut geographical_extent_builder =
        FixedSizeListBuilder::with_capacity(Float64Builder::new(), 6, num_rows);
    let mut children_builder = ListBuilder::with_capacity(UInt32Builder::new(), num_rows);
    let mut parents_builder = ListBuilder::with_capacity(UInt32Builder::new(), num_rows);

    // For attributes, collect arrays to combine later
    let mut attribute_arrays = Vec::with_capacity(num_rows);
    let mut extra_arrays = Vec::with_capacity(num_rows);

    // --- Iterate and Append Data ---
    for (resource_ref, cityobject) in cityobjects.iter() {
        // ResourceId in pool
        id_builder.append_value(resource_ref.index());

        // Process semantic type with extension
        match cityobject.type_cityobject() {
            CityObjectType::Extension(ext_value) => {
                type_builder.append_value("Extension");
                extension_builder.append_value(ext_value.as_ref());
            }
            other_type => {
                type_builder.append_value(&other_type.to_string());
                extension_builder.append_null();
            }
        }

        // Geometries
        if let Some(geometries_vec) = cityobject.geometry() {
            let indices_builder = geometries_builder.values();
            // NOTE: Wanted to use `extend` here but that builds an Nullable array
            for child in geometries_vec {
                indices_builder.append_value(child.index());
            }
            geometries_builder.append(true);
        } else {
            geometries_builder.append(false); // Append null list
        }

        // Geographical extent
        if let Some(geographical_extent) = cityobject.geographical_extent() {
            let values_builder = geographical_extent_builder.values();
            for value in geographical_extent.as_slice() {
                values_builder.append_value(*value);
            }
            geographical_extent_builder.append(true);
        } else {
            geographical_extent_builder.append(false); // Append null list
        }

        // Children
        if let Some(children_vec) = cityobject.children() {
            let indices_builder = children_builder.values();
            // NOTE: Wanted to use `extend` here but that builds an Nullable array
            for child in children_vec {
                indices_builder.append_value(child.index());
            }
            children_builder.append(true);
        } else {
            children_builder.append(false); // Append null list
        }

        // Parents
        if let Some(parents_vec) = cityobject.parents() {
            let indices_builder = parents_builder.values();
            // NOTE: Wanted to use `extend` here but that builds an Nullable array
            for child in parents_vec {
                indices_builder.append_value(child.index());
            }
            parents_builder.append(true);
        } else {
            parents_builder.append(false); // Append null list
        }

        // Attributes
        if let Some(attributes) = cityobject.attributes() {
            // Convert these attributes to a MapArray
            let (_, map_array) = attributes_to_arrow(attributes, "attributes")?;
            attribute_arrays.push(Arc::new(map_array) as ArrayRef);
        } else {
            // Create an empty MapArray with the correct structure
            let empty_attributes = Attributes::<OwnedStringStorage, ResourceId32>::new();
            let (_, map_array) = attributes_to_arrow(&empty_attributes, "attributes")?;
            attribute_arrays.push(Arc::new(map_array) as ArrayRef);
        }

        // Extra properties
        if let Some(extra) = cityobject.extra() {
            // Convert these extra properties to a MapArray
            let (_, map_array) = attributes_to_arrow(extra, "extra")?;
            extra_arrays.push(Arc::new(map_array) as ArrayRef);
        } else {
            // Create an empty MapArray with the correct structure
            let empty_extra = Attributes::<OwnedStringStorage, ResourceId32>::new();
            let (_, map_array) = attributes_to_arrow(&empty_extra, "extra")?;
            extra_arrays.push(Arc::new(map_array) as ArrayRef);
        }
    }

    // Concatenate all attribute arrays
    let combined_attributes = arrow::compute::concat(
        &attribute_arrays
            .iter()
            .map(|a| a.as_ref())
            .collect::<Vec<_>>(),
    )?;
    // Concatenate all extra arrays
    let combined_extra =
        arrow::compute::concat(&extra_arrays.iter().map(|a| a.as_ref()).collect::<Vec<_>>())?;

    // Create basic arrays
    let arrays: Vec<ArrayRef> = vec![
        Arc::new(id_builder.finish()),
        Arc::new(type_builder.finish()),
        Arc::new(extension_builder.finish()),
        Arc::new(geometries_builder.finish()),
        combined_attributes,
        Arc::new(geographical_extent_builder.finish()),
        Arc::new(children_builder.finish()),
        Arc::new(parents_builder.finish()),
        combined_extra,
    ];

    RecordBatch::try_new(Arc::new(schema), arrays).map_err(Error::from)
}

pub fn cityobjects_schema() -> Schema {
    // Define the schema for the CityObjects RecordBatch
    Schema::new(vec![
        Field::new("id", DataType::UInt32, false),
        Field::new(
            "type_cityobject",
            DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)),
            false,
        ),
        Field::new("extension_value", DataType::Utf8, true),
        // TODO: the list items should be not-nullable, but it seems tha the PrimitiveBuilder only builds nullable arrays and I'm lazy now to manually set up the builder as for example in metadata.geographical_extent, but with correct offsets
        Field::new_list(
            "geometries",
            Field::new_list_field(DataType::UInt32, true),
            true,
        ),
        map_field("attributes"),
        Field::new(
            "geographical_extent",
            DataType::FixedSizeList(Arc::new(Field::new_list_field(DataType::Float64, false)), 6),
            true,
        ),
        // TODO: the list items should be not-nullable, but it seems tha the PrimitiveBuilder only builds nullable arrays and I'm lazy now to manually set up the builder as for example in metadata.geographical_extent, but with correct offsets
        Field::new_list(
            "children",
            Field::new_list_field(DataType::UInt32, true),
            true,
        ),
        // TODO: the list items should be not-nullable, but it seems tha the PrimitiveBuilder only builds nullable arrays and I'm lazy now to manually set up the builder as for example in metadata.geographical_extent, but with correct offsets
        Field::new_list(
            "parents",
            Field::new_list_field(DataType::UInt32, true),
            true,
        ),
        map_field("extra"),
    ])
}
