//! Write to Arrow data, to file or IPC, from a [cityjson::v2_0::CityModel] object.
//! Write Arrow data from a `CityModelArrowParts` structure to files or streams.
//!
//! This module provides functions for writing the components of a CityJSON model
//! represented as Arrow RecordBatches to various output formats.

use crate::error::Result;
use crate::CityModelArrowParts;
use arrow::ipc::writer::{FileWriter, IpcWriteOptions, StreamWriter};
use arrow::record_batch::RecordBatch;
use cityjson::prelude::{ResourceId32, StringStorage};
use nanoserde::{DeJson, SerJson};
use std::fmt::{Debug, Display};
use std::fs::{self, File};
use std::hash::Hash;
use std::io::{Cursor, Write};
use std::path::Path;
// parquet
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;

#[derive(Debug, DeJson, SerJson)]
pub struct FileManifest {
    pub format: String, // "arrow" or "parquet"
    pub type_citymodel: String,
    pub version: Option<String>,
    pub components: FileComponents,
}

#[derive(Debug, DeJson, SerJson)]
pub struct FileComponents {
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
    let manifest = FileManifest {
        format: "arrow".to_string(),
        type_citymodel: format!("{:?}", parts.type_citymodel),
        version: parts.version.map(|v| format!("{}", v)),
        components: FileComponents {
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
pub fn write_citymodel_to_directory<SS>(
    model: &cityjson::v2_0::CityModel<u32, ResourceId32, SS>,
    dir_path: impl AsRef<Path>,
) -> Result<()>
where
    SS: StringStorage + Default,
    SS::String: AsRef<str> + Eq + Hash + Clone + Debug + Default + Display,
{
    let parts = crate::citymodel_to_arrow_parts(model)?;
    write_to_directory(&parts, dir_path)
}

/// Identifies the type of component being sent in the stream frame.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ComponentId {
    TypeCityModel = 1,
    Version = 2,
    Metadata = 3,
    Transform = 4,
    Vertices = 5,
    Geometries = 6,
    Semantics = 7,
    CityObjects = 8,
    Extensions = 9,
    Extra = 10,
    TemplateVertices = 11,
    TemplateGeometries = 12,
    Materials = 13,
    Textures = 14,
    VerticesTexture = 15,
    // EndStream is used to indicate the end of the stream
    EndStream = 255,
}

impl TryFrom<u8> for ComponentId {
    type Error = crate::error::Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            1 => Ok(ComponentId::TypeCityModel),
            2 => Ok(ComponentId::Version),
            3 => Ok(ComponentId::Metadata),
            4 => Ok(ComponentId::Transform),
            5 => Ok(ComponentId::Vertices),
            6 => Ok(ComponentId::Geometries),
            7 => Ok(ComponentId::Semantics),
            8 => Ok(ComponentId::CityObjects),
            9 => Ok(ComponentId::Extensions),
            10 => Ok(ComponentId::Extra),
            11 => Ok(ComponentId::TemplateVertices),
            12 => Ok(ComponentId::TemplateGeometries),
            13 => Ok(ComponentId::Materials),
            14 => Ok(ComponentId::Textures),
            15 => Ok(ComponentId::VerticesTexture),
            255 => Ok(ComponentId::EndStream),
            _ => Err(crate::error::Error::Conversion(format!(
                "Invalid ComponentId byte: {}",
                value
            ))),
        }
    }
}

/// Writes the frame header (Component ID + Data Length) to the writer.
fn write_frame_header<W: Write>(writer: &mut W, id: ComponentId, length: u64) -> Result<()> {
    writer.write_all(&[id as u8])?;
    writer.write_all(&length.to_be_bytes())?; // Use big-endian for length
    Ok(())
}

/// Serializes an optional RecordBatch using Arrow IPC streaming format
/// into an in-memory buffer, then writes it to the stream with framing.
fn write_batch_component<W: Write>(
    writer: &mut W,
    id: ComponentId,
    batch_opt: &Option<RecordBatch>,
    options: &IpcWriteOptions,
) -> Result<()> {
    if let Some(batch) = batch_opt {
        // Skip empty batches as they add little value but require framing/schema
        if batch.num_rows() == 0 || batch.schema().fields().is_empty() {
            // log::debug!("Skipping empty batch for component {:?}", id);
            return Ok(());
        }

        // log::debug!("Writing component {:?} with {} rows", id, batch.num_rows());
        let mut buffer: Vec<u8> = Vec::new(); // In-memory buffer
        {
            let mut cursor = Cursor::new(&mut buffer);
            // Create a new writer specifically for this batch and its schema
            let mut ipc_writer = StreamWriter::try_new_with_options(
                &mut cursor,
                batch.schema().as_ref(),
                options.clone(),
            )?;
            ipc_writer.write(batch)?;
            ipc_writer.finish()?; // Finish the IPC stream within the buffer
        }

        // Write frame header with the length of the serialized IPC data
        write_frame_header(writer, id, buffer.len() as u64)?;
        // Write the actual IPC data
        writer.write_all(&buffer)?;
        // log::debug!("Wrote {} bytes for component {:?}", buffer.len(), id);
    } else {
        // log::debug!("Skipping None batch for component {:?}", id);
    }
    Ok(())
}

/// Writes simple string data to the stream with framing.
fn write_string_component<W: Write>(writer: &mut W, id: ComponentId, data: &str) -> Result<()> {
    //log::debug!("Writing string component {:?}", id);
    let bytes = data.as_bytes();
    write_frame_header(writer, id, bytes.len() as u64)?;
    writer.write_all(bytes)?;
    //log::debug!("Wrote {} bytes for component {:?}", bytes.len(), id);
    Ok(())
}

/// Writes the CityModelArrowParts components sequentially to a stream
/// (e.g., socket, pipe) using a framing protocol:
/// [Component ID (u8)] [Data Length (u64 BE)] [Data Bytes]
///
/// The Data Bytes for RecordBatch components are serialized using the
/// Arrow IPC streaming format.
///
/// # Arguments
/// * `parts` - The CityModelArrowParts to write.
/// * `writer` - The output stream implementing `std::io::Write`.
/// * `options` - Arrow IPC write options (e.g., for dictionary handling).
pub fn write_parts_to_stream<W: Write>(
    parts: &CityModelArrowParts,
    writer: &mut W,
    options: &IpcWriteOptions,
) -> Result<()> {
    //log::info!("Starting to write CityModelArrowParts to stream...");

    // --- Write Top-Level Metadata ---
    write_string_component(
        writer,
        ComponentId::TypeCityModel,
        &parts.type_citymodel.to_string(),
    )?;
    if let Some(version) = parts.version {
        write_string_component(writer, ComponentId::Version, version.to_string().as_str())?;
    }

    // --- Write RecordBatch Components ---
    // The order should ideally match the order defined in ComponentId for consistency
    write_batch_component(writer, ComponentId::Metadata, &parts.metadata, options)?;
    write_batch_component(writer, ComponentId::Transform, &parts.transform, options)?;
    write_batch_component(writer, ComponentId::Vertices, &parts.vertices, options)?;
    write_batch_component(writer, ComponentId::Geometries, &parts.geometries, options)?;
    write_batch_component(writer, ComponentId::Semantics, &parts.semantics, options)?;
    write_batch_component(
        writer,
        ComponentId::CityObjects,
        &parts.cityobjects,
        options,
    )?;
    write_batch_component(writer, ComponentId::Extensions, &parts.extensions, options)?;
    write_batch_component(writer, ComponentId::Extra, &parts.extra, options)?;
    write_batch_component(
        writer,
        ComponentId::TemplateVertices,
        &parts.template_vertices,
        options,
    )?;
    write_batch_component(
        writer,
        ComponentId::TemplateGeometries,
        &parts.template_geometries,
        options,
    )?;
    write_batch_component(writer, ComponentId::Materials, &parts.materials, options)?;
    write_batch_component(writer, ComponentId::Textures, &parts.textures, options)?;
    write_batch_component(
        writer,
        ComponentId::VerticesTexture,
        &parts.vertices_texture,
        options,
    )?;
    // Add calls for any other parts if the struct evolves

    // --- Write End Marker ---
    //log::info!("Writing end stream marker...");
    write_frame_header(writer, ComponentId::EndStream, 0)?; // Length 0 for end marker

    writer.flush()?; // Ensure all buffered data is sent
    //log::info!("Finished writing CityModelArrowParts to stream.");

    Ok(())
}

// --------------------- parquet

/// Write CityModelArrowParts to a directory with separate Parquet files for each component
///
/// This function creates a directory structure and writes each component of the CityModelArrowParts
/// as a separate Parquet file. It also generates a manifest.json file that describes the contents
/// and structure of the files.
///
/// # Parameters
///
/// * `parts` - The CityModelArrowParts structure containing Arrow data components
/// * `dir_path` - The directory path where the files should be written
/// * `compression` - Optional compression type to use (defaults to SNAPPY)
///
/// # Returns
///
/// `Result<()>` - Ok(()) if successful, or an Error if writing fails
pub fn write_to_parquet_directory<P: AsRef<Path>>(
    parts: &CityModelArrowParts,
    dir_path: P,
    compression: Option<Compression>,
) -> Result<()> {
    let dir_path = dir_path.as_ref();

    // Create directory if it doesn't exist
    fs::create_dir_all(dir_path)?;

    // Write manifest file with format type, type_citymodel, version, and component filenames
    let manifest = FileManifest {
        format: "parquet".to_string(),
        type_citymodel: format!("{:?}", parts.type_citymodel),
        version: parts.version.map(|v| format!("{}", v)),
        components: FileComponents {
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

    // Configure Parquet writer properties
    let props = WriterProperties::builder()
        .set_compression(compression.unwrap_or(Compression::SNAPPY))
        .build();

    // Helper function to write a RecordBatch to a Parquet file
    let write_batch = |batch: &RecordBatch, name: &str| -> Result<()> {
        let file_path = dir_path.join(format!("{}.parquet", name));
        let file = File::create(file_path)?;
        let mut writer = ArrowWriter::try_new(file, batch.schema(), Some(props.clone()))?;
        writer.write(batch)?;
        writer.close()?;
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

/// Convenience function to write a CityModel directly to Parquet files
///
/// This function takes a CityModel, converts it to CityModelArrowParts using
/// the `citymodel_to_arrow_parts` function, and then writes it to a directory
/// in Parquet format.
///
/// # Parameters
///
/// * `model` - The CityModel to write
/// * `dir_path` - The directory path where the files should be written
/// * `compression` - Optional compression type to use (defaults to SNAPPY)
///
/// # Returns
///
/// `Result<()>` - Ok(()) if successful, or an Error if writing fails
pub fn write_citymodel_to_parquet_directory<SS>(
    model: &cityjson::v2_0::CityModel<u32, ResourceId32, SS>,
    dir_path: impl AsRef<Path>,
    compression: Option<Compression>,
) -> Result<()>
where
    SS: StringStorage + Default,
    SS::String: AsRef<str> + Eq + Hash + Clone + Debug + Default + Display,
{
    let parts = crate::citymodel_to_arrow_parts(model)?;
    write_to_parquet_directory(&parts, dir_path, compression)
}

// ---------------------

#[cfg(test)]
mod tests {
    use super::Result;
    use super::*;
    use crate::citymodel_to_arrow_parts;
    use cityjson::prelude::*;
    use cityjson::v2_0::*;
    use tempfile::tempdir;

    #[test]
    fn test_write_empty_model() -> Result<()> {
        let model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
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
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

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

    use arrow::ipc::reader::StreamReader;
    use std::collections::HashMap;
    use std::io::{Cursor, Read};

    // Helper function to read a frame from a reader
    fn read_frame<R: Read>(reader: &mut R) -> Result<(ComponentId, Vec<u8>)> {
        let mut id_buf = [0u8; 1];
        reader.read_exact(&mut id_buf)?;
        let id = ComponentId::try_from(id_buf[0])?;

        let mut len_buf = [0u8; 8];
        reader.read_exact(&mut len_buf)?;
        let len = u64::from_be_bytes(len_buf);

        let mut data_buf = vec![0u8; len as usize];
        if len > 0 {
            reader.read_exact(&mut data_buf)?;
        }

        Ok((id, data_buf))
    }

    #[test]
    fn test_write_and_read_stream() -> Result<()> {
        // 1. Create some test data
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        model.metadata_mut().set_title("Stream Test");
        model.add_vertex(QuantizedCoordinate::new(10, 20, 30))?;
        let mut obj = CityObject::new("obj-stream".to_string(), CityObjectType::Building);
        obj.attributes_mut().insert(
            "status".to_string(),
            AttributeValue::String("ok".to_string()),
        );
        model.cityobjects_mut().add(obj);

        let parts = citymodel_to_arrow_parts(&model)?;

        // 2. Write to an in-memory buffer (simulating a socket)
        let mut output_buffer: Vec<u8> = Vec::new();
        let options = IpcWriteOptions::default();
        write_parts_to_stream(&parts, &mut output_buffer, &options)?;

        assert!(!output_buffer.is_empty());

        // 3. Read back and verify
        let mut input_cursor = Cursor::new(output_buffer);
        let mut received_parts: HashMap<ComponentId, Option<RecordBatch>> = HashMap::new();
        let mut received_type: Option<String> = None;
        let mut received_version: Option<String> = None;

        loop {
            let (id, data) = read_frame(&mut input_cursor)?;
            //log::debug!("Read frame: ID={:?}, Length={}", id, data.len());

            match id {
                ComponentId::EndStream => {
                    assert_eq!(data.len(), 0);
                    //log::info!("EndStream marker received.");
                    break;
                }
                ComponentId::TypeCityModel => {
                    received_type =
                        Some(String::from_utf8(data).expect("Invalid UTF8 for TypeCityModel"));
                }
                ComponentId::Version => {
                    received_version =
                        Some(String::from_utf8(data).expect("Invalid UTF8 for Version"));
                }
                // Handle RecordBatch components
                ComponentId::Metadata
                | ComponentId::Transform
                | ComponentId::Vertices
                | ComponentId::Geometries
                | ComponentId::Semantics
                | ComponentId::CityObjects
                | ComponentId::Extensions
                | ComponentId::Extra
                | ComponentId::TemplateVertices
                | ComponentId::TemplateGeometries
                | ComponentId::Materials
                | ComponentId::Textures
                | ComponentId::VerticesTexture => {
                    let mut data_cursor = Cursor::new(data);
                    // Use StreamReader as we wrote with StreamWriter
                    let mut reader = StreamReader::try_new(&mut data_cursor, None)?;
                    // Expect exactly one batch per component stream
                    if let Some(batch_result) = reader.next() {
                        let batch = batch_result?;
                        received_parts.insert(id, Some(batch));
                    } else {
                        panic!("Expected a RecordBatch for component {:?}", id);
                    }
                    // Ensure no more batches in this component's stream
                    assert!(
                        reader.next().is_none(),
                        "More than one batch found for component {:?}",
                        id
                    );
                }
            }
        }

        // 4. Assertions
        assert_eq!(received_type.as_deref(), Some("CityJSON"));
        assert_eq!(received_version.as_deref(), Some("2.0"));

        // Check received batches (presence and basic row count)
        assert!(received_parts.contains_key(&ComponentId::Metadata));
        assert_eq!(
            received_parts[&ComponentId::Metadata]
                .as_ref()
                .unwrap()
                .num_rows(),
            1
        );

        assert!(received_parts.contains_key(&ComponentId::Vertices));
        assert_eq!(
            received_parts[&ComponentId::Vertices]
                .as_ref()
                .unwrap()
                .num_rows(),
            1
        );

        assert!(received_parts.contains_key(&ComponentId::CityObjects));
        assert_eq!(
            received_parts[&ComponentId::CityObjects]
                .as_ref()
                .unwrap()
                .num_rows(),
            1
        );

        // Check that components not present in the original `parts` were not received
        assert!(!received_parts.contains_key(&ComponentId::Transform));
        assert!(!received_parts.contains_key(&ComponentId::Geometries));
        // ... add checks for other potentially absent components

        Ok(())
    }
}

#[cfg(test)]
mod tests_parquet {
    use super::*;
    use cityjson::prelude::*;
    use cityjson::v2_0::*;
    use parquet::basic::Compression;
    use tempfile::tempdir;

    #[test]
    fn test_write_empty_model_parquet() -> crate::error::Result<()> {
        let model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        let parts = crate::citymodel_to_arrow_parts(&model)?;

        let temp_dir = tempdir()?;
        let output_dir = temp_dir.path().join("empty_model_parquet");

        write_to_parquet_directory(&parts, &output_dir, Some(Compression::SNAPPY))?;

        // Check that the manifest file exists
        let manifest_path = output_dir.join("manifest.json");
        assert!(manifest_path.exists());

        Ok(())
    }

    #[test]
    fn test_write_model_with_data_parquet() -> crate::error::Result<()> {
        // Create a simple model with some data
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // Add a vertex
        model.add_vertex(QuantizedCoordinate::new(10, 20, 30))?;

        // Convert to parts
        let parts = crate::citymodel_to_arrow_parts(&model)?;

        let temp_dir = tempdir()?;
        let output_dir = temp_dir.path().join("model_with_data_parquet");

        write_to_parquet_directory(&parts, &output_dir, Some(Compression::ZSTD(parquet::basic::ZstdLevel::try_new(1).unwrap())))?;

        // Check that vertices file exists
        let vertices_path = output_dir.join("vertices.parquet");
        assert!(vertices_path.exists());

        Ok(())
    }

    #[test]
    fn test_write_citymodel_to_parquet_directory() -> crate::error::Result<()> {
        // Create a model with metadata and a vertex
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        model.metadata_mut().set_title("Test Parquet Model");
        model.add_vertex(QuantizedCoordinate::new(10, 20, 30))?;

        let temp_dir = tempdir()?;
        let output_dir = temp_dir.path().join("direct_model_parquet");

        write_citymodel_to_parquet_directory(&model, &output_dir, None)?;

        // Check that manifest and expected files exist
        let manifest_path = output_dir.join("manifest.json");
        let metadata_path = output_dir.join("metadata.parquet");
        let vertices_path = output_dir.join("vertices.parquet");

        assert!(manifest_path.exists());
        assert!(metadata_path.exists());
        assert!(vertices_path.exists());

        // Read the manifest to check format
        let manifest_json = std::fs::read_to_string(manifest_path)?;
        let manifest: FileManifest = DeJson::deserialize_json(&manifest_json)
            .map_err(|e| crate::error::Error::Conversion(format!("Failed to parse manifest: {}", e)))?;

        assert_eq!(manifest.format, "parquet");
        assert!(manifest.components.metadata);
        assert!(manifest.components.vertices);

        Ok(())
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use cityjson::prelude::*;
    use cityjson::v2_0::*;
    use parquet::basic::Compression;
    use tempfile::tempdir;

    #[test]
    fn test_compare_arrow_and_parquet_output() -> crate::error::Result<()> {
        // Create a model with various components
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // Add metadata
        // model.metadata_mut().set_title("Multi-format Test Model");

        // Add vertices
        model.add_vertex(QuantizedCoordinate::new(10, 20, 30))?;
        model.add_vertex(QuantizedCoordinate::new(40, 50, 60))?;

        // Add a city object
        let mut building = CityObject::new("building-1".to_string(), CityObjectType::Building);
        // TODO: https://github.com/apache/arrow-rs/issues/73
        // building.attributes_mut().insert("height".to_string(), AttributeValue::Float(42.0));
        model.cityobjects_mut().add(building);

        // TODO: https://github.com/apache/arrow-rs/issues/73
        // // Add extra properties
        // model.extra_mut().insert(
        //     "testProperty".to_string(),
        //     AttributeValue::String("Test Value".to_string()),
        // );

        // Set transform
        model.transform_mut().set_scale([0.001, 0.001, 0.001]);
        model.transform_mut().set_translate([1000.0, 2000.0, 0.0]);

        // Convert to parts
        let parts = crate::citymodel_to_arrow_parts(&model)?;

        let temp_dir = tempdir()?;
        let arrow_dir = temp_dir.path().join("arrow_output");
        let parquet_dir = temp_dir.path().join("parquet_output");

        // Write both formats
        write_to_directory(&parts, &arrow_dir)?;
        write_to_parquet_directory(&parts, &parquet_dir, Some(Compression::SNAPPY))?;

        // Verify that equivalent files exist in both directories
        let check_files = |name: &str| -> crate::error::Result<()> {
            let arrow_path = arrow_dir.join(format!("{}.arrow", name));
            let parquet_path = parquet_dir.join(format!("{}.parquet", name));

            assert!(arrow_path.exists(), "Arrow file {} does not exist", name);
            assert!(parquet_path.exists(), "Parquet file {} does not exist", name);

            Ok(())
        };

        // Check manifest files
        assert!(arrow_dir.join("manifest.json").exists());
        assert!(parquet_dir.join("manifest.json").exists());

        // Check component files
        // check_files("metadata")?;
        check_files("vertices")?;
        check_files("cityobjects")?;
        check_files("transform")?;
        // check_files("extra")?;

        // Read and verify the parquet manifest format
        let parquet_manifest_path = parquet_dir.join("manifest.json");
        let manifest_json = std::fs::read_to_string(parquet_manifest_path)?;

        if let Ok(file_manifest) = FileManifest::deserialize_json(&manifest_json) {
            // New format manifest
            assert_eq!(file_manifest.format, "parquet");
            // assert!(file_manifest.components.metadata);
            assert!(file_manifest.components.vertices);
            assert!(file_manifest.components.cityobjects);
            assert!(file_manifest.components.transform);
            // assert!(file_manifest.components.extra);
        } else {
            panic!("Could not parse manifest file");
        }

        Ok(())
    }

    #[test]
    fn test_citymodel_direct_writers() -> crate::error::Result<()> {
        // Create a model
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // Add a vertex and metadata
        model.add_vertex(QuantizedCoordinate::new(10, 20, 30))?;
        model.metadata_mut().set_title("Direct Writer Test");

        let temp_dir = tempdir()?;
        let arrow_dir = temp_dir.path().join("direct_arrow");
        let parquet_dir = temp_dir.path().join("direct_parquet");

        // Use the direct writer functions
        write_citymodel_to_directory(&model, &arrow_dir)?;
        write_citymodel_to_parquet_directory(&model, &parquet_dir, None)?;

        // Verify files exist
        assert!(arrow_dir.join("vertices.arrow").exists());
        assert!(arrow_dir.join("metadata.arrow").exists());
        assert!(arrow_dir.join("manifest.json").exists());

        assert!(parquet_dir.join("vertices.parquet").exists());
        assert!(parquet_dir.join("metadata.parquet").exists());
        assert!(parquet_dir.join("manifest.json").exists());

        Ok(())
    }
}