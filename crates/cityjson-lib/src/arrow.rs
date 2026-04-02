use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use crate::{CityModel, Error, Result};

/// Decode a `CityModel` from a live Arrow IPC stream.
pub fn from_reader<R: Read>(reader: R) -> Result<CityModel> {
    cityarrow::ModelDecoder
        .decode(reader)
        .map(CityModel::from)
        .map_err(Error::from)
}

/// Encode a `CityModel` as a live Arrow IPC stream.
pub fn to_writer(writer: &mut impl Write, model: &CityModel) -> Result<()> {
    cityarrow::ModelEncoder
        .encode(model.as_inner(), writer)
        .map_err(Error::from)
}

/// Decode a `CityModel` from a live Arrow IPC stream file.
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<CityModel> {
    let file = File::open(path)?;
    from_reader(BufReader::new(file))
}

/// Encode a `CityModel` as a live Arrow IPC stream file.
pub fn to_file<P: AsRef<Path>>(path: P, model: &CityModel) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    to_writer(&mut writer, model)?;
    writer.flush()?;
    Ok(())
}
