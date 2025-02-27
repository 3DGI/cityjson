//! # String storage
//!
//! This module provides string storage strategies for the CityJSON data model.
//! It defines traits and implementations for both owned and borrowed string types,
//! allowing for flexible memory management based on application needs.
//!
//! The module supports two primary string storage strategies:
//! - `OwnedStringStorage`: Uses Rust's `String` type for full ownership of string data
//! - `BorrowedStringStorage`: Uses string references (`&str`) for borrowed data
//!
//! This design enables cityjson-rs users to choose between memory efficiency (borrowed strings)
//! and convenience/ownership (owned strings) depending on their specific use case.
//!
//! ## Examples
//!
//! ```rust
//! use cityjson::storage::{StringStorage, OwnedStringStorage, BorrowedStringStorage};
//!
//! // Using owned strings
//! type MyOwnedString = <OwnedStringStorage as StringStorage>::String;
//! let owned: MyOwnedString = "example".to_string();
//!
//! // Using borrowed strings with a lifetime
//! type MyBorrowedString<'a> = <BorrowedStringStorage<'a> as StringStorage>::String;
//! let borrowed: MyBorrowedString = "example";
//! ```

use std::borrow::Borrow;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

/// Trait for different string storage strategies (owned vs borrowed)
///
/// This trait defines the requirements for string storage implementations and
/// provides an associated type to represent the actual string type used.
/// Implementing types can specify whether strings should be owned (`String`)
/// or borrowed (`&str`) based on application needs.
pub trait StringStorage: Clone + Debug + Default {
    /// The string type (String for owned, &str for borrowed)
    ///
    /// This associated type determines the actual string representation:
    /// - `String` for owned storage
    /// - `&str` for borrowed storage
    ///
    /// The constraints ensure that regardless of the storage strategy,
    /// the string type supports all necessary operations for the CityJSON model.
    type String: AsRef<str> + Eq + Hash + Borrow<str> + Clone + Debug + Default;
}

/// Storage implementation for owned strings
///
/// This implementation uses Rust's `String` type to store string data,
/// providing full ownership and control over the memory allocation.
/// Use this storage strategy when:
/// - You need to modify string contents
/// - Strings have dynamic lifetimes
/// - You want to avoid lifetime management complexity
#[derive(Clone, Debug, Default, PartialEq, Hash, PartialOrd)]
pub struct OwnedStringStorage;

impl StringStorage for OwnedStringStorage {
    type String = String;
}

/// Storage implementation for borrowed strings
///
/// This implementation uses string references (`&str`) to avoid copying string data.
/// The lifetime parameter `'a` defines how long the string references must remain valid.
/// Use this storage strategy when:
/// - You're processing data without modifying strings
/// - Memory efficiency is critical
/// - Source data already owns the strings
#[derive(Clone, Debug, Default, PartialEq, Hash, PartialOrd)]
pub struct BorrowedStringStorage<'a>(PhantomData<&'a ()>);

impl<'a> StringStorage for BorrowedStringStorage<'a> {
    type String = &'a str;
}
