use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json_borrow::Value;

use crate::errors::{Error, Result};

#[derive(Debug, Deserialize, Serialize)]
pub struct CityModel<'a> {
    pub version: Option<CityJSONVersion>,
    pub geometry: Option<Vec<Boundary>>,
    #[serde(borrow, deserialize_with = "deserialize_attributes")]
    pub attributes: Option<Attributes<'a>>,
}

type Attributes<'a> = Value<'a>;

pub fn deserialize_attributes<'de: 'a, 'a, D>(
    deserializer: D,
) -> std::result::Result<Option<Attributes<'a>>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s = Value::deserialize(deserializer)?;
    Ok((!s.is_null()).then_some(s))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Boundary {
    inner: Vec<usize>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
#[serde(tag = "version", try_from = "String", into = "String")]
pub enum CityJSONVersion {
    V2_0,
}

impl fmt::Display for CityJSONVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CityJSONVersion::V2_0 => {
                write!(f, "2.0")
            }
        }
    }
}

impl TryFrom<&str> for CityJSONVersion {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        CityJSONVersion::_from_str(value)
    }
}

impl TryFrom<String> for CityJSONVersion {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        CityJSONVersion::_from_str(value.as_ref())
    }
}

impl CityJSONVersion {
    fn _from_str(value: &str) -> Result<CityJSONVersion> {
        match value {
            "2.0" | "2.0.0" => Ok(CityJSONVersion::V2_0),
            _ => Err(Error::UnsupportedVersion(
                value.to_string(),
                "2.0, 2.0.0".to_string(),
            )),
        }
    }
}

/// This implementation is only used for serializing the CityJSON version, because serde cannot
/// serialize from 'try_into' (which is provided by the 'try_from' implementations).
/// So we need this Into, even though [std says that one should avoid implementing Into](https://doc.rust-lang.org/std/convert/trait.Into.html).
#[allow(clippy::from_over_into)]
impl Into<String> for CityJSONVersion {
    fn into(self) -> String {
        match self {
            CityJSONVersion::V2_0 => String::from("2.0"),
        }
    }
}
