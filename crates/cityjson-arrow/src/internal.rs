use crate::convert;
use crate::error::Result;
use crate::schema::CityModelArrowParts;
use cityjson::v2_0::OwnedCityModel;

pub use crate::transport::{
    CanonicalTable, build_parts, collect_tables, concat_record_batches, schema_for_table,
    validate_schema,
};

/// Internal bridge for sibling crates that need canonical transport parts.
///
/// This is not part of the supported end-user API.
///
/// # Errors
///
/// Returns an error when canonical transport encoding fails.
pub fn encode_parts(model: &OwnedCityModel) -> Result<CityModelArrowParts> {
    convert::encode_parts(model)
}

/// Internal bridge for sibling crates that need canonical transport parts.
///
/// This is not part of the supported end-user API.
///
/// # Errors
///
/// Returns an error when canonical transport decoding fails.
pub fn decode_parts(parts: &CityModelArrowParts) -> Result<OwnedCityModel> {
    convert::decode_parts(parts)
}
