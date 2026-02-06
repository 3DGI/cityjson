//! Version detection for CityJSON documents.

use crate::CityJSONVersion;
use crate::error::{Error, Result};

/// Detects the CityJSON version from a JSON object.
///
/// # Arguments
///
/// * `json_str` - A JSON string containing a CityJSON document
///
/// # Returns
///
/// The detected `CityJSONVersion`.
///
/// # Errors
///
/// - `Error::InvalidJson` if the JSON is malformed
/// - `Error::MissingVersion` if the "version" field is missing
/// - `Error::UnsupportedCityJSONVersion` if the version is not recognized
pub fn detect_version(json_str: &str) -> Result<CityJSONVersion> {
    let value: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| Error::InvalidJson(e.to_string()))?;

    let version_str = value
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or(Error::MissingVersion)?;

    match version_str {
        "1.0" => Ok(CityJSONVersion::V1_0),
        "1.1" => Ok(CityJSONVersion::V1_1),
        "2.0" => Ok(CityJSONVersion::V2_0),
        other => Err(Error::UnsupportedCityJSONVersion(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_version_1_0() {
        let json = r#"{"version": "1.0"}"#;
        assert_eq!(detect_version(json).unwrap(), CityJSONVersion::V1_0);
    }

    #[test]
    fn test_detect_version_1_1() {
        let json = r#"{"version": "1.1"}"#;
        assert_eq!(detect_version(json).unwrap(), CityJSONVersion::V1_1);
    }

    #[test]
    fn test_detect_version_2_0() {
        let json = r#"{"version": "2.0"}"#;
        assert_eq!(detect_version(json).unwrap(), CityJSONVersion::V2_0);
    }

    #[test]
    fn test_detect_version_missing() {
        let json = r#"{"type": "CityJSON"}"#;
        assert!(matches!(detect_version(json), Err(Error::MissingVersion)));
    }

    #[test]
    fn test_detect_version_unsupported() {
        let json = r#"{"version": "3.0"}"#;
        assert!(matches!(
            detect_version(json),
            Err(Error::UnsupportedCityJSONVersion(_))
        ));
    }
}
