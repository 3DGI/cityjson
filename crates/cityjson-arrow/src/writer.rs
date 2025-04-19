//! Write to Arrow data, to file or IPC, from a [cityjson::v2_0::CityModel] object.
//! Write Arrow data from a `CityModelArrowParts` structure to files or streams.
//!
//! This module provides functions for writing the components of a CityJSON model
//! represented as Arrow RecordBatches to various output formats.

use arrow::ipc::writer::{FileWriter, StreamWriter};
use arrow::record_batch::RecordBatch;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use cityjson::prelude::ResourceId32;
use nanoserde::{DeJson, SerJson};
use crate::CityModelArrowParts;
use crate::error::{Result};

#[derive(Debug, DeJson, SerJson)]
pub struct ArrowManifest {
    type_citymodel: String,
    version: Option<String>,
    components: ArrowComponents
}

#[derive(Debug, DeJson, SerJson)]
pub struct ArrowComponents {
    pub extensions: bool,
    pub extra: bool,
    pub metadata: bool,
    pub cityobjects: bool,
    pub transform: bool,
    pub vertices: bool,
    pub geometries: bool,
    pub template_vertices: bool,
    pub template_geometries: bool,
    pub semantics: bool,
    pub materials: bool,
    pub textures: bool,
    pub vertices_texture: bool,
}

/// Write CityModelArrowParts to a directory with separate Arrow IPC files for each component
///
/// This function creates a directory structure and writes each component of the CityModelArrowParts
/// as a separate Arrow IPC file. It also generates a manifest.json file that describes the contents
/// and structure of the files.
///
/// # Parameters
///
/// * `parts` - The CityModelArrowParts structure containing Arrow data components
/// * `dir_path` - The directory path where the files should be written
///
/// # Returns
///
/// `Result<()>` - Ok(()) if successful, or an Error if writing fails
pub fn write_to_directory<P: AsRef<Path>>(parts: &CityModelArrowParts, dir_path: P) -> Result<()> {
    let dir_path = dir_path.as_ref();

    // Create directory if it doesn't exist
    fs::create_dir_all(dir_path)?;

    // Write manifest file with type_citymodel, version, and component filenames
    let manifest = ArrowManifest {
        type_citymodel: format!("{:?}", parts.type_citymodel),
        version: parts.version.map(|v| format!("{}", v)),
        components: ArrowComponents {
            extensions: parts.extensions.is_some(),
            extra: parts.extra.is_some(),
            metadata: parts.metadata.is_some(),
            cityobjects: parts.cityobjects.is_some(),
            transform: parts.transform.is_some(),
            vertices: parts.vertices.is_some(),
            geometries: parts.geometries.is_some(),
            template_vertices: parts.template_vertices.is_some(),
            template_geometries: parts.template_geometries.is_some(),
            semantics: parts.semantics.is_some(),
            materials: parts.materials.is_some(),
            textures: parts.textures.is_some(),
            vertices_texture: parts.vertices_texture.is_some(),
        },
    };
    let manifest_path = dir_path.join("manifest.json");
    let mut manifest_file = File::create(manifest_path)?;
    let manifest_json = manifest.serialize_json();
    manifest_file.write_all(manifest_json.as_bytes())?;

    // Helper function to write a RecordBatch to an Arrow file
    let write_batch = |batch: &RecordBatch, name: &str| -> Result<()> {
        let file_path = dir_path.join(format!("{}.arrow", name));
        let file = File::create(file_path)?;
        let mut writer = FileWriter::try_new(file, &batch.schema())?;
        writer.write(batch)?;
        writer.finish()?;
        Ok(())
    };

    // Write each component if it exists
    if let Some(batch) = &parts.extensions {
        write_batch(batch, "extensions")?;
    }
    if let Some(batch) = &parts.extra {
        write_batch(batch, "extra")?;
    }
    if let Some(batch) = &parts.metadata {
        write_batch(batch, "metadata")?;
    }
    if let Some(batch) = &parts.cityobjects {
        write_batch(batch, "cityobjects")?;
    }
    if let Some(batch) = &parts.transform {
        write_batch(batch, "transform")?;
    }
    if let Some(batch) = &parts.vertices {
        write_batch(batch, "vertices")?;
    }
    if let Some(batch) = &parts.geometries {
        write_batch(batch, "geometries")?;
    }
    if let Some(batch) = &parts.template_vertices {
        write_batch(batch, "template_vertices")?;
    }
    if let Some(batch) = &parts.template_geometries {
        write_batch(batch, "template_geometries")?;
    }
    if let Some(batch) = &parts.semantics {
        write_batch(batch, "semantics")?;
    }
    if let Some(batch) = &parts.materials {
        write_batch(batch, "materials")?;
    }
    if let Some(batch) = &parts.textures {
        write_batch(batch, "textures")?;
    }
    if let Some(batch) = &parts.vertices_texture {
        write_batch(batch, "vertices_texture")?;
    }

    Ok(())
}

/// Write a single component from CityModelArrowParts to an Arrow IPC stream
///
/// This function writes a specified component of the CityModelArrowParts to an Arrow IPC stream.
///
/// # Parameters
///
/// * `batch` - The RecordBatch containing the component data
/// * `stream` - Any type that implements the Write trait
///
/// # Returns
///
/// `Result<()>` - Ok(()) if successful, or an Error if writing fails
pub fn write_component_to_ipc_stream<W: Write + Send>(
    batch: &RecordBatch,
    mut stream: W,
) -> Result<()> {
    let mut ipc_writer = StreamWriter::try_new(&mut stream, &batch.schema())?;
    ipc_writer.write(batch)?;
    ipc_writer.finish()?;
    Ok(())
}

/// Convenience function to write a CityModelArrowParts directly from a CityModel
///
/// This function takes a CityModel, converts it to CityModelArrowParts using
/// the `citymodel_to_arrow_parts` function, and then writes it to a directory.
///
/// # Parameters
///
/// * `model` - The CityModel to write
/// * `dir_path` - The directory path where the files should be written
///
/// # Returns
///
/// `Result<()>` - Ok(()) if successful, or an Error if writing fails
pub fn write_citymodel<SS>(
    model: &cityjson::v2_0::CityModel<u32, ResourceId32, SS>,
    dir_path: impl AsRef<Path>,
) -> Result<()>
where
    SS: cityjson::prelude::StringStorage + Default,
    SS::String: AsRef<str> + Eq + std::hash::Hash + Clone + std::fmt::Debug + Default + std::fmt::Display,
{
    let parts = crate::citymodel_to_arrow_parts(model)?;
    write_to_directory(&parts, dir_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cityjson::prelude::*;
    use cityjson::v2_0::CityModel;
    use tempfile::tempdir;
    use super::Result;

    #[test]
    fn test_write_empty_model() -> Result<()> {
        let model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        let parts = crate::citymodel_to_arrow_parts(&model)?;

        let temp_dir = tempdir()?;
        let output_dir = temp_dir.path().join("empty_model");

        write_to_directory(&parts, &output_dir)?;

        // Check that the manifest file exists
        let manifest_path = output_dir.join("manifest.json");
        assert!(manifest_path.exists());

        Ok(())
    }

    #[test]
    fn test_write_model_with_data() -> Result<()> {
        // Create a simple model with some data
        let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // Add a vertex
        model.add_vertex(QuantizedCoordinate::new(10, 20, 30))?;

        // Convert to parts
        let parts = crate::citymodel_to_arrow_parts(&model)?;

        let temp_dir = tempdir()?;
        let output_dir = temp_dir.path().join("model_with_data");

        write_to_directory(&parts, &output_dir)?;

        // Check that vertices file exists
        let vertices_path = output_dir.join("vertices.arrow");
        assert!(vertices_path.exists());

        Ok(())
    }
}