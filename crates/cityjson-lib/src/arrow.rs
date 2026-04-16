use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Write};
use std::path::Path;

use arrow::record_batch::RecordBatch;

use crate::{CityModel, Error, Result};

pub use cityjson_arrow::transport::CanonicalTable;
pub use cityjson_arrow::{
    CityArrowHeader, ExportOptions, ImportOptions, ProjectionLayout, SchemaVersion, WriteReport,
};

#[derive(Debug, Clone)]
pub struct ArrowBatches {
    header: CityArrowHeader,
    projection: ProjectionLayout,
    batches: Vec<(CanonicalTable, RecordBatch)>,
}

impl ArrowBatches {
    #[must_use]
    pub const fn header(&self) -> &CityArrowHeader {
        &self.header
    }

    #[must_use]
    pub const fn projection(&self) -> &ProjectionLayout {
        &self.projection
    }

    #[must_use]
    pub fn batches(&self) -> &[(CanonicalTable, RecordBatch)] {
        &self.batches
    }
}

pub fn from_bytes(bytes: &[u8]) -> Result<CityModel> {
    from_reader(Cursor::new(bytes))
}

pub fn to_vec(model: &CityModel) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let _ = to_writer(&mut bytes, model)?;
    Ok(bytes)
}

pub fn export_batches(model: &CityModel) -> Result<ArrowBatches> {
    export_batches_with_options(model, &ExportOptions::default())
}

pub fn export_batches_with_options(
    model: &CityModel,
    options: &ExportOptions,
) -> Result<ArrowBatches> {
    let reader = cityjson_arrow::export_reader(model.as_inner(), options).map_err(Error::from)?;
    Ok(ArrowBatches {
        header: reader.header().clone(),
        projection: reader.projection().clone(),
        batches: reader.collect(),
    })
}

pub fn import_batches(batches: &ArrowBatches) -> Result<CityModel> {
    import_batches_with_options(batches, &ImportOptions::default())
}

pub fn import_batches_with_options(
    batches: &ArrowBatches,
    options: &ImportOptions,
) -> Result<CityModel> {
    cityjson_arrow::import_batches(
        batches.header.clone(),
        batches.projection.clone(),
        batches.batches.clone(),
        options,
    )
    .map(CityModel::from)
    .map_err(Error::from)
}

pub fn from_reader(reader: impl std::io::Read) -> Result<CityModel> {
    cityjson_arrow::read_stream(reader, &ImportOptions::default())
        .map(CityModel::from)
        .map_err(Error::from)
}

pub fn to_writer(writer: &mut impl Write, model: &CityModel) -> Result<WriteReport> {
    cityjson_arrow::write_stream(writer, model.as_inner(), &ExportOptions::default())
        .map_err(Error::from)
}

pub fn from_file(path: impl AsRef<Path>) -> Result<CityModel> {
    let file = File::open(path)?;
    from_reader(BufReader::new(file))
}

pub fn to_file(path: impl AsRef<Path>, model: &CityModel) -> Result<WriteReport> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    let report = to_writer(&mut writer, model)?;
    writer.flush()?;
    Ok(report)
}
