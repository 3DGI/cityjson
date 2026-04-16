use std::path::Path;

use crate::{CityModel, Error, Result};

/// Decode a `CityModel` from a persistent cityparquet package file.
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<CityModel> {
    cityjson_parquet::PackageReader
        .read_file(path)
        .map(CityModel::from)
        .map_err(Error::from)
}

/// Encode a `CityModel` as a persistent cityparquet package file.
pub fn to_file<P: AsRef<Path>>(path: P, model: &CityModel) -> Result<()> {
    cityjson_parquet::PackageWriter
        .write_file(path, model)
        .map(|_| ())
        .map_err(Error::from)
}
