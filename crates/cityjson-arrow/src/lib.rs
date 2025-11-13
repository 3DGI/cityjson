pub mod conversion;
pub mod error;
pub mod reader;
pub mod writer;

use crate::conversion::attributes::arrow_to_attributes_owned;
use crate::conversion::cityobjects::arrow_to_cityobjects;
use crate::conversion::geometry::arrow_to_geometries;
use crate::conversion::metadata::arrow_to_metadata;
use crate::conversion::semantics::arrow_to_semantics;
use crate::conversion::transform::arrow_to_transform;
use crate::conversion::vertices::batch_to_vertices;
use crate::error::{Error, Result};
use arrow::array::{MapArray, StructArray};
use arrow::record_batch::RecordBatch;
use cityjson::prelude::*;
use cityjson::v2_0::CityModel;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::sync::Arc;

pub struct CityModelArrowParts {
    pub type_citymodel: CityModelType,
    pub version: Option<CityJSONVersion>,
    pub extensions: Option<RecordBatch>,
    pub extra: Option<RecordBatch>,
    pub metadata: Option<RecordBatch>,
    pub cityobjects: Option<RecordBatch>,
    pub transform: Option<RecordBatch>,
    pub vertices: Option<RecordBatch>,
    pub geometries: Option<RecordBatch>,
    pub template_vertices: Option<RecordBatch>,
    pub template_geometries: Option<RecordBatch>,
    pub semantics: Option<RecordBatch>,
    pub materials: Option<RecordBatch>,
    pub textures: Option<RecordBatch>,
    pub vertices_texture: Option<RecordBatch>,
}

/// Converts a cityjson-rs CityModel (v2.0) into its constituent Arrow parts.
pub fn citymodel_to_arrow_parts<SS>(
    model: &CityModel<u32, ResourceId32, SS>,
) -> Result<CityModelArrowParts>
where
    SS: StringStorage + Default,
    SS::String:
        AsRef<str> + Eq + PartialEq + PartialOrd + Ord + Hash + Clone + Debug + Default + Display,
{
    // todo: A feature does not have a version, but it is stored in the metadata. This
    //  code only verifies models, not features.
    if let Some(ref version) = model.version() {
        if version != &CityJSONVersion::V2_0 {
            return Err(Error::Unsupported(format!(
                "CityArrow currently only supports CityJSON v2.0, found v{}",
                version
            )));
        }
    }

    let metadata_batch = match model.metadata() {
        None => None,
        Some(metadata) => Option::from({
            let struct_array = conversion::metadata::metadata_to_arrow(metadata)?;
            // RecordBatch::try_from(StructArray) is Infallible
            RecordBatch::from(&struct_array)
        }),
    };

    let transform_batch = model
        .transform()
        .map(conversion::transform::transform_to_arrow)
        .transpose()?;

    // Convert vertices (example using your existing function structure)
    let vertices_batch = if !model.vertices().is_empty() {
        Some(conversion::vertices::vertices_to_batch(
            model.vertices().as_slice(),
        ))
    } else {
        None
    };

    let extra_batch = match model.extra() {
        None => None,
        Some(extra_attrs) => {
            if extra_attrs.is_empty() {
                None
            } else {
                let (schema, map_array) =
                    conversion::attributes::attributes_to_arrow(extra_attrs, "extra")?;
                Some(RecordBatch::try_new(
                    Arc::new(schema),
                    vec![Arc::new(map_array)],
                )?)
            }
        }
    };

    let geometries_batch = if model.iter_geometries().len() == 0 {
        None
    } else {
        Some(conversion::geometry::geometries_to_arrow(
            model.iter_geometries(),
        )?)
    };

    let semantics_batch = if model.iter_semantics().len() == 0 {
        None
    } else {
        Some(conversion::semantics::semantics_to_arrow(
            model.iter_semantics(),
        )?)
    };

    let cityobjects_batch = if model.cityobjects().is_empty() {
        None
    } else {
        Some(conversion::cityobjects::cityobjects_to_arrow(
            model.cityobjects(),
        )?)
    };

    Ok(CityModelArrowParts {
        type_citymodel: model.type_citymodel(),
        version: model.version(),
        extensions: None,
        extra: extra_batch,
        metadata: metadata_batch,
        cityobjects: cityobjects_batch,
        transform: transform_batch,
        vertices: vertices_batch,
        geometries: geometries_batch,
        template_vertices: None,
        template_geometries: None,
        semantics: semantics_batch,
        materials: None,
        textures: None,
        vertices_texture: None,
    })
}

/// Converts CityModelArrowParts back to a cityjson-rs CityModel (v2.0).
///
/// This function reconstructs a complete CityModel from its Arrow representation.
/// It utilizes the component converters to transform each Arrow RecordBatch back
/// into its corresponding cityjson-rs object.
///
/// # Parameters
///
/// * `parts` - The CityModelArrowParts containing Arrow data components
///
/// # Returns
///
/// A Result containing the reconstructed CityModel or an error
pub fn arrow_parts_to_citymodel(
    parts: &CityModelArrowParts,
) -> Result<CityModel<u32, ResourceId32, OwnedStringStorage>> {
    // Create a new empty CityModel with the specified type
    let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new(parts.type_citymodel);

    // Verify version compatibility
    if let Some(version) = parts.version {
        if version != CityJSONVersion::V2_0 {
            return Err(Error::Unsupported(format!(
                "CityArrow currently only supports CityJSON v2.0, found v{}",
                version
            )));
        }
    }

    // Convert and set metadata if present
    if let Some(metadata_batch) = &parts.metadata {
        // TODO: avoid cloning the batch
        let metadata = arrow_to_metadata(&StructArray::from(metadata_batch.clone()))?;
        *model.metadata_mut() = metadata;
    }

    // Convert and set transform if present
    if let Some(transform_batch) = &parts.transform {
        let transform = arrow_to_transform(transform_batch)?;
        *model.transform_mut() = transform;
    }

    // Convert and set vertices if present
    if let Some(vertices_batch) = &parts.vertices {
        let vertices = batch_to_vertices::<u32>(vertices_batch)?;
        *model.vertices_mut() = vertices;
    }

    // Convert and set extra attributes if present
    if let Some(extra_batch) = &parts.extra {
        let extra_array = extra_batch
            .column(0)
            .as_any()
            .downcast_ref::<MapArray>()
            .ok_or_else(|| Error::Conversion("Failed to get extra map".to_string()))?;

        let extra_attrs = arrow_to_attributes_owned(extra_array)?;
        *model.extra_mut() = extra_attrs;
    }

    // Convert and set geometries if present
    if let Some(geometries_batch) = &parts.geometries {
        let geometry_pool = arrow_to_geometries(geometries_batch)?;
        *model.iter_geometries_mut() = geometry_pool;
    }

    // Convert and set semantics if present
    if let Some(semantics_batch) = &parts.semantics {
        let semantics_pool = arrow_to_semantics(semantics_batch)?;
        // Transfer each semantic from the returned pool to the model
        for (_, semantic) in semantics_pool.iter() {
            model.add_semantic(semantic.clone());
        }
    }

    // Convert and set cityobjects if present
    if let Some(cityobjects_batch) = &parts.cityobjects {
        let cityobjects = arrow_to_cityobjects(cityobjects_batch)?;
        *model.cityobjects_mut() = cityobjects;
    }

    // The following components don't have conversion functions implemented yet:
    // - materials
    // - textures
    // - vertices_texture
    // - extensions
    // - template_vertices and template_geometries

    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writer::write_to_directory;
    use cityjson::v2_0::*;
    use std::env;
    use std::path::PathBuf;

    #[test]
    fn test_empty_model_conversion() {
        // Create an empty CityModel, convert to parts, then back
        let original_model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        let parts = citymodel_to_arrow_parts(&original_model).expect("Failed to convert to parts");

        let converted_model =
            arrow_parts_to_citymodel(&parts).expect("Failed to convert back to model");

        // Verify basic properties
        assert_eq!(converted_model.type_citymodel(), CityModelType::CityJSON);
        assert_eq!(converted_model.version(), Some(CityJSONVersion::V2_0));
        assert_eq!(converted_model.vertex_count(), 0);
        assert_eq!(converted_model.geometry_count(), 0);
        assert_eq!(converted_model.cityobjects().len(), 0);
    }

    #[test]
    fn test_model_with_metadata() {
        // Create a model with metadata
        let mut original_model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        original_model.metadata_mut().set_title("Test City");
        original_model
            .metadata_mut()
            .set_reference_system(CRS::new("EPSG:4326".to_string()));

        // Convert to parts and back
        let parts = citymodel_to_arrow_parts(&original_model).expect("Failed to convert to parts");
        let converted_model =
            arrow_parts_to_citymodel(&parts).expect("Failed to convert back to model");

        // Verify metadata was preserved
        assert_eq!(
            converted_model.metadata().unwrap().title(),
            Some("Test City")
        );
        assert_eq!(
            converted_model
                .metadata()
                .unwrap()
                .reference_system()
                .unwrap()
                .to_string(),
            "EPSG:4326"
        );
    }

    #[test]
    fn test_model_with_vertices() {
        // Create a model with vertices
        let mut original_model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        original_model
            .add_vertex(QuantizedCoordinate::new(10, 20, 30))
            .unwrap();
        original_model
            .add_vertex(QuantizedCoordinate::new(40, 50, 60))
            .unwrap();

        // Convert to parts and back
        let parts = citymodel_to_arrow_parts(&original_model).expect("Failed to convert to parts");
        let converted_model =
            arrow_parts_to_citymodel(&parts).expect("Failed to convert back to model");

        // Verify vertices were preserved
        assert_eq!(converted_model.vertex_count(), 2);
        assert_eq!(
            converted_model.get_vertex(VertexIndex::new(0)).unwrap(),
            &QuantizedCoordinate::new(10, 20, 30)
        );
        assert_eq!(
            converted_model.get_vertex(VertexIndex::new(1)).unwrap(),
            &QuantizedCoordinate::new(40, 50, 60)
        );
    }

    #[test]
    fn test_model_with_transform() {
        // Create a model with transform
        let mut original_model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        original_model
            .transform_mut()
            .set_scale([0.001, 0.001, 0.001]);
        original_model
            .transform_mut()
            .set_translate([1000.0, 2000.0, 0.0]);

        // Convert to parts and back
        let parts = citymodel_to_arrow_parts(&original_model).expect("Failed to convert to parts");
        let converted_model =
            arrow_parts_to_citymodel(&parts).expect("Failed to convert back to model");

        // Verify transform was preserved
        assert_eq!(
            converted_model.transform().unwrap().scale(),
            [0.001, 0.001, 0.001]
        );
        assert_eq!(
            converted_model.transform().unwrap().translate(),
            [1000.0, 2000.0, 0.0]
        );
    }

    #[test]
    fn test_model_with_cityobjects() {
        // Create a model with city objects
        let mut original_model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // Add a building
        let mut building = CityObject::new("building-1".to_string(), CityObjectType::Building);
        let height_id = building.attributes_mut().add(AttributeValue::Float(25.5));
        building.attributes_mut().insert("height".to_string(), height_id);
        let building_id = original_model.cityobjects_mut().add(building);

        // Add a bridge that references the building
        let mut bridge = CityObject::new("bridge-1".to_string(), CityObjectType::Bridge);
        bridge.children_mut().push(building_id);
        original_model.cityobjects_mut().add(bridge);

        // Convert to parts and back
        let parts = citymodel_to_arrow_parts(&original_model).expect("Failed to convert to parts");
        let converted_model =
            arrow_parts_to_citymodel(&parts).expect("Failed to convert back to model");

        // Verify city objects were preserved
        assert_eq!(converted_model.cityobjects().len(), 2);

        // Find the objects by type
        let mut found_building = false;
        let mut found_bridge = false;

        for (_, obj) in converted_model.cityobjects().iter() {
            match obj.type_cityobject() {
                CityObjectType::Building => {
                    found_building = true;
                    assert!(obj.attributes().is_some());
                    let attrs = obj.attributes().unwrap();
                    assert_eq!(attrs.get("height"), Some(&AttributeValue::Float(25.5)));
                }
                CityObjectType::Bridge => {
                    found_bridge = true;
                    assert!(obj.children().is_some());
                    assert_eq!(obj.children().unwrap().len(), 1);
                }
                _ => panic!("Unexpected city object type"),
            }
        }

        assert!(found_building, "Building not found");
        assert!(found_bridge, "Bridge not found");
    }

    #[test]
    fn test_complex_model_roundtrip() {
        // Create a model with multiple components
        let mut original_model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // Set metadata
        original_model
            .metadata_mut()
            .set_title("Complex Test Model");

        // Set transform
        original_model
            .transform_mut()
            .set_scale([0.001, 0.001, 0.001]);
        original_model
            .transform_mut()
            .set_translate([1000.0, 2000.0, 0.0]);

        // Create a semantic
        let roof_semantic = Semantic::new(SemanticType::RoofSurface);

        // Create a geometry
        let mut geometry_builder = GeometryBuilder::new(
            &mut original_model,
            GeometryType::MultiSurface,
            BuilderMode::Regular,
        )
        .with_lod(LoD::LoD2);

        let v0 = geometry_builder.add_point(QuantizedCoordinate::new(10, 20, 30));
        let v1 = geometry_builder.add_point(QuantizedCoordinate::new(40, 50, 60));
        let v2 = geometry_builder.add_point(QuantizedCoordinate::new(70, 80, 90));
        let v3 = geometry_builder.add_point(QuantizedCoordinate::new(100, 110, 120));
        let ring = geometry_builder.add_ring(&[v0, v1, v2, v3]).unwrap();
        geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring).unwrap();
        geometry_builder
            .set_semantic_surface(None, roof_semantic)
            .unwrap();

        let geometry_id = geometry_builder.build().unwrap();

        // Create a city object that uses this geometry
        let mut building =
            CityObject::new("building-complex".to_string(), CityObjectType::Building);
        building.geometry_mut().push(geometry_id);
        let height_id = building.attributes_mut().add(AttributeValue::Float(25.5));
        building.attributes_mut().insert("height".to_string(), height_id);
        original_model.cityobjects_mut().add(building);

        // Set extra properties at root level
        let project_id = original_model.extra_mut().add(AttributeValue::String("Test project".to_string()));
        original_model.extra_mut().insert("projectInfo".to_string(), project_id);

        // Convert to parts and back
        let parts = citymodel_to_arrow_parts(&original_model).expect("Failed to convert to parts");

        // DEBUG JSON
        // write_to_json_directory(&parts, "complex_model_test").expect("Failed to write JSON");
        // end DEBUG JSON
        // DEBUG ARROW
        let output_dir = env::var_os("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tests/output/lib_complex_model_roundtrip");
        dbg!(&output_dir);
        write_to_directory(&parts, output_dir).expect("Failed to write Arrow files");
        // end DEBUG ARROW

        let converted_model =
            arrow_parts_to_citymodel(&parts).expect("Failed to convert back to model");

        // Verify all components were preserved
        assert_eq!(
            converted_model.metadata().unwrap().title(),
            Some("Complex Test Model")
        );
        assert_eq!(
            converted_model.transform().unwrap().scale(),
            [0.001, 0.001, 0.001]
        );
        assert_eq!(converted_model.vertex_count(), 4);
        assert_eq!(converted_model.semantic_count(), 1);
        assert_eq!(converted_model.geometry_count(), 1);
        assert_eq!(converted_model.cityobjects().len(), 1);

        // Verify extra properties
        assert!(converted_model.extra().is_some());
        assert_eq!(
            converted_model.extra().unwrap().get("projectInfo"),
            Some(&AttributeValue::String("Test project".to_string()))
        );

        // Verify the building object
        let building_obj = converted_model.cityobjects().iter().next().unwrap().1;
        assert_eq!(building_obj.geometry().unwrap().len(), 1);
        assert_eq!(
            building_obj.attributes().unwrap().get("height"),
            Some(&AttributeValue::Float(25.5))
        );
    }
}
