//! # Legacy CityJSON Import
//!
//! This module handles importing CityJSON files from older versions (1.0, 1.1)
//! and converting them to the current v2.0 data model.
//!
//! ## Usage
//!
//! ```rust
//! use cityjson::import::import_cityjson;
//! use cityjson::prelude::*;
//!
//! # fn main() -> cityjson::error::Result<()> {
//! let json = r#"{"type": "CityJSON", "version": "1.1", "CityObjects": {}, "vertices": []}"#;
//! let model: cityjson::v2_0::CityModel<u32, OwnedStringStorage> = import_cityjson(json)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Version Handling
//!
//! - **v1.0**: Converted to v2.0 with schema mapping
//! - **v1.1**: Converted to v2.0 with schema mapping
//! - **v2.0**: Parsed directly (no conversion needed)

mod v1_0;
mod v1_1;
mod version;

use crate::CityJSONVersion;
use crate::error::Result;
use crate::prelude::*;
use crate::v2_0::CityModel;

pub use version::detect_version;

/// Imports a CityJSON document from a JSON string.
///
/// This function:
/// 1. Detects the CityJSON version from the document
/// 2. Converts legacy versions (1.0, 1.1) to v2.0
/// 3. Returns a v2.0 CityModel
///
/// # Arguments
///
/// * `json` - A JSON string containing a CityJSON document
///
/// # Returns
///
/// A `v2_0::CityModel` regardless of the input version.
///
/// # Errors
///
/// - `Error::InvalidJson` if the JSON is malformed
/// - `Error::MissingVersion` if the "version" field is missing
/// - `Error::UnsupportedCityJSONVersion` if the version is not 1.0, 1.1, or 2.0
/// - `Error::Import` if conversion fails
///
/// # Example
///
/// ```rust
/// use cityjson::import::import_cityjson;
/// use cityjson::prelude::*;
///
/// # fn main() -> cityjson::error::Result<()> {
/// let json = r#"{"type": "CityJSON", "version": "2.0", "CityObjects": {}, "vertices": []}"#;
/// let model = import_cityjson::<OwnedStringStorage>(json)?;
/// # Ok(())
/// # }
/// ```
pub fn import_cityjson<SS: StringStorage>(json: &str) -> Result<CityModel<u32, SS>>
where
    SS::String: From<String>,
{
    let version = detect_version(json)?;

    match version {
        CityJSONVersion::V1_0 => v1_0::convert_to_v2(json),
        CityJSONVersion::V1_1 => v1_1::convert_to_v2(json),
        CityJSONVersion::V2_0 => convert_v2_native(json),
    }
}

/// Converts a v2.0 JSON document to CityModel.
///
/// Since v2.0 is the native format, we can use the same conversion logic as v1.1
/// (as they are structurally very similar).
fn convert_v2_native<SS: StringStorage>(json: &str) -> Result<CityModel<u32, SS>>
where
    SS::String: From<String>,
{
    // For v2.0, reuse v1.1 logic since v2.0 is structurally similar
    // TODO: Implement direct v2.0 parsing or serde deserialization
    v1_1::convert_to_v2(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_detects_versions() {
        let v1_0_json = r#"{"version": "1.0", "vertices": [], "CityObjects": {}}"#;
        let v1_1_json = r#"{"version": "1.1", "vertices": [], "CityObjects": {}}"#;
        let v2_0_json = r#"{"version": "2.0", "vertices": [], "CityObjects": {}}"#;

        assert_eq!(detect_version(v1_0_json).unwrap(), CityJSONVersion::V1_0);
        assert_eq!(detect_version(v1_1_json).unwrap(), CityJSONVersion::V1_1);
        assert_eq!(detect_version(v2_0_json).unwrap(), CityJSONVersion::V2_0);
    }

    #[test]
    fn test_import_v1_0_minimal() {
        let json = r#"{"version": "1.0", "vertices": [], "CityObjects": {}}"#;
        let result = import_cityjson::<OwnedStringStorage>(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_import_v1_1_minimal() {
        let json = r#"{"version": "1.1", "vertices": [], "CityObjects": {}}"#;
        let result = import_cityjson::<OwnedStringStorage>(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_import_v2_0_minimal() {
        let json = r#"{"version": "2.0", "vertices": [], "CityObjects": {}}"#;
        let result = import_cityjson::<OwnedStringStorage>(json);
        assert!(result.is_ok());
    }
}
