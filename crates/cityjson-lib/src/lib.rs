#![doc = include_str!("../docs/public-api.md")]

mod error;
mod format;
mod io;
mod model;
mod version;

pub use cityjson;
pub use cityjson::CityModelType;
pub use cityjson::prelude;
pub use cityjson::v2_0;
pub use error::{Error, Result};
pub use model::CityModel;
pub use version::CityJSONVersion;
