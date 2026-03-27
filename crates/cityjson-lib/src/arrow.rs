use std::path::Path;

use crate::{CityModel, Result};

/// Illustrative placeholder for a future explicit Arrow format boundary.
pub fn to_file<P: AsRef<Path>>(_path: P, _model: &CityModel) -> Result<()> {
    todo!("implement the Arrow format boundary in a dedicated backend crate")
}
