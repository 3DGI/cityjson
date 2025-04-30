use crate::conversion::attributes::{attributes_to_arrow, map_field};
use crate::error::{Error, Result};
use arrow::array::{
    Array, ArrayRef, BooleanArray, DictionaryArray, FixedSizeListArray, FixedSizeListBuilder,
    Float64Array, Float64Builder, Int64Array, ListArray, ListBuilder, MapArray, RecordBatch,
    StringArray, StringBuilder, StringDictionaryBuilder, UInt32Array, UInt32Builder, UInt64Array,
    UnionArray,
};
use arrow::datatypes::{DataType, Field, Int8Type, Schema};
use cityjson::prelude::{
    AttributeValue, Attributes, BBox, BBoxTrait, CityObjectTrait, CityObjectsTrait,
    OwnedStringStorage, ResourceId32, StringStorage,
};
use cityjson::v2_0::{CityObject, CityObjectType, CityObjects};
use std::collections::HashMap;
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
            values_builder.append_slice(geographical_extent.as_slice());
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

/// Converts an Arrow RecordBatch to a cityjson-rs CityObjects container.
///
/// This function reconstructs CityObjects from Arrow's columnar format,
/// preserving all properties, attributes, and relationships.
///
/// # Parameters
///
/// * `batch` - The Arrow RecordBatch containing CityObjects data
///
/// # Returns
///
/// A Result containing the populated CityObjects pool or an error
pub fn arrow_to_cityobjects<SS: StringStorage + Default>(
    batch: &RecordBatch,
) -> Result<CityObjects<SS, ResourceId32>>
where
    SS::String: AsRef<str> + From<String> + Eq + Hash,
{
    // Create a new empty CityObjects container
    let mut cityobjects = CityObjects::<SS, ResourceId32>::new();

    // If the batch is empty, return the empty container
    if batch.num_rows() == 0 {
        return Ok(cityobjects);
    }

    // Extract the required columns
    let id_array = batch
        .column_by_name("id")
        .ok_or_else(|| Error::MissingField("id".to_string()))?
        .as_any()
        .downcast_ref::<UInt32Array>()
        .ok_or_else(|| Error::Conversion("Failed to downcast id array".to_string()))?;

    let type_array = batch
        .column_by_name("type_cityobject")
        .ok_or_else(|| Error::MissingField("type_cityobject".to_string()))?
        .as_any()
        .downcast_ref::<DictionaryArray<Int8Type>>()
        .ok_or_else(|| Error::Conversion("Failed to downcast type_cityobject array".to_string()))?;

    let extension_array = batch
        .column_by_name("extension_value")
        .ok_or_else(|| Error::MissingField("extension_value".to_string()))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| Error::Conversion("Failed to downcast extension_value array".to_string()))?;

    let geometries_array = batch
        .column_by_name("geometries")
        .ok_or_else(|| Error::MissingField("geometries".to_string()))?
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| Error::Conversion("Failed to downcast geometries array".to_string()))?;

    let attributes_array = batch
        .column_by_name("attributes")
        .ok_or_else(|| Error::MissingField("attributes".to_string()))?
        .as_any()
        .downcast_ref::<MapArray>()
        .ok_or_else(|| Error::Conversion("Failed to downcast attributes array".to_string()))?;

    let geographical_extent_array = batch
        .column_by_name("geographical_extent")
        .ok_or_else(|| Error::MissingField("geographical_extent".to_string()))?
        .as_any()
        .downcast_ref::<FixedSizeListArray>()
        .ok_or_else(|| {
            Error::Conversion("Failed to downcast geographical_extent array".to_string())
        })?;

    let children_array = batch
        .column_by_name("children")
        .ok_or_else(|| Error::MissingField("children".to_string()))?
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| Error::Conversion("Failed to downcast children array".to_string()))?;

    let parents_array = batch
        .column_by_name("parents")
        .ok_or_else(|| Error::MissingField("parents".to_string()))?
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| Error::Conversion("Failed to downcast parents array".to_string()))?;

    let extra_array = batch
        .column_by_name("extra")
        .ok_or_else(|| Error::MissingField("extra".to_string()))?
        .as_any()
        .downcast_ref::<MapArray>()
        .ok_or_else(|| Error::Conversion("Failed to downcast extra array".to_string()))?;

    // First pass: Create all city objects without relationships
    let mut original_ids = Vec::with_capacity(batch.num_rows());
    let mut new_ids = Vec::with_capacity(batch.num_rows());

    for i in 0..batch.num_rows() {
        // Extract city object type
        let type_id = type_array.keys().value(i);
        let type_values = type_array.values();
        let type_dict = type_values
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("Expected StringArray for type dictionary");
        let type_value = type_dict.value(type_id as usize);

        // Create the CityObjectType
        let cityobject_type = if type_value == "Extension" {
            // For extension types, we need to get the extension value
            if extension_array.is_null(i) {
                return Err(Error::Conversion(
                    "Extension type without extension_value".to_string(),
                ));
            }
            let extension_value = extension_array.value(i);
            CityObjectType::Extension(SS::String::from(extension_value.to_string()))
        } else {
            // Parse the standard type
            parse_cityobject_type(type_value)?
        };

        // Create a new city object with ID
        let id_str = format!("id-{}", id_array.value(i));
        let mut cityobject = CityObject::new(SS::String::from(id_str), cityobject_type);

        // Set geometries if present
        if !geometries_array.is_null(i) {
            let geom_list = geometries_array.value(i);
            let geom_values = geom_list
                .as_any()
                .downcast_ref::<UInt32Array>()
                .ok_or_else(|| {
                    Error::Conversion("Failed to downcast geometry values".to_string())
                })?;

            for j in 0..geom_values.len() {
                if !geom_values.is_null(j) {
                    cityobject
                        .geometry_mut()
                        .push(ResourceId32::new(geom_values.value(j), 0));
                }
            }
        }

        // Set attributes if present
        if !attributes_array.is_null(i) {
            let mut attributes = Attributes::<SS, ResourceId32>::new();

            // Get the entries struct array for this row
            let entries = attributes_array.value(i);

            // Extract keys and values
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
                            .ok_or_else(|| Error::Conversion("Expected UInt64Array".to_string()))?;
                        AttributeValue::Unsigned(array.value(values.value_offset(j)))
                    }
                    3 => {
                        let array = values
                            .child(3)
                            .as_any()
                            .downcast_ref::<Int64Array>()
                            .ok_or_else(|| Error::Conversion("Expected Int64Array".to_string()))?;
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
                            .ok_or_else(|| Error::Conversion("Expected StringArray".to_string()))?;
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
                *cityobject.attributes_mut() = attributes;
            }
        }

        // Set geographical extent if present
        if !geographical_extent_array.is_null(i) {
            let extent_list = geographical_extent_array.value(i);
            let extent_values = extent_list
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| Error::Conversion("Failed to downcast extent values".to_string()))?;

            if extent_values.len() == 6 {
                let bbox = BBox::new(
                    extent_values.value(0),
                    extent_values.value(1),
                    extent_values.value(2),
                    extent_values.value(3),
                    extent_values.value(4),
                    extent_values.value(5),
                );
                cityobject.set_geographical_extent(Some(bbox));
            }
        }

        // Set extra properties if present
        if !extra_array.is_null(i) {
            let mut extra_attrs = Attributes::<SS, ResourceId32>::new();

            // Get the entries struct array for this row
            let entries = extra_array.value(i);

            // Extract keys and values
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
                            .ok_or_else(|| Error::Conversion("Expected UInt64Array".to_string()))?;
                        AttributeValue::Unsigned(array.value(values.value_offset(j)))
                    }
                    3 => {
                        let array = values
                            .child(3)
                            .as_any()
                            .downcast_ref::<Int64Array>()
                            .ok_or_else(|| Error::Conversion("Expected Int64Array".to_string()))?;
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
                            .ok_or_else(|| Error::Conversion("Expected StringArray".to_string()))?;
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

                extra_attrs.insert(key, attr_value);
            }

            if !extra_attrs.is_empty() {
                *cityobject.extra_mut() = extra_attrs;
            }
        }

        // Add the object to the container
        let original_id = id_array.value(i);
        let new_id = cityobjects.add(cityobject);

        original_ids.push(original_id);
        new_ids.push(new_id);
    }

    // Create a mapping from original IDs to new ResourceId32 objects
    let id_mapping: HashMap<u32, ResourceId32> = original_ids
        .into_iter()
        .zip(new_ids.iter().cloned())
        .collect();

    // Second pass: Set up relationships
    for i in 0..batch.num_rows() {
        let new_id = new_ids[i];

        // Set children if present
        if !children_array.is_null(i) {
            let children_list = children_array.value(i);
            let children_values = children_list
                .as_any()
                .downcast_ref::<UInt32Array>()
                .ok_or_else(|| {
                    Error::Conversion("Failed to downcast children values".to_string())
                })?;

            if children_values.len() > 0 {
                if let Some(cityobject) = cityobjects.get_mut(new_id) {
                    let children_vec = cityobject.children_mut();
                    for j in 0..children_values.len() {
                        if !children_values.is_null(j) {
                            let child_original_id = children_values.value(j);
                            if let Some(child_new_id) = id_mapping.get(&child_original_id) {
                                children_vec.push(*child_new_id);
                            }
                        }
                    }
                }
            }
        }

        // Set parents if present
        if !parents_array.is_null(i) {
            let parents_list = parents_array.value(i);
            let parents_values = parents_list
                .as_any()
                .downcast_ref::<UInt32Array>()
                .ok_or_else(|| {
                    Error::Conversion("Failed to downcast parents values".to_string())
                })?;

            if parents_values.len() > 0 {
                if let Some(cityobject) = cityobjects.get_mut(new_id) {
                    let parents_vec = cityobject.parents_mut();
                    for j in 0..parents_values.len() {
                        if !parents_values.is_null(j) {
                            let parent_original_id = parents_values.value(j);
                            if let Some(parent_new_id) = id_mapping.get(&parent_original_id) {
                                parents_vec.push(*parent_new_id);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(cityobjects)
}

/// Parses a city object type string into a CityObjectType enum
fn parse_cityobject_type<SS: StringStorage>(value: &str) -> Result<CityObjectType<SS>> {
    match value {
        "Bridge" => Ok(CityObjectType::Bridge),
        "BridgePart" => Ok(CityObjectType::BridgePart),
        "BridgeInstallation" => Ok(CityObjectType::BridgeInstallation),
        "BridgeConstructiveElement" => Ok(CityObjectType::BridgeConstructiveElement),
        "BridgeRoom" => Ok(CityObjectType::BridgeRoom),
        "BridgeFurniture" => Ok(CityObjectType::BridgeFurniture),
        "Building" => Ok(CityObjectType::Building),
        "BuildingPart" => Ok(CityObjectType::BuildingPart),
        "BuildingInstallation" => Ok(CityObjectType::BuildingInstallation),
        "BuildingConstructiveElement" => Ok(CityObjectType::BuildingConstructiveElement),
        "BuildingFurniture" => Ok(CityObjectType::BuildingFurniture),
        "BuildingStorey" => Ok(CityObjectType::BuildingStorey),
        "BuildingRoom" => Ok(CityObjectType::BuildingRoom),
        "BuildingUnit" => Ok(CityObjectType::BuildingUnit),
        "CityFurniture" => Ok(CityObjectType::CityFurniture),
        "CityObjectGroup" => Ok(CityObjectType::CityObjectGroup),
        "Default" => Ok(CityObjectType::Default),
        "GenericCityObject" => Ok(CityObjectType::GenericCityObject),
        "LandUse" => Ok(CityObjectType::LandUse),
        "OtherConstruction" => Ok(CityObjectType::OtherConstruction),
        "PlantCover" => Ok(CityObjectType::PlantCover),
        "SolitaryVegetationObject" => Ok(CityObjectType::SolitaryVegetationObject),
        "TINRelief" => Ok(CityObjectType::TINRelief),
        "WaterBody" => Ok(CityObjectType::WaterBody),
        "Road" => Ok(CityObjectType::Road),
        "Railway" => Ok(CityObjectType::Railway),
        "Waterway" => Ok(CityObjectType::Waterway),
        "TransportSquare" => Ok(CityObjectType::TransportSquare),
        "Tunnel" => Ok(CityObjectType::Tunnel),
        "TunnelPart" => Ok(CityObjectType::TunnelPart),
        "TunnelInstallation" => Ok(CityObjectType::TunnelInstallation),
        "TunnelConstructiveElement" => Ok(CityObjectType::TunnelConstructiveElement),
        "TunnelHollowSpace" => Ok(CityObjectType::TunnelHollowSpace),
        "TunnelFurniture" => Ok(CityObjectType::TunnelFurniture),
        _ => Err(Error::Conversion(format!(
            "Unknown city object type: {}",
            value
        ))),
    }
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
    use arrow::array::{Array, AsArray, DictionaryArray, FixedSizeListArray, ListArray};
    use arrow::datatypes::Int8Type;
    use cityjson::prelude::{AttributeValue, BBox, ResourceId32};
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

    #[test]
    fn test_arrow_to_cityobjects() {
        // Create test CityObjects
        let mut original_cityobjects = OwnedCityObjects::<ResourceId32>::new();

        // Create building object
        let mut building = CityObject::new("building-1".to_string(), CityObjectType::Building);
        building
            .attributes_mut()
            .insert("height".to_string(), AttributeValue::Float(25.5));
        building
            .attributes_mut()
            .insert("year_built".to_string(), AttributeValue::Integer(1985));
        building.set_geographical_extent(Some(BBox::new(100.0, 200.0, 0.0, 150.0, 250.0, 25.5)));
        building.geometry_mut().push(ResourceId32::new(1, 0));
        building.geometry_mut().push(ResourceId32::new(2, 0));

        // Create extension type object
        let mut custom_obj = CityObject::new(
            "custom-1".to_string(),
            CityObjectType::Extension("+CustomFeature".to_string()),
        );

        // Set up relationships
        let building_ref = original_cityobjects.add(building);
        custom_obj.children_mut().push(building_ref);
        original_cityobjects.add(custom_obj);

        // Convert to Arrow
        let batch = cityobjects_to_arrow(&original_cityobjects).unwrap();

        // Convert back to CityObjects
        let result = arrow_to_cityobjects::<OwnedStringStorage>(&batch).unwrap();

        // Verify result
        assert_eq!(result.len(), 2, "Should have 2 CityObjects");

        // Verify objects and relationships
        let mut has_building = false;
        let mut has_custom = false;

        for (_, obj) in result.iter() {
            match obj.type_cityobject() {
                CityObjectType::Building => {
                    has_building = true;
                    assert_eq!(
                        obj.attributes().unwrap().get("height"),
                        Some(&AttributeValue::Float(25.5))
                    );
                    assert_eq!(obj.geometry().unwrap().len(), 2);
                }
                CityObjectType::Extension(ext) => {
                    has_custom = true;
                    assert_eq!(ext.as_str(), "+CustomFeature");
                    assert_eq!(obj.children().unwrap().len(), 1);
                }
                _ => panic!("Unexpected object type"),
            }
        }

        assert!(has_building && has_custom, "Missing expected objects");
    }
}
