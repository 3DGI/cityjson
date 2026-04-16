#![allow(clippy::all, clippy::pedantic)]
#![doc = include_str!("../docs/public-api.md")]

#[cfg(feature = "arrow")]
pub mod arrow;
mod error;
pub mod json;
pub mod ops;
mod version;

pub use cityjson;
pub use cityjson::v2_0::OwnedCityModel as CityModel;
pub use error::{Error, ErrorKind, Result};
pub use version::CityJSONVersion;
