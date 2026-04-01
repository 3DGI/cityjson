use std::path::Path;

use crate::{CityModel, Error, Result};

pub use cityparquet::{
    CityArrowHeader, CityArrowPackageVersion, CityModelArrowParts, PackageManifest,
    PackageTableEncoding, PackageTables, ProjectedFieldSpec, ProjectedValueType, ProjectionLayout,
    canonical_schema_set,
};

/// Write a Parquet package directory rooted at `path`.
pub fn to_file<P: AsRef<Path>>(path: P, model: &CityModel) -> Result<()> {
    write_package_dir(path, model).map(|_| ())
}

pub fn from_file<P: AsRef<Path>>(path: P) -> Result<CityModel> {
    read_package_dir(path)
}

pub fn write_package<P: AsRef<Path>>(path: P, model: &CityModel) -> Result<PackageManifest> {
    write_package_dir(path, model)
}

pub fn write_package_dir<P: AsRef<Path>>(path: P, model: &CityModel) -> Result<PackageManifest> {
    let parts = cityparquet::to_parts(model.as_inner()).map_err(Error::from)?;
    cityparquet::write_package_dir(path, &parts).map_err(Error::from)
}

pub fn read_package<P: AsRef<Path>>(path: P) -> Result<CityModel> {
    read_package_dir(path)
}

pub fn read_package_dir<P: AsRef<Path>>(path: P) -> Result<CityModel> {
    let parts = cityparquet::read_package_dir(path).map_err(Error::from)?;
    cityparquet::from_parts(&parts)
        .map(CityModel::from)
        .map_err(Error::from)
}
