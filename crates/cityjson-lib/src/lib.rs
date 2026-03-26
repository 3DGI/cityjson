#![allow(clippy::all, clippy::pedantic)]
#![doc = include_str!("../docs/public-api.md")]

mod error;
pub mod json;
mod format;
mod io;
mod model;
pub mod ops;
mod version;

pub use cityjson;
pub use cityjson::CityModelType;
pub use cityjson::prelude;
pub use cityjson::v2_0;
pub use error::{Error, ErrorKind, Result};
pub use model::CityModel;
pub use version::CityJSONVersion;
