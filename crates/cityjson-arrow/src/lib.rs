pub mod conversion;
pub mod error;

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
) -> error::Result<CityModelArrowParts>
where
    SS: StringStorage + Default,
    SS::String:
        AsRef<str> + Eq + PartialEq + PartialOrd + Ord + Hash + Clone + Debug + Default + Display,
{
    // todo: A feature does not have a version, but it is stored in the metadata. This
    //  code only verifies models, not features.
    if let Some(ref version) = model.version() {
        if version != &CityJSONVersion::V2_0 {
            return Err(error::Error::Unsupported(format!(
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

    let geometries_batch = if model.geometries().is_empty() {
        None
    } else {
        Some(conversion::geometry::geometries_to_arrow(
            model.geometries(),
        )?)
    };

    let semantics_batch = if model.semantics().is_empty() {
        None
    } else {
        Some(conversion::semantics::semantics_to_arrow(
            model.semantics(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use cityjson::v2_0::CityModel;
    use cityjson::CityModelType;
    #[test]
    fn test_empty_model_conversion() {
        let model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        let parts = citymodel_to_arrow_parts(&model).expect("Conversion failed");

        assert_eq!(parts.type_citymodel, CityModelType::CityJSON);
        assert_eq!(parts.version, Some(CityJSONVersion::V2_0)); // Default is V2_0

        assert!(parts.metadata.is_none());
        assert!(parts.transform.is_none());
        assert!(parts.vertices.is_none());
        assert!(parts.extra.is_none());
        assert!(parts.extensions.is_none());
        // ... assert other batch parts are None ...
    }

    #[test]
    fn test_model_with_metadata() {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        model.metadata_mut().set_title("Test Title");

        let parts = citymodel_to_arrow_parts(&model).expect("Conversion failed");

        assert_eq!(parts.type_citymodel, CityModelType::CityJSON);
        assert_eq!(parts.version, Some(CityJSONVersion::V2_0));

        assert!(parts.metadata.is_some());
        let metadata_batch = parts.metadata.unwrap();
        assert_eq!(metadata_batch.num_rows(), 1);
        // Further checks on metadata content...

        assert!(parts.transform.is_none()); // Transform wasn't set
        assert!(parts.vertices.is_none()); // Vertices weren't added
    }

    // ... other tests remain valid ...

    #[test]
    fn test_model_with_extra_attrs() {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        model
            .extra_mut()
            .insert("my_extra_prop".to_string(), AttributeValue::Integer(123));

        let parts = citymodel_to_arrow_parts(&model).expect("Conversion failed");

        assert!(parts.extra.is_some());
        let extra_batch = parts.extra.unwrap();
        assert_eq!(extra_batch.num_rows(), 1);
        // Further checks needed here based on the actual map array structure
    }

    #[test]
    fn test_model_with_empty_extra_attrs() {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        // Ensure extra is Some but empty
        let _ = model.extra_mut();
        assert!(model.extra().is_some() && model.extra().unwrap().is_empty());

        let parts = citymodel_to_arrow_parts(&model).expect("Conversion failed");

        assert!(parts.extra.is_none()); // Empty attributes should result in None batch
    }
}
