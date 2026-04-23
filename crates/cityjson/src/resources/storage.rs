//! String storage strategies for the `CityJSON` data model.
//!
//! Two strategies are available: [`OwnedStringStorage`] (uses `String`) and
//! [`BorrowedStringStorage`] (uses `&str`).
//!
//! ```rust
//! use cityjson::resources::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};
//!
//! fn to_upper_text<SS: StringStorage>(value: SS::String) -> String {
//!     value.as_ref().to_uppercase()
//! }
//!
//! let owned: <OwnedStringStorage as StringStorage>::String = "example".to_string();
//! assert_eq!(to_upper_text::<OwnedStringStorage>(owned), "EXAMPLE");
//!
//! let backing = String::from("borrowed");
//! let borrowed: <BorrowedStringStorage<'_> as StringStorage>::String = backing.as_str();
//! assert_eq!(to_upper_text::<BorrowedStringStorage<'_>>(borrowed), "BORROWED");
//! ```

use std::borrow::Borrow;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;

/// Trait for string storage strategies.
pub trait StringStorage: Clone + Debug + Default + PartialEq + Eq + Hash {
    /// `String` for owned storage, `&str` for borrowed.
    type String: AsRef<str>
        + Deref<Target = str>
        + Eq
        + PartialEq
        + PartialOrd
        + Ord
        + Hash
        + Borrow<str>
        + Clone
        + Debug
        + Default
        + Display;
}

/// Storage strategy using owned `String` values.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, PartialOrd)]
pub struct OwnedStringStorage;

impl StringStorage for OwnedStringStorage {
    type String = String;
}

/// Storage strategy using borrowed `&str` references.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, PartialOrd)]
pub struct BorrowedStringStorage<'a>(PhantomData<&'a ()>);

impl<'a> StringStorage for BorrowedStringStorage<'a> {
    type String = &'a str;
}
