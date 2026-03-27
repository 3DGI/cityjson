#![allow(clippy::all, clippy::pedantic)]
#![doc = include_str!("../docs/public-api.md")]

pub mod arrow;
mod error;
pub mod json;
mod model;
pub mod ops;
pub mod parquet;
mod version;

pub use cityjson;
pub use error::{Error, ErrorKind, Result};
pub use model::CityModel;
pub use version::CityJSONVersion;
