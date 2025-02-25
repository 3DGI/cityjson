//! # String storage
use std::borrow::Borrow;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

/// Trait for different string storage strategies (owned vs borrowed)
pub trait StringStorage: Clone + Debug + Default {
    /// The string type (String for owned, &str for borrowed)
    type String: AsRef<str> + Eq + Hash + Borrow<str> + Clone + Debug + Default;
}

/// Storage implementation for owned strings
#[derive(Clone, Debug, Default)]
pub struct OwnedStringStorage;

impl StringStorage for OwnedStringStorage {
    type String = String;
}

/// Storage implementation for borrowed strings
#[derive(Clone, Debug, Default)]
pub struct BorrowedStringStorage<'a>(PhantomData<&'a ()>);

impl<'a> StringStorage for BorrowedStringStorage<'a> {
    type String = &'a str;
}
