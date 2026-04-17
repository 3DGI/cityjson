use std::path::Path;

use crate::{CityModel, Error, Result};

/// Decode a `CityModel` from a persistent cityparquet package file.
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<CityModel> {
    let _ = path;
    Err(Error::UnsupportedFeature(
        "Parquet transport is not implemented in the first public release".into(),
    ))
}

/// Encode a `CityModel` as a persistent cityparquet package file.
pub fn to_file<P: AsRef<Path>>(path: P, model: &CityModel) -> Result<()> {
    let _ = (path, model);
    Err(Error::UnsupportedFeature(
        "Parquet transport is not implemented in the first public release".into(),
    ))
}
