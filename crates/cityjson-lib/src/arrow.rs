use std::path::Path;

use crate::{CityModel, Error, Result};

pub use cityarrow::{
    CityArrowHeader, CityArrowPackageVersion, CityModelArrowParts, PackageManifest,
    PackageTableEncoding, PackageTables, ProjectedFieldSpec, ProjectedValueType, ProjectionLayout,
    canonical_schema_set,
};

pub fn to_parts(model: &CityModel) -> Result<CityModelArrowParts> {
    cityarrow::to_parts(model.as_inner()).map_err(Error::from)
}

pub fn from_parts(parts: CityModelArrowParts) -> Result<CityModel> {
    cityarrow::from_parts(&parts)
        .map(CityModel::from)
        .map_err(Error::from)
}

/// Write an Arrow IPC package directory rooted at `path`.
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
    let parts = to_parts(model)?;
    cityarrow::write_package_ipc_dir(path, &parts).map_err(Error::from)
}

pub fn read_package<P: AsRef<Path>>(path: P) -> Result<CityModel> {
    read_package_dir(path)
}

pub fn read_package_dir<P: AsRef<Path>>(path: P) -> Result<CityModel> {
    let parts = cityarrow::read_package_ipc_dir(path).map_err(Error::from)?;
    from_parts(parts)
}
