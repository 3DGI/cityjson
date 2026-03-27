use std::path::Path;

use crate::{CityModel, Result};

/// Illustrative placeholder for a future explicit Parquet format boundary.
pub fn to_file<P: AsRef<Path>>(_path: P, _model: &CityModel) -> Result<()> {
    todo!("implement the Parquet format boundary in a dedicated backend crate")
}
