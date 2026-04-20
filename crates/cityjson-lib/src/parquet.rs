use std::path::Path;

use crate::{CityModel, Error, Result};

pub use cityjson_parquet::{PackageManifest, ParquetDatasetManifest};

pub fn from_file(path: impl AsRef<Path>) -> Result<CityModel> {
    cityjson_parquet::PackageReader::default()
        .read_file(path)
        .map(CityModel::from)
        .map_err(Error::from)
}

pub fn to_file(path: impl AsRef<Path>, model: &CityModel) -> Result<PackageManifest> {
    cityjson_parquet::PackageWriter::default()
        .write_file(path, model)
        .map_err(Error::from)
}

pub fn from_dir(path: impl AsRef<Path>) -> Result<CityModel> {
    cityjson_parquet::ParquetDatasetReader::default()
        .read_dir(path)
        .map(CityModel::from)
        .map_err(Error::from)
}

pub fn to_dir(path: impl AsRef<Path>, model: &CityModel) -> Result<ParquetDatasetManifest> {
    cityjson_parquet::ParquetDatasetWriter::default()
        .write_dir(path, model)
        .map_err(Error::from)
}
