use std::path::Path;

use crate::{CityModel, Error, Result};

/// Decode a `CityModel` from a persistent cityparquet package file.
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<CityModel> {
    cityparquet::PackageReader
        .read_file(path)
        .map(CityModel::from)
        .map_err(Error::from)
}

/// Encode a `CityModel` as a persistent cityparquet package file.
pub fn to_file<P: AsRef<Path>>(path: P, model: &CityModel) -> Result<()> {
    cityparquet::PackageWriter
        .write_file(path, model.as_inner())
        .map(|_| ())
        .map_err(Error::from)
}
