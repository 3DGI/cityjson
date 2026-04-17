use std::error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    Io,
    Syntax,
    Version,
    Shape,
    Unsupported,
    Model,
}

pub enum Error {
    Io(std::io::Error),
    Syntax(String),
    CityJSON(cityjson::error::Error),
    MissingVersion,
    ExpectedCityJSON(String),
    ExpectedCityJSONFeature(String),
    UnsupportedType(String),
    UnsupportedVersion { found: String, supported: String },
    Streaming(String),
    Import(String),
    UnsupportedFeature(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Io(_) => ErrorKind::Io,
            Self::Syntax(_) => ErrorKind::Syntax,
            Self::CityJSON(_) => ErrorKind::Model,
            Self::MissingVersion => ErrorKind::Version,
            Self::ExpectedCityJSON(_) | Self::ExpectedCityJSONFeature(_) => ErrorKind::Shape,
            Self::UnsupportedType(_)
            | Self::UnsupportedVersion { .. }
            | Self::UnsupportedFeature(_) => ErrorKind::Unsupported,
            Self::Streaming(_) => ErrorKind::Shape,
            Self::Import(_) => ErrorKind::Model,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Syntax(error) => write!(f, "JSON error: {error}"),
            Self::CityJSON(error) => write!(f, "cityjson error: {error}"),
            Self::MissingVersion => write!(f, "CityJSON object must contain a version member"),
            Self::ExpectedCityJSON(found) => {
                write!(f, "expected a CityJSON object, found {found}")
            }
            Self::ExpectedCityJSONFeature(found) => {
                write!(f, "expected a CityJSONFeature object, found {found}")
            }
            Self::UnsupportedType(found) => {
                write!(f, "unsupported CityJSON type: {found}")
            }
            Self::UnsupportedVersion { found, supported } => {
                write!(
                    f,
                    "unsupported CityJSON version {found}; supported versions: {supported}"
                )
            }
            Self::Streaming(message) => write!(f, "streaming error: {message}"),
            Self::Import(message) => write!(f, "import error: {message}"),
            Self::UnsupportedFeature(message) => write!(f, "unsupported feature: {message}"),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<cityjson::error::Error> for Error {
    fn from(value: cityjson::error::Error) -> Self {
        Self::CityJSON(value)
    }
}

#[cfg(feature = "json")]
impl From<cityjson_json::Error> for Error {
    fn from(value: cityjson_json::Error) -> Self {
        match value {
            cityjson_json::Error::Json(error) => Self::Syntax(error.to_string()),
            cityjson_json::Error::Utf8(error) => Self::Syntax(error.to_string()),
            cityjson_json::Error::CityJson(error) => Self::CityJSON(error),
            cityjson_json::Error::UnsupportedType(found) => Self::UnsupportedType(found),
            cityjson_json::Error::UnsupportedVersion(found) => Self::UnsupportedVersion {
                found,
                supported: cityjson::CityJSONVersion::V2_0.to_string(),
            },
            cityjson_json::Error::MalformedRootObject(reason) => Self::Syntax(reason.to_owned()),
            cityjson_json::Error::InvalidValue(reason) => Self::Import(reason),
            cityjson_json::Error::UnsupportedFeature(feature) => {
                Self::UnsupportedFeature(feature.to_owned())
            }
            cityjson_json::Error::UnresolvedCityObjectReference {
                source_id,
                target_id,
                relation,
            } => Self::Import(format!(
                "unresolved CityObject {relation} reference from '{source_id}' to '{target_id}'"
            )),
        }
    }
}
