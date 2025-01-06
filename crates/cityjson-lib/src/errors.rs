//! When operations on city models go wrong.
use serde_cityjson::SupportedFileExtension;
use std::error;
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;

pub enum Error {
    ExpectedCityJSON(String),
    ExpectedCityJSONFeature(String),
    InvalidExtension(PathBuf),
    Io(std::io::Error),
    MalformedCityJSON(serde_json::Error, Option<serde_json::Value>), // Some(_) if JSON was syntactically valid
    MetadataError(String),
    StreamingError(String),
    UnsupportedExtension,
    UnsupportedVersion(String, String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ExpectedCityJSON(t) => {
                write!(f, "the CityModel type should be CityJSON, but got {}", t)
            }
            Error::ExpectedCityJSONFeature(t) => {
                write!(
                    f,
                    "the CityModel type should be CityJSONFeature, but got {}",
                    t
                )
            }
            Error::InvalidExtension(pb) => {
                write!(
                    f,
                    "the Path.extension method should have returned the file extension from {}",
                    pb.display()
                )
            }
            Error::Io(e) => {
                write!(f, "IO error: {}", e)
            }
            Error::MalformedCityJSON(error, value) => {
                write!(f, "error while deserializing the JSON document: {}", error)?;

                if let Some(value) = value.as_ref() {
                    write!(f, ", value: {}", value)?;
                }

                Ok(())
            }
            Error::MetadataError(s) => {
                write!(f, "{}", s)
            }
            Error::StreamingError(s) => {
                write!(f, "{}", s)
            }
            Error::UnsupportedVersion(v, supported) => {
                write!(
                    f,
                    "the CityJSON version should be one of {}, but got {}",
                    supported, v
                )
            }
            Error::UnsupportedExtension => {
                write!(
                    f,
                    "the file extension should be one of {}",
                    SupportedFileExtension
                )
            }
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::MalformedCityJSON(error, None)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_cityjson::errors::Error> for Error {
    fn from(err: serde_cityjson::errors::Error) -> Self {
        use serde_cityjson::errors::Error as CityjsonError;

        match err {
            CityjsonError::UnsupportedVersion(ver, supported) => {
                Error::UnsupportedVersion(ver, supported)
            }
            CityjsonError::IncompatibleBoundary(source, target) => Error::MalformedCityJSON(
                serde::de::Error::custom(format!(
                    "incompatible boundary: cannot convert {} boundary to {}",
                    source, target
                )),
                None,
            ),
            CityjsonError::ExpectedCityJSON(type_str) => Error::ExpectedCityJSON(type_str),
            CityjsonError::ExpectedCityJSONFeature(type_str) => {
                Error::ExpectedCityJSONFeature(type_str)
            }
            CityjsonError::InvalidExtension(path) => Error::InvalidExtension(path),
            CityjsonError::Io(err) => Error::Io(err),
            CityjsonError::MalformedCityJSON(json_err, value) => {
                Error::MalformedCityJSON(json_err, value)
            }
            CityjsonError::MetadataError(msg) => Error::MetadataError(msg),
            CityjsonError::StreamingError(msg) => Error::StreamingError(msg),
            CityjsonError::UnsupportedExtension => Error::UnsupportedExtension,
        }
    }
}

impl error::Error for Error {}
