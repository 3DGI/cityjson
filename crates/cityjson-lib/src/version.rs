use std::fmt::{Display, Formatter};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CityJSONVersion {
    V1_0,
    V1_1,
    V2_0,
}

impl CityJSONVersion {
    pub(crate) fn supported_versions() -> &'static str {
        "1.0, 1.0.0, 1.0.1, 1.0.2, 1.0.3, 1.1, 1.1.0, 1.1.1, 1.1.2, 1.1.3, 2.0, 2.0.0, 2.0.1"
    }
}

impl Default for CityJSONVersion {
    fn default() -> Self {
        Self::V2_0
    }
}

impl Display for CityJSONVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1_0 => write!(f, "1.0"),
            Self::V1_1 => write!(f, "1.1"),
            Self::V2_0 => write!(f, "2.0"),
        }
    }
}

impl TryFrom<&str> for CityJSONVersion {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "1.0" | "1.0.0" | "1.0.1" | "1.0.2" | "1.0.3" => Ok(Self::V1_0),
            "1.1" | "1.1.0" | "1.1.1" | "1.1.2" | "1.1.3" => Ok(Self::V1_1),
            "2.0" | "2.0.0" | "2.0.1" => Ok(Self::V2_0),
            other => Err(Error::UnsupportedVersion {
                found: other.to_string(),
                supported: Self::supported_versions().to_string(),
            }),
        }
    }
}

impl TryFrom<String> for CityJSONVersion {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Self::try_from(value.as_str())
    }
}
