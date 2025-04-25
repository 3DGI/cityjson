//! # Error types
//!
//! When operations go wrong.
use arrow::error::ArrowError;
use std::fmt::{Debug, Display, Formatter};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Arrow(ArrowError),
    CityJSON(cityjson::error::Error),
    Conversion(String),
    Unsupported(String),
    SchemaMismatch { expected: String, found: String },
    MissingField(String),
    Io(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Arrow(e) => write!(f, "Arrow error: {}", e),
            Error::CityJSON(e) => write!(f, "CityJSON error: {}", e),
            Error::Conversion(s) => write!(f, "could not convert due to {}", s),
            Error::Unsupported(s) => write!(f, "feature {} is not supported", s),
            Error::SchemaMismatch { expected, found } => {
                write!(f, "expected schema: {}, found schema: {}", expected, found)
            }
            Error::MissingField(s) => write!(f, "field {} should be present in the Arrow data", s),
            Error::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl From<ArrowError> for Error {
    fn from(value: ArrowError) -> Self {
        Self::Arrow(value)
    }
}

impl From<cityjson::error::Error> for Error {
    fn from(value: cityjson::error::Error) -> Self {
        Self::CityJSON(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl std::error::Error for Error {}
