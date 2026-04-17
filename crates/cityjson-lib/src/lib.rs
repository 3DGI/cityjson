#![allow(clippy::all, clippy::pedantic)]
#![doc = include_str!("../docs/public-api.md")]

mod error;
#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "json")]
pub mod ops;
pub mod query {
    pub use cityjson::query::{ModelSummary, summary};
}
mod version;

pub use Model as CityModel;
pub use cityjson;
pub use cityjson::v2_0::OwnedCityModel as Model;
pub use error::{Error, ErrorKind, Result};
pub use version::CityJSONVersion;
