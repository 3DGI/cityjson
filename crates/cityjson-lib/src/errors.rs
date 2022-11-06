//! When operations on city models go wrong.
use crate::{CityModelType, SupportedExtensions};
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
                write!(f, "Expected a CityModel type of CityJSON, but got {}", t)
            }
            Error::ExpectedCityJSONFeature(t) => {
                write!(
                    f,
                    "Expected a CityModel type of CityJSONFeature, but got {}",
                    t
                )
            }
            Error::UnsupportedVersion(v, supported) => {
                write!(
                    f,
                    "Unsupported CityJSON version: {}. Versions supported: {}",
                    v, supported
                )
            }
            Error::UnsupportedExtension => {
                write!(
                    f,
                    "Not a supported extension. Extensions supported: {}",
                    SupportedExtensions::print_all()
                )
            }
            Error::InvalidExtension(pb) => {
                write!(f, "Could not find a file extension in {}", pb.display())
            }
            Error::Io(e) => {
                write!(f, "IO Error: {}", e)
            }
            Error::MalformedCityJSON(e) => {
                write!(f, "Error while deserializing the JSON document: {}", e)
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
