#![cfg_attr(docsrs, feature(doc_cfg))]
//! `serde_cityjson` is a [`CityJSON`] v2.0 serde adapter around [`cityjson`].

mod de;
pub mod errors;
mod ser;
pub mod v2_0;

pub use cityjson::prelude;
pub use cityjson::v2_0::{BorrowedCityModel, CityModel, OwnedCityModel};
pub use cityjson::{CityJSONVersion, CityModelType};

pub use errors::{Error, Result};
pub use v2_0::{
    as_json, from_str, from_str_borrowed, from_str_owned, to_string, to_string_validated,
    ParseStringStorage, SerializableCityModel,
};
