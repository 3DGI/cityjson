use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Write};
use std::path::Path;

use crate::{CityModel, Error, Result};

pub use cityjson_arrow::{
    ExportOptions, ImportOptions, SchemaVersion, WriteReport, read_stream, write_stream,
};

pub fn from_bytes(bytes: &[u8]) -> Result<CityModel> {
    from_reader(Cursor::new(bytes))
}

pub fn to_vec(model: &CityModel) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let _ = to_writer(&mut bytes, model)?;
    Ok(bytes)
}

pub fn from_reader(reader: impl std::io::Read) -> Result<CityModel> {
    cityjson_arrow::read_stream(reader, &ImportOptions::default())
        .map(CityModel::from)
        .map_err(Error::from)
}

pub fn to_writer(writer: impl Write, model: &CityModel) -> Result<WriteReport> {
    cityjson_arrow::write_stream(writer, model, &ExportOptions::default()).map_err(Error::from)
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
