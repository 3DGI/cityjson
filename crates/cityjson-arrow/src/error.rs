//! # Error types
//!
//! When operations go wrong.
use arrow::error::ArrowError;
use parquet::errors::ParquetError;
use std::fmt::{Debug, Display, Formatter};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Arrow(ArrowError),
    Parquet(ParquetError),
    CityJSON(cityjson_types::error::Error),
    Json(serde_json::Error),
    Conversion(String),
    Unsupported(String),
    SchemaMismatch { expected: String, found: String },
    MissingField(String),
    Io(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Arrow(e) => write!(f, "Arrow error: {e}"),
            Error::Parquet(e) => write!(f, "Parquet error: {e}"),
            Error::CityJSON(e) => write!(f, "CityJSON error: {e}"),
            Error::Json(e) => write!(f, "JSON error: {e}"),
            Error::Conversion(s) => write!(f, "could not convert due to {s}"),
            Error::Unsupported(s) => write!(f, "feature {s} is not supported"),
            Error::SchemaMismatch { expected, found } => {
                write!(f, "expected schema: {expected}, found schema: {found}")
            }
            Error::MissingField(s) => write!(f, "field {s} should be present in the Arrow data"),
            Error::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl From<ArrowError> for Error {
    fn from(value: ArrowError) -> Self {
        Self::Arrow(value)
    }
}

impl From<ParquetError> for Error {
    fn from(value: ParquetError) -> Self {
        Self::Parquet(value)
    }
}

impl From<cityjson_types::error::Error> for Error {
    fn from(value: cityjson_types::error::Error) -> Self {
        Self::CityJSON(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl std::error::Error for Error {}
