use std::error;
use std::fmt::{Debug, Display, Formatter};

pub enum Error {
    Json(serde_json::Error),
    Utf8(std::str::Utf8Error),
    CityJson(cityjson::error::Error),
    UnsupportedType(String),
    UnsupportedVersion(String),
    MalformedRootObject(&'static str),
    InvalidValue(String),
    UnsupportedFeature(&'static str),
    UnresolvedCityObjectReference {
        source_id: String,
        target_id: String,
        relation: &'static str,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Json(err) => write!(f, "JSON error: {err}"),
            Error::Utf8(err) => write!(f, "UTF-8 error: {err}"),
            Error::CityJson(err) => write!(f, "cityjson error: {err}"),
            Error::UnsupportedType(kind) => {
                write!(f, "unsupported CityJSON root type: {kind}")
            }
            Error::UnsupportedVersion(version) => {
                write!(f, "unsupported CityJSON version: {version}")
            }
            Error::MalformedRootObject(reason) => write!(f, "malformed root object: {reason}"),
            Error::InvalidValue(reason) => write!(f, "invalid value: {reason}"),
            Error::UnsupportedFeature(feature) => {
                write!(
                    f,
                    "unsupported feature in current migration slice: {feature}"
                )
            }
            Error::UnresolvedCityObjectReference {
                source_id,
                target_id,
                relation,
            } => write!(
                f,
                "unresolved CityObject {relation} reference from '{source_id}' to '{target_id}'"
            ),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<cityjson::error::Error> for Error {
    fn from(error: cityjson::error::Error) -> Self {
        Self::CityJson(error)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Self {
        Self::Utf8(error)
    }
}

impl error::Error for Error {}
