use std::fmt::{Debug, Display, Formatter};

pub enum Error {
    IncompatibleBoundary(String, String),
    IndexConversion {
        source_type: String,
        target_type: String,
        value: String,
    },
    IndexOverflow {
        index_type: String,
        value: String,
    },

}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IncompatibleBoundary(source_boundarytype, target_boundarytype) => {
                write!(
                    f,
                    "cannot convert a {} to a {}",
                    source_boundarytype, target_boundarytype
                )
            }
            Error::IndexConversion {
                source_type,
                target_type,
                value,
            } => {
                write!(
                    f,
                    "failed to convert index from {} to {}: value {}",
                    source_type, target_type, value
                )
            }
            Error::IndexOverflow { index_type, value } => {
                write!(f, "index overflow for {}: value {}", index_type, value)
            }
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for Error {}
