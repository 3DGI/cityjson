use std::fmt::{Debug, Display, Formatter};

pub enum Error {
    IncompatibleBoundary(String, String),
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
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for Error {}
