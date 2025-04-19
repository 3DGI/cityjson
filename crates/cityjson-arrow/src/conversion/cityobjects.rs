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
            geographical_extent_builder.values().append_slice(&[0.0; 6]);
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
        // TODO: the list items should be not-nullable, but it seems tha the PrimitiveBuilder only builds nullable arrays and I'm lazy now to manually set up the builder as for example in metadata.geographical_extent, but with correct offsets
        Field::new(
            "geographical_extent",
            DataType::FixedSizeList(Arc::new(Field::new_list_field(DataType::Float64, true)), 6),
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

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{
        Array, AsArray, DictionaryArray, FixedSizeListArray, ListArray,
    };
    use arrow::datatypes::Int8Type;
    use cityjson::prelude::{AttributeValue, BBox, ResourceId32 };
    use cityjson::v2_0::{CityObject, CityObjectType, OwnedCityObjects};

    #[test]
    fn test_cityobjects_to_arrow() {
        // Create a collection of city objects
        let mut cityobjects = OwnedCityObjects::<ResourceId32>::new();

        // Create first city object - a building
        let mut building = CityObject::new("building-1".to_string(), CityObjectType::Building);

        // Add attributes
        let attributes = building.attributes_mut();
        attributes.insert("height".to_string(), AttributeValue::Float(25.5));
        attributes.insert("year_built".to_string(), AttributeValue::Integer(1985));
        attributes.insert(
            "name".to_string(),
            AttributeValue::String("Main Tower".to_string()),
        );

        // Add geographical extent
        building.set_geographical_extent(Some(BBox::new(100.0, 200.0, 0.0, 150.0, 250.0, 25.5)));

        // Add geometry references
        building.geometry_mut().push(ResourceId32::new(1, 0));
        building.geometry_mut().push(ResourceId32::new(2, 0));

        // Second city object - an extension type
        let mut custom_obj = CityObject::new(
            "custom-1".to_string(),
            CityObjectType::Extension("+CustomFeature".to_string()),
        );

        // Add children/parents to demonstrate relationship
        let building_ref = cityobjects.add(building);
        custom_obj.children_mut().push(building_ref.clone());

        let custom_ref = cityobjects.add(custom_obj);

        // Now convert to Arrow
        let batch =
            cityobjects_to_arrow(&cityobjects).expect("Failed to convert cityobjects to Arrow");

        // Verify basic structure
        assert_eq!(
            batch.num_rows(),
            2,
            "Batch should have 2 rows (one per city object)"
        );
        assert_eq!(
            batch.num_columns(),
            9,
            "Batch should have 9 columns as per schema"
        );

        // Verify IDs (first column should be the object IDs)
        let id_array = batch
            .column(0)
            .as_any()
            .downcast_ref::<arrow::array::UInt32Array>()
            .expect("First column should be UInt32Array of IDs");
        assert_eq!(id_array.value(0), building_ref.index());
        assert_eq!(id_array.value(1), custom_ref.index());

        // Verify object types
        let type_array = batch
            .column(1)
            .as_any()
            .downcast_ref::<DictionaryArray<Int8Type>>()
            .expect("Second column should be StringDictionaryArray of types");
        assert_eq!(type_array.values().as_string::<i32>().value(0), "Building");
        assert_eq!(type_array.values().as_string::<i32>().value(1), "Extension");

        // Verify extension values
        let extension_array = batch
            .column(2)
            .as_any()
            .downcast_ref::<arrow::array::StringArray>()
            .expect("Third column should be StringArray of extension values");
        assert!(
            extension_array.is_null(0),
            "Building should have null extension value"
        );
        assert_eq!(
            extension_array.value(1),
            "+CustomFeature",
            "Extension object should have correct value"
        );

        // Verify geometries
        let geometries_array = batch
            .column(3)
            .as_any()
            .downcast_ref::<ListArray>()
            .expect("Fourth column should be ListArray of geometry references");
        assert!(
            !geometries_array.is_null(0),
            "Building should have geometries"
        );

        // Get the underlying UInt32Array of geometry references for the building
        let building_geometries = geometries_array.value(0);
        let geom_ids = building_geometries
            .as_any()
            .downcast_ref::<arrow::array::UInt32Array>()
            .expect("Geometry list should contain UInt32Array");
        assert_eq!(
            geom_ids.len(),
            2,
            "Building should have 2 geometry references"
        );
        assert_eq!(geom_ids.value(0), 1);
        assert_eq!(geom_ids.value(1), 2);

        // Verify geographical extent
        let geo_extent_array = batch
            .column(5)
            .as_any()
            .downcast_ref::<FixedSizeListArray>()
            .expect("Sixth column should be FixedSizeListArray for geographical extent");

        assert!(
            !geo_extent_array.is_null(0),
            "Building should have geographical extent"
        );
        assert!(
            geo_extent_array.is_null(1),
            "Custom object should have null geographical extent"
        );

        // Get the values of the geographical extent
        let extent_values = geo_extent_array.value(0);
        let extent_array = extent_values
            .as_any()
            .downcast_ref::<arrow::array::Float64Array>()
            .expect("Geographical extent should contain Float64Array");

        assert_eq!(
            extent_array.len(),
            6,
            "Geographical extent should have 6 values"
        );
        assert_eq!(extent_array.value(0), 100.0); // minx
        assert_eq!(extent_array.value(1), 200.0); // miny
        assert_eq!(extent_array.value(2), 0.0); // minz
        assert_eq!(extent_array.value(3), 150.0); // maxx
        assert_eq!(extent_array.value(4), 250.0); // maxy
        assert_eq!(extent_array.value(5), 25.5); // maxz

        // Verify children relationships
        let children_array = batch
            .column(6)
            .as_any()
            .downcast_ref::<ListArray>()
            .expect("Seventh column should be ListArray of children references");

        assert!(
            children_array.is_null(0),
            "Building should have no children"
        );
        assert!(
            !children_array.is_null(1),
            "Custom object should have children"
        );

        let custom_children = children_array.value(1);
        let child_ids = custom_children
            .as_any()
            .downcast_ref::<arrow::array::UInt32Array>()
            .expect("Children list should contain UInt32Array");
        assert_eq!(child_ids.len(), 1, "Custom object should have 1 child");
        assert_eq!(child_ids.value(0), building_ref.index());

        // Verify parents relationships
        let parents_array = batch
            .column(7)
            .as_any()
            .downcast_ref::<ListArray>()
            .expect("Eighth column should be ListArray of parent references");

        // The building has no parents explicitly set in our test
        assert!(
            parents_array.is_null(0) || parents_array.value(0).len() == 0,
            "Building should have no parents"
        );
    }
}
