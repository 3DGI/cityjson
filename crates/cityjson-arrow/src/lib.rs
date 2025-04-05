pub mod conversion;
pub mod error;

use arrow::record_batch::RecordBatch;
use cityjson::prelude::*;
use cityjson::v2_0::CityModel;
use std::fmt::{Debug, Display};
use std::hash::Hash;

pub struct CityModelArrowParts {
    pub metadata: Option<RecordBatch>,
    pub transform: Option<RecordBatch>,
    pub vertices: Option<RecordBatch>,
    pub template_vertices: Option<RecordBatch>,
    pub texture_vertices: Option<RecordBatch>,
    pub cityobjects: Option<RecordBatch>,
    pub geometries: Option<RecordBatch>,
    pub template_geometries: Option<RecordBatch>,
    pub semantics: Option<RecordBatch>,
    pub materials: Option<RecordBatch>,
    pub textures: Option<RecordBatch>,
}

/// Converts a cityjson-rs CityModel (v2.0) into its constituent Arrow parts.
pub fn citymodel_to_arrow_parts<VR, RR, SS>(
    model: &CityModel<VR, RR, SS>,
) -> error::Result<CityModelArrowParts>
where
    VR: VertexRef + Default, // Added Default constraint if needed by VerticesBuilder etc.
    RR: ResourceRef + Default, // Added Default constraint
    SS: StringStorage + Default, // Added Default constraint
    SS::String:
        AsRef<str> + Eq + PartialEq + PartialOrd + Ord + Hash + Clone + Debug + Default + Display,
{
    let metadata_batch = match model.metadata() {
        None => None,
        Some(metadata) => {
            let struct_array = conversion::metadata::metadata_to_arrow(metadata)?;
            // RecordBatch::try_from(StructArray) is Infallible
            RecordBatch::try_from(&struct_array).ok()
        }
    };

    let transform_batch = model
        .transform()
        .map(|t| conversion::transform::transform_to_arrow(t))
        .transpose()?;

    // Convert vertices (example using your existing function structure)
    let vertices_batch = if !model.vertices().is_empty() {
        Some(conversion::vertices::vertices_to_batch(
            model.vertices().as_slice(),
        ))
    } else {
        None
    };

    Ok(CityModelArrowParts {
        metadata: metadata_batch,
        transform: transform_batch,
        vertices: vertices_batch,
        template_vertices: None,   // Placeholder
        texture_vertices: None,    // Placeholder
        cityobjects: None,         // Placeholder
        geometries: None,          // Placeholder
        template_geometries: None, // Placeholder
        semantics: None,           // Placeholder
        materials: None,           // Placeholder
        textures: None,            // Placeholder
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cityjson::v2_0::CityModel;
    use cityjson::CityModelType;

    #[test]
    fn test_empty_model_conversion() {
        // Use specific types expected by your conversion functions for now
        let model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        let parts = citymodel_to_arrow_parts(&model).expect("Conversion failed");

        assert!(parts.metadata.is_none());
        assert!(parts.transform.is_none());
        assert!(parts.vertices.is_none());
        // ... assert other parts are None ...
    }

    #[test]
    fn test_model_with_metadata() {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        model.metadata_mut().set_title("Test Title".to_string());

        let parts = citymodel_to_arrow_parts(&model).expect("Conversion failed");

        assert!(parts.metadata.is_some());
        let metadata_batch = parts.metadata.unwrap();
        assert_eq!(metadata_batch.num_rows(), 1);
        // Further checks on metadata content...

        assert!(parts.transform.is_none()); // Transform wasn't set
        assert!(parts.vertices.is_none()); // Vertices weren't added
    }

    #[test]
    fn test_model_with_transform() {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        model.transform_mut().set_scale([0.1, 0.1, 0.1]);
        model.transform_mut().set_translate([10.0, 20.0, 30.0]);

        let parts = citymodel_to_arrow_parts(&model).expect("Conversion failed");

        assert!(parts.transform.is_some());
        let transform_batch = parts.transform.unwrap();
        assert_eq!(transform_batch.num_rows(), 1);
        // Further checks on transform content...

        assert!(parts.metadata.is_none()); // Metadata wasn't set
    }

    #[test]
    fn test_model_with_vertices() {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        model.add_vertex(QuantizedCoordinate::new(1, 2, 3)).unwrap();
        model.add_vertex(QuantizedCoordinate::new(4, 5, 6)).unwrap();

        let parts = citymodel_to_arrow_parts(&model).expect("Conversion failed");

        assert!(parts.vertices.is_some());
        let vertices_batch = parts.vertices.unwrap();
        assert_eq!(vertices_batch.num_rows(), 2); // Should have 2 rows (vertices)
        // Further checks on vertex content...
    }
}
