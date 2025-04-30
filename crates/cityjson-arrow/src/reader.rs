//! Read Arrow data into a [cityjson::v2_0::CityModel] object.

// In src/reader.rs

use crate::writer::FileManifest;
// Reuse the manifest struct from writer
use crate::{
    error::{Error, Result},
    CityModelArrowParts,
};
use arrow::buffer::Buffer;
use arrow::ipc::convert::fb_to_schema;
use arrow::ipc::reader::{read_footer_length, FileDecoder};
use arrow::ipc::{root_as_footer, Block};
use arrow::record_batch::RecordBatch;
use cityjson::{CityJSONVersion, CityModelType};
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

/// Incrementally decodes [`RecordBatch`]es from an IPC file stored in a Arrow
/// [`Buffer`] using the [`FileDecoder`] API.
///
/// This is a wrapper around the example in the `FileDecoder` which handles the
/// low level interaction with the Arrow IPC format.
struct IPCBufferDecoder {
    /// Memory (or memory mapped) Buffer with the data
    buffer: Buffer,
    /// Decoder that reads Arrays that refers to the underlying buffers
    decoder: FileDecoder,
    /// Location of the batches within the buffer
    batches: Vec<Block>,
}

impl IPCBufferDecoder {
    fn new(buffer: Buffer) -> Self {
        let trailer_start = buffer.len() - 10;
        let footer_len = read_footer_length(buffer[trailer_start..].try_into().unwrap()).unwrap();
        let footer = root_as_footer(&buffer[trailer_start - footer_len..trailer_start]).unwrap();

        let schema = fb_to_schema(footer.schema().unwrap());

        let mut decoder = FileDecoder::new(Arc::new(schema), footer.version());

        // Read dictionaries
        for block in footer.dictionaries().iter().flatten() {
            let block_len = block.bodyLength() as usize + block.metaDataLength() as usize;
            let data = buffer.slice_with_length(block.offset() as _, block_len);
            decoder.read_dictionary(block, &data).unwrap();
        }

        // convert to Vec from the flatbuffers Vector to avoid having a direct dependency on flatbuffers
        let batches = footer
            .recordBatches()
            .map(|b| b.iter().copied().collect())
            .unwrap_or_default();

        Self {
            buffer,
            decoder,
            batches,
        }
    }

    /// Return the number of [`RecordBatch`]es in this buffer
    fn num_batches(&self) -> usize {
        self.batches.len()
    }

    /// Return the [`RecordBatch`] at message index `i`.
    ///
    /// This may return `None` if the IPC message was None
    fn get_batch(&self, i: usize) -> arrow::error::Result<Option<RecordBatch>> {
        let block = &self.batches[i];
        let block_len = block.bodyLength() as usize + block.metaDataLength() as usize;
        let data = self
            .buffer
            .slice_with_length(block.offset() as _, block_len);
        self.decoder.read_record_batch(block, &data)
    }
}

/// Reads CityModelArrowParts from a directory containing component Arrow IPC files and a manifest.
///
/// This function uses memory mapping for efficient, potentially zero-copy reads of the Arrow files.
///
/// # Arguments
/// * `dir_path` - The path to the directory created by `write_to_directory`.
///
/// # Returns
/// A `Result` containing the populated `CityModelArrowParts`.
pub fn read_from_directory<P: AsRef<Path>>(dir_path: P) -> Result<CityModelArrowParts> {
    let dir_path = dir_path.as_ref();

    // 1. Read and parse the manifest
    let manifest_path = dir_path.join("manifest.json");
    let manifest_json = std::fs::read_to_string(manifest_path).map_err(|e| Error::Io(e))?; // Add Io variant to your Error enum
    let manifest: FileManifest = nanoserde::DeJson::deserialize_json(&manifest_json)
        .map_err(|e| Error::Conversion(format!("Failed to parse manifest: {}", e)))?; // Add/use Conversion variant

    // Helper function to read a single component file
    let read_component = |name: &str| -> Result<Option<RecordBatch>> {
        let file_path = dir_path.join(format!("{}.arrow", name));
        if !file_path.exists() {
            return Ok(None);
        }

        let ipc_file = File::open(&file_path).map_err(Error::Io)?;

        // Memory map the file
        // SAFETY: Assuming the file is a valid Arrow IPC file.
        let mmap = unsafe { memmap2::Mmap::map(&ipc_file)? };

        // Convert the mmap region to an Arrow `Buffer` to back the arrow arrays. We
        // do this by first creating a `bytes::Bytes` (which is zero copy) and then
        // creating a Buffer from the `Bytes` (which is also zero copy)
        let bytes = bytes::Bytes::from_owner(mmap);
        let buffer = Buffer::from(bytes);

        // Now, use the FileDecoder API (wrapped by `IPCBufferDecoder` for
        // convenience) to crate Arrays re-using the data in the underlying buffer
        let decoder = IPCBufferDecoder::new(buffer);

        // Assuming one RecordBatch per component file, which is how write_to_directory works
        if decoder.num_batches() != 1 {
            return Err(Error::Conversion(format!(
                "Expected 1 RecordBatch in {}, found {}",
                file_path.display(),
                decoder.num_batches()
            )));
        }

        // Read the RecordBatch - this is where zero-copy happens for buffers
        Ok(decoder.get_batch(0)?)
    };

    // 2. Read components based on the manifest
    Ok(CityModelArrowParts {
        type_citymodel: CityModelType::try_from(manifest.type_citymodel.as_str())?,
        version: manifest
            .version
            .map(|v| CityJSONVersion::try_from(v.as_str()))
            .transpose()?,

        extensions: if manifest.components.extensions {
            read_component("extensions")?
        } else {
            None
        },
        extra: if manifest.components.extra {
            read_component("extra")?
        } else {
            None
        },
        metadata: if manifest.components.metadata {
            read_component("metadata")?
        } else {
            None
        },
        cityobjects: if manifest.components.cityobjects {
            read_component("cityobjects")?
        } else {
            None
        },
        transform: if manifest.components.transform {
            read_component("transform")?
        } else {
            None
        },
        vertices: if manifest.components.vertices {
            read_component("vertices")?
        } else {
            None
        },
        geometries: if manifest.components.geometries {
            read_component("geometries")?
        } else {
            None
        },
        template_vertices: if manifest.components.template_vertices {
            read_component("template_vertices")?
        } else {
            None
        },
        template_geometries: if manifest.components.template_geometries {
            read_component("template_geometries")?
        } else {
            None
        },
        semantics: if manifest.components.semantics {
            read_component("semantics")?
        } else {
            None
        },
        materials: if manifest.components.materials {
            read_component("materials")?
        } else {
            None
        },
        textures: if manifest.components.textures {
            read_component("textures")?
        } else {
            None
        },
        vertices_texture: if manifest.components.vertices_texture {
            read_component("vertices_texture")?
        } else {
            None
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::citymodel_to_arrow_parts;
    use crate::writer::write_to_directory;
    use arrow::array::StringArray;
    use cityjson::prelude::*;
    use cityjson::v2_0::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_read_from_directory() -> crate::error::Result<()> {
        // 1. Create a sample model with test data
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // Add metadata, vertices, and city objects
        model.metadata_mut().set_title("Test Model");
        model.add_vertex(QuantizedCoordinate::new(10, 20, 30))?;

        let mut building = CityObject::new("building-1".to_string(), CityObjectType::Building);
        building
            .attributes_mut()
            .insert("height".to_string(), AttributeValue::Float(25.5));
        model.cityobjects_mut().add(building);

        // 2. Convert model to Arrow parts
        let original_parts = citymodel_to_arrow_parts(&model)?;

        // 3. Write parts to temporary directory
        let temp_dir = tempdir()?;
        let dir_path = temp_dir.path();
        write_to_directory(&original_parts, dir_path)?;

        // 4. Read the parts back using the function under test
        let read_parts = read_from_directory(dir_path)?;

        // 5. Verify basic metadata matches
        assert_eq!(read_parts.type_citymodel, original_parts.type_citymodel);
        assert_eq!(read_parts.version, original_parts.version);

        // 6. Verify components that should exist
        assert!(read_parts.metadata.is_some());
        assert!(read_parts.vertices.is_some());
        assert!(read_parts.cityobjects.is_some());

        // 7. Check component content
        // Metadata should contain our title
        let metadata_batch = read_parts.metadata.as_ref().unwrap();
        assert_eq!(metadata_batch.num_rows(), 1);
        let title_col = metadata_batch.column_by_name("title").unwrap();
        let title_array = title_col.as_any().downcast_ref::<StringArray>().unwrap();
        assert_eq!(title_array.value(0), "Test Model");

        // Vertices should have one row
        assert_eq!(read_parts.vertices.as_ref().unwrap().num_rows(), 1);

        // City objects should have one row
        assert_eq!(read_parts.cityobjects.as_ref().unwrap().num_rows(), 1);

        // 8. Verify absent components are None
        assert!(read_parts.transform.is_none());
        assert!(read_parts.geometries.is_none());
        assert!(read_parts.template_vertices.is_none());
        assert!(read_parts.template_geometries.is_none());
        assert!(read_parts.semantics.is_none());
        assert!(read_parts.materials.is_none());
        assert!(read_parts.textures.is_none());
        assert!(read_parts.vertices_texture.is_none());

        // 9. Test partial reading - remove a component file
        let cityobjects_path = dir_path.join("cityobjects.arrow");
        fs::remove_file(&cityobjects_path)?;

        // 10. Read again and verify the component is now None
        let partial_parts = read_from_directory(dir_path)?;
        assert!(partial_parts.cityobjects.is_none());
        assert!(partial_parts.metadata.is_some()); // Other components still exist

        Ok(())
    }
}
