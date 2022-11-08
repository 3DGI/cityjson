//! When operations on city models go wrong.
use crate::{CityModelType, SupportedFileExtension};
use std::error;
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;

pub enum Error {
    ExpectedCityJSON(CityModelType),
    ExpectedCityJSONFeature(CityModelType),
    UnsupportedVersion(String, String),
    UnsupportedExtension,
    InvalidExtension(PathBuf),
    StreamingError(String),
    Io(std::io::Error),
    MalformedCityJSON(serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::StreamingError(s) => {
                write!(f, "{}", s)
            }
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
            Error::UnsupportedVersion(v, supported) => {
                write!(
                    f,
                    "the CityJSON version should be {}, but got {}",
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
            Error::MalformedCityJSON(e) => {
                write!(f, "error while deserializing the JSON document: {}", e)
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
        Self::MalformedCityJSON(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl error::Error for Error {}
