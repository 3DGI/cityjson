use std::borrow::Cow;

use cityjson::prelude::{BorrowedStringStorage, OwnedStringStorage};
use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{BorrowedCityModel, CityModel, OwnedCityModel};

use crate::de::build::build_model;
use crate::de::root::RawRoot;
use crate::errors::{Error, Result};

/// Adapter trait that bridges `StringStorage` with a parse-time lifetime.
///
/// This is the only owned-vs-borrowed switch in the entire parser. Every
/// import function is generic over `SS: ParseStringStorage<'de>`.
///
/// `store` converts a `&'de str` borrowed from the JSON input into `SS::String`.
/// `store_cow` converts a `Cow<'de, str>` — handling escaped strings in JSON
/// that cannot be zero-copy borrowed.
pub trait ParseStringStorage<'de>: StringStorage {
    /// Convert a string borrowed from the JSON input into the storage string type.
    fn store(value: &'de str) -> Self::String;

    /// Convert a potentially-owned `Cow` string into the storage string type.
    ///
    /// For `OwnedStringStorage` this always succeeds.
    /// For `BorrowedStringStorage`, this returns `Err` when the string was
    /// escaped in JSON and therefore could not be zero-copy borrowed.
    ///
    /// # Errors
    ///
    /// Returns an error if the string contains JSON escape sequences and the
    /// storage type requires zero-copy borrowing.
    fn store_cow(value: Cow<'de, str>) -> Result<Self::String>;
}

impl<'de> ParseStringStorage<'de> for OwnedStringStorage {
    fn store(value: &'de str) -> String {
        value.to_owned()
    }

    fn store_cow(value: Cow<'de, str>) -> Result<String> {
        Ok(value.into_owned())
    }
}

impl<'de> ParseStringStorage<'de> for BorrowedStringStorage<'de> {
    fn store(value: &'de str) -> &'de str {
        value
    }

    fn store_cow(value: Cow<'de, str>) -> Result<&'de str> {
        match value {
            Cow::Borrowed(s) => Ok(s),
            Cow::Owned(_) => Err(Error::InvalidValue(
                "attribute string contains JSON escape sequences; not supported in borrowed mode"
                    .to_owned(),
            )),
        }
    }
}

/// Parse a `CityJSON` document into a `CityModel<u32, SS>`.
///
/// This is the primary entry point. Both owned and borrowed storage modes
/// go through this single implementation.
///
/// # Errors
///
/// Returns an error if the input is not valid `CityJSON`.
pub fn from_str<'de, SS>(input: &'de str) -> Result<CityModel<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let raw: RawRoot<'de> = serde_json::from_str(input)?;
    build_model::<SS>(raw)
}

pub(crate) fn from_str_owned(input: &str) -> Result<OwnedCityModel> {
    from_str::<OwnedStringStorage>(input)
}

pub(crate) fn from_str_borrowed<'de>(input: &'de str) -> Result<BorrowedCityModel<'de>> {
    from_str::<BorrowedStringStorage<'de>>(input)
}
