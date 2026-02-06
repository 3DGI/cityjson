//! CityJSON v1.1 to v2.0 conversion.
//!
//! ## Schema Differences (v1.1 → v2.0)
//!
//! v1.1 is structurally very similar to v2.0. Key differences:
//!
//! | v1.1 | v2.0 | Notes |
//! |------|------|-------|
//! | `BridgeConstructionElement` | `BridgeConstructiveElement` | Renamed |
//! | `TunnelConstructionElement` | `TunnelConstructiveElement` | Renamed |

use crate::error::Result;
use crate::prelude::*;
use crate::v2_0::CityModel;

/// Converts a CityJSON v1.1 document to v2.0.
///
/// Since v1.1 is structurally similar to v2.0, most conversion is straightforward.
pub fn convert_to_v2<SS: StringStorage>(json_str: &str) -> Result<CityModel<u32, SS>>
where
    SS::String: From<String>,
{
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| crate::error::Error::InvalidJson(e.to_string()))?;

    convert_from_value::<SS>(&value)
}

/// Converts from a parsed JSON value.
pub(crate) fn convert_from_value<SS: StringStorage>(
    value: &serde_json::Value,
) -> Result<CityModel<u32, SS>>
where
    SS::String: From<String>,
{
    // v1.1 is very similar to v2.0, so we reuse most of the v1.0 converter logic
    // with adjustments for v1.1-specific differences
    super::v1_0::convert_from_value(value)
}

#[cfg(test)]
mod tests {
    // TODO: Add v1.1-specific tests when needed
}
