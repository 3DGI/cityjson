use crate::error::Result;
use crate::prelude::VertexIndex;
use num::{CheckedAdd, FromPrimitive, Unsigned};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::num::TryFromIntError;

/// An integer reference that can be used for vertex indexing.
///
/// This trait is implemented for u16, u32, and u64 to allow flexibility in
/// memory usage while maintaining performance on 64-bit platforms.
///
/// # Type Requirements
///
/// The implementing type must satisfy several requirements:
/// - Unsigned integer type
/// - Convertible to/from usize with possible failure
/// - Support for common operations (addition, comparison, etc.)
/// - Support for numeric conversion via the num crate
///
/// # Examples
///
/// This trait is already implemented for u16, u32, and u64:
///
/// ```
/// use cityjson::prelude::*;
///
/// // Access constants from the trait
/// assert_eq!(u16::MAX, 65535);
/// assert_eq!(u32::MIN, 0);
/// ```
pub trait VertexRef:
    Unsigned
    + TryInto<usize>
    + TryFrom<usize, Error = TryFromIntError>
    + TryFrom<u32>
    + FromPrimitive
    + CheckedAdd
    + Copy
    + Debug
    + Default
    + Display
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + Hash
{
    /// The maximum value for this integer type
    const MAX: Self;

    /// The minimum value for this integer type (always 0 for unsigned types)
    const MIN: Self;
}

impl VertexRef for u16 {
    const MAX: Self = u16::MAX;
    const MIN: Self = u16::MIN;
}

impl VertexRef for u32 {
    const MAX: Self = u32::MAX;
    const MIN: Self = u32::MIN;
}

impl VertexRef for u64 {
    const MAX: Self = u64::MAX;
    const MIN: Self = u64::MIN;
}

/// A trait for converting a `Vec<T>` to a `Vec<VertexIndex<T>>`.
///
/// This trait provides a convenient way to convert a vector of raw indices into
/// a vector of wrapped indices.
///
/// # Type Parameters
///
/// * `T` - A type that implements the [`VertexRef`] trait
///
/// # Examples
///
/// ```
/// use cityjson::prelude::*;
///
/// // Create a vector of u16 values
/// let raw_indices = vec![1u16, 2, 3, 4, 5];
///
/// // Convert to a vector of VertexIndex<u16>
/// let vertex_indices = raw_indices.to_vertex_indices();
///
/// // Now you can use the strongly-typed indices
/// assert_eq!(vertex_indices.len(), 5);
/// assert_eq!(vertex_indices[0].value(), 1);
/// assert_eq!(vertex_indices[4].value(), 5);
/// ```
pub trait VertexIndexVec<T>
where
    T: VertexRef,
{
    /// Convert a `Vec<T>` into a `Vec<VertexIndex<T>>`.
    ///
    /// # Returns
    ///
    /// A new vector containing wrapped vertex indices
    fn to_vertex_indices(self) -> Vec<VertexIndex<T>>;
}

impl<T> VertexIndexVec<T> for Vec<T>
where
    T: VertexRef,
{
    fn to_vertex_indices(self) -> Vec<VertexIndex<T>> {
        self.into_iter().map(VertexIndex::new).collect()
    }
}

/// A trait for creating a collection of sequential vertex indices.
///
/// This trait allows creating a vector of sequential indices starting from a base value.
pub trait VertexIndicesSequence<T>
where
    T: VertexRef,
{
    /// Create a vector of sequential vertex indices.
    ///
    /// # Parameters
    ///
    /// * `start` - The starting index value
    /// * `count` - The number of indices to generate
    ///
    /// # Returns
    ///
    /// A Result containing a vector of sequential VertexIndex<T> values,
    /// or an error if the sequence would exceed the maximum value for type T.
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::prelude::*;
    ///
    /// // Generate a sequence of 5 indices starting from 0
    /// let indices = VertexIndex16::sequence(0, 5).unwrap();
    /// assert_eq!(indices.len(), 5);
    /// assert_eq!(indices[0].value(), 0);
    /// assert_eq!(indices[4].value(), 4);
    ///
    /// // This would fail because u16::MAX - 5 + 10 > u16::MAX
    /// let result = VertexIndex16::sequence(u16::MAX - 5, 10);
    /// assert!(result.is_err());
    /// ```
    fn sequence(start: T, count: usize) -> Result<Vec<VertexIndex<T>>>;
}
