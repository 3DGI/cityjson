use crate::{CityModel, Result};

/// Illustrative placeholder for higher-level workflows that sit above the core model.
pub fn merge<I>(_models: I) -> Result<CityModel>
where
    I: IntoIterator<Item = CityModel>,
{
    todo!("implement model-authoritative merge delegation through cityjson-rs")
}
