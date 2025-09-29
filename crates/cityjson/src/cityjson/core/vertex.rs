//! # Vertex types
//!
//! This module provides efficient vertex indexing functionality for CityJSON geometries.
//! It supports different integer sizes (u16, u32, u64) for memory efficiency while
//! maintaining zero-cost abstractions on 64-bit platforms.
//!
//! ## Key Components
//!
//! - todo
//!
//! ## Platform Support
//!
//! This crate only supports 64-bit platforms (x86_64, aarch64, etc.) as it is designed
//! for processing large CityJSON datasets on modern desktop and server machines.
//!
//! ## Design Notes
//!
//! - All index operations use zero-cost conversions on 64-bit platforms
//! - Memory layout is optimized for alignment, and cache efficiency
//! - Coordinate types are aligned for efficient SIMD operations
//! - Index conversion is type-safe and checked at compile time and runtime
//!
//! ## Examples
//!
//! ```
//! use cityjson::prelude::*;
//!
//! // Create indices with different sizes
//! let small_idx = VertexIndex16::new(42u16);
//! let default_idx = VertexIndex32::new(1000u32);
//!
//! // Collect indices
//! let mut indices = Vec::new();
//! indices.push(default_idx);
//!
//! // Safe conversion between sizes
//! let larger_idx: VertexIndex32 = small_idx.try_into().unwrap();
//! ```
//!
//! ### Converting between index types
//!
//! ```
//! use cityjson::prelude::*;
//!
//! // Create a 16-bit index
//! let small_idx = VertexIndex16::new(42u16);
//!
//! // Convert to a 32-bit index (always succeeds)
//! let medium_idx: VertexIndex32 = small_idx.try_into().unwrap();
//! assert_eq!(medium_idx.value(), 42u32);
//!
//! // Convert to a 64-bit index (always succeeds)
//! let large_idx: VertexIndex64 = medium_idx.try_into().unwrap();
//! assert_eq!(large_idx.value(), 42u64);
//!
//! // Converting from larger to smaller may fail
//! let big_idx = VertexIndex32::new(70000u32); // Too large for u16
//! let result: Result<VertexIndex16 > = big_idx.try_into();
//! assert!(result.is_err());
//! ```
//!
//! ### Creating indices from raw values
//!
//! ```
//! use cityjson::prelude::*;
//!
//! // Create indices from integers
//! let idx1: VertexIndex16 = 42u16.into();
//! let idx2: VertexIndex32 = 1000u32.into();
//! let idx3: VertexIndex64 = 1000000u64.into();
//!
//! // You can also try to convert between types
//! let idx4: VertexIndex32 = 50u16.into(); // Small to large conversion always works
//!
//! // Large to small conversions must use try_into
//! let result = VertexIndex16::try_from(70000u32);
//! assert!(result.is_err()); // u16 can't hold 70000
//! ```
//!
//! ### Converting vectors of indices
//!
//! ```
//! use cityjson::prelude::*;
//!
//! // Raw integer vector
//! let raw_indices = vec![0u16, 1, 2, 3, 4];
//!
//! // Convert to a vector of VertexIndex16
//! let vertex_indices = raw_indices.to_vertex_indices();
//!
//! // Now you have a strongly-typed vector of indices
//! assert_eq!(vertex_indices[0].value(), 0u16);
//! assert_eq!(vertex_indices[4].value(), 4u16);
//! ```

#[cfg(not(target_pointer_width = "64"))]
compile_error!("This crate only supports 64-bit platforms");

use crate::error::{Error, Result};
use num::{CheckedAdd, FromPrimitive, Unsigned};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::mem::size_of;
use std::num::TryFromIntError;
use std::ops::AddAssign;

//------------------------------------------------------------------------------
// Core integer trait and implementations
//------------------------------------------------------------------------------

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

//------------------------------------------------------------------------------
// Index types and implementations
//------------------------------------------------------------------------------

/// A 16-bit vertex index (up to 65,535 vertices)
pub type VertexIndex16 = VertexIndex<u16>;

/// A 32-bit vertex index (up to 4,294,967,295 vertices)
pub type VertexIndex32 = VertexIndex<u32>;

/// A 64-bit vertex index (virtually unlimited vertices)
pub type VertexIndex64 = VertexIndex<u64>;

/// A generic index type for vertices that can use different integer sizes.
///
/// # Platform Requirements
///
/// This type assumes a 64-bit platform where all integer types (u16, u32, u64)
/// can be safely converted to usize for indexing operations.
///
/// # Type Parameters
///
/// * `T` - An unsigned integer type that implements the [`VertexRef`] trait
///
/// # Memory Layout
///
/// This struct uses `#[repr(transparent)]` to ensure that it has the same memory
/// layout as the underlying integer type, making it a zero-cost abstraction.
///
/// # Examples
///
/// ```
/// # use cityjson::prelude::*;
/// // Create indices of different sizes
/// let idx16 = VertexIndex16::new(42u16);
/// let idx32 = VertexIndex32::new(70000u32);
///
/// // Convert from smaller to larger (always succeeds)
/// let larger: VertexIndex32 = idx16.try_into().unwrap();
/// assert_eq!(larger.value(), 42);
///
/// // Convert from larger to smaller (may fail)
/// let result: Result<VertexIndex16 > = idx32.try_into();
/// assert!(result.is_err());
/// ```
///
/// ## Arithmetic Operations
///
/// ```
/// # use cityjson::prelude::*;
/// let mut idx = VertexIndex16::new(10);
/// idx += VertexIndex16::new(5);
/// assert_eq!(idx.value(), 15);
///
/// // Addition checks for overflow
/// let mut max_idx = VertexIndex16::new(u16::MAX);
/// // This would panic: max_idx += VertexIndex16::new(1);
/// ```
#[derive(Copy, Clone, Default, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
#[repr(transparent)]
pub struct VertexIndex<T: VertexRef>(T);

impl<T: VertexRef> VertexIndex<T> {
    /// Create a new vertex index.
    ///
    /// # Parameters
    ///
    /// * `value` - The raw index value
    ///
    /// # Returns
    ///
    /// A new `VertexIndex<T>` containing the provided value
    ///
    /// # Examples
    ///
    /// ```
    /// # use cityjson::prelude::*;
    /// let idx = VertexIndex16::new(42u16);
    /// assert_eq!(idx.value(), 42u16);
    /// ```
    #[inline]
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Get the underlying value.
    ///
    /// # Returns
    ///
    /// The raw index value of type `T`
    ///
    /// # Examples
    ///
    /// ```
    /// # use cityjson::prelude::*;
    /// let idx = VertexIndex32::new(42u32);
    /// assert_eq!(idx.value(), 42u32);
    /// ```
    #[inline]
    pub fn value(&self) -> T {
        self.0
    }

    /// Convert to usize for internal indexing operations.
    ///
    /// This method is crucial for using the index with Rust collections like
    /// Vec, as they require usize for indexing.
    ///
    /// # Safety
    ///
    /// This is safe on 64-bit platforms as all our integer types
    /// (u16, u32, u64) fit within usize (u64)
    ///
    /// # Returns
    ///
    /// The index value as a usize
    ///
    /// # Examples
    ///
    /// ```
    /// # use cityjson::prelude::*;
    /// let idx = VertexIndex16::new(42u16);
    /// let usize_value = idx.to_usize();
    /// assert_eq!(usize_value, 42usize);
    /// ```
    #[inline(always)]
    pub fn to_usize(&self) -> usize {
        unsafe {
            match size_of::<T>() {
                2 => {
                    // T = u16
                    let x: u16 = std::mem::transmute_copy(&self.0);
                    x as usize
                }
                4 => {
                    // T = u32
                    let x: u32 = std::mem::transmute_copy(&self.0);
                    x as usize
                }
                8 => {
                    // T = u64
                    let x: u64 = std::mem::transmute_copy(&self.0);
                    x as usize
                }
                _ => unreachable!("Only u16, u32, or u64 are allowed"),
            }
        }
    }

    /// Create a VertexIndex from a u32 value, if it fits in the target type.
    ///
    /// # Parameters
    ///
    /// * `value` - The u32 value to convert
    ///
    /// # Returns
    ///
    /// `Some(VertexIndex<T>)` if the conversion succeeds, or `None` if the value
    /// doesn't fit in type `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cityjson::prelude::*;
    /// // This works because 42 fits in u16
    /// let idx16 = VertexIndex16::from_u32(42).unwrap();
    /// assert_eq!(idx16.value(), 42u16);
    ///
    /// // This fails because 70000 doesn't fit in u16
    /// let bad_idx = VertexIndex16::from_u32(70000);
    /// assert!(bad_idx.is_none());
    ///
    /// // This works for u32 and u64 which can hold the value
    /// assert!(VertexIndex32::from_u32(70000).is_some());
    /// assert!(VertexIndex64::from_u32(70000).is_some());
    /// ```
    #[inline(always)]
    pub fn from_u32(value: u32) -> Option<Self> {
        T::from_u32(value).map(|v| Self::new(v))
    }

    /// Returns true if this index is at the maximum value for its type.
    ///
    /// # Returns
    ///
    /// `true` if the index is at its maximum value, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// # use cityjson::prelude::*;
    /// let idx = VertexIndex16::new(u16::MAX);
    /// assert!(idx.is_max());
    ///
    /// let idx = VertexIndex16::new(42);
    /// assert!(!idx.is_max());
    /// ```
    #[inline]
    pub fn is_max(&self) -> bool {
        self.0 == T::MAX
    }

    /// Returns true if this index is zero.
    ///
    /// # Returns
    ///
    /// `true` if the index is zero, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// # use cityjson::prelude::*;
    /// let idx = VertexIndex32::new(0);
    /// assert!(idx.is_zero());
    ///
    /// let idx = VertexIndex32::new(1);
    /// assert!(!idx.is_zero());
    /// ```
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0 == T::MIN
    }

    /// Returns the next index value if it doesn't overflow.
    ///
    /// # Returns
    ///
    /// `Some(VertexIndex<T>)` with the next value if it doesn't overflow,
    /// or `None` if adding 1 would overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cityjson::prelude::*;
    /// let idx = VertexIndex16::new(42);
    /// let next = idx.next();
    /// assert_eq!(next.unwrap().value(), 43);
    ///
    /// // This would return None because it would overflow
    /// let max_idx = VertexIndex16::new(u16::MAX);
    /// assert!(max_idx.next().is_none());
    /// ```
    #[inline]
    pub fn next(&self) -> Option<Self> {
        self.0.checked_add(&T::from_u8(1)?).map(Self::new)
    }
}

impl<T: VertexRef> Display for VertexIndex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: VertexRef> AddAssign for VertexIndex<T> {
    fn add_assign(&mut self, other: Self) {
        self.0 = self
            .0
            .checked_add(&other.0)
            .expect("index addition overflow");
    }
}

// Conversion implementations
impl TryFrom<VertexIndex<u16>> for VertexIndex<u32> {
    type Error = Error;

    fn try_from(value: VertexIndex<u16>) -> Result<Self> {
        Ok(VertexIndex(u32::from(value.0)))
    }
}

impl TryFrom<VertexIndex<u16>> for VertexIndex<u64> {
    type Error = Error;

    fn try_from(value: VertexIndex<u16>) -> Result<Self> {
        Ok(VertexIndex(u64::from(value.0)))
    }
}

impl TryFrom<VertexIndex<u32>> for VertexIndex<u64> {
    type Error = Error;

    fn try_from(value: VertexIndex<u32>) -> Result<Self> {
        Ok(VertexIndex(u64::from(value.0)))
    }
}

impl TryFrom<VertexIndex<u32>> for VertexIndex<u16> {
    type Error = Error;

    fn try_from(value: VertexIndex<u32>) -> Result<Self> {
        u16::try_from(value.0)
            .map(VertexIndex)
            .map_err(|_| Error::IndexConversion {
                source_type: "u32".to_string(),
                target_type: "u16".to_string(),
                value: value.0.to_string(),
            })
    }
}

impl TryFrom<VertexIndex<u64>> for VertexIndex<u32> {
    type Error = Error;

    fn try_from(value: VertexIndex<u64>) -> Result<Self> {
        u32::try_from(value.0)
            .map(VertexIndex)
            .map_err(|_| Error::IndexConversion {
                source_type: "u64".to_string(),
                target_type: "u32".to_string(),
                value: value.0.to_string(),
            })
    }
}

impl TryFrom<VertexIndex<u64>> for VertexIndex<u16> {
    type Error = Error;

    fn try_from(value: VertexIndex<u64>) -> Result<Self> {
        u16::try_from(value.0)
            .map(VertexIndex)
            .map_err(|_| Error::IndexConversion {
                source_type: "u64".to_string(),
                target_type: "u16".to_string(),
                value: value.0.to_string(),
            })
    }
}

//------------------------------------------------------------------------------
// Integer to VertexIndex conversions
//------------------------------------------------------------------------------

impl From<u16> for VertexIndex<u16> {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<u16> for VertexIndex<u32> {
    fn from(value: u16) -> Self {
        Self(u32::from(value))
    }
}

impl From<u16> for VertexIndex<u64> {
    fn from(value: u16) -> Self {
        Self(u64::from(value))
    }
}

impl From<u32> for VertexIndex<u32> {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<u32> for VertexIndex<u64> {
    fn from(value: u32) -> Self {
        Self(u64::from(value))
    }
}

impl From<u64> for VertexIndex<u64> {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

// Fallible conversions (TryFrom)
impl TryFrom<u32> for VertexIndex<u16> {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        u16::try_from(value)
            .map(Self)
            .map_err(|_| Error::IndexConversion {
                source_type: "u32".to_string(),
                target_type: "u16".to_string(),
                value: value.to_string(),
            })
    }
}

impl TryFrom<u64> for VertexIndex<u16> {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self> {
        u16::try_from(value)
            .map(Self)
            .map_err(|_| Error::IndexConversion {
                source_type: "u64".to_string(),
                target_type: "u16".to_string(),
                value: value.to_string(),
            })
    }
}

impl TryFrom<u64> for VertexIndex<u32> {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self> {
        u32::try_from(value)
            .map(Self)
            .map_err(|_| Error::IndexConversion {
                source_type: "u64".to_string(),
                target_type: "u32".to_string(),
                value: value.to_string(),
            })
    }
}

// usize conversions
impl<T: VertexRef> TryFrom<usize> for VertexIndex<T> {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self> {
        T::try_from(value)
            .map(Self)
            .map_err(|_| Error::IndexConversion {
                source_type: "usize".to_string(),
                target_type: std::any::type_name::<T>().to_string(),
                value: value.to_string(),
            })
    }
}

//------------------------------------------------------------------------------
// Collection types and implementations
//------------------------------------------------------------------------------

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
    /// A Result containing a vector of sequential `VertexIndex<T>` values,
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

impl<T: VertexRef> VertexIndicesSequence<T> for VertexIndex<T> {
    fn sequence(start: T, count: usize) -> Result<Vec<VertexIndex<T>>> {
        let mut result = Vec::with_capacity(count);
        let mut current = start;

        for _ in 0..count {
            result.push(VertexIndex::new(current));

            // Check if we would overflow
            if let Some(next) = current.checked_add(&T::from_u8(1).unwrap_or_default()) {
                current = next;
            } else {
                return Err(Error::IndexConversion {
                    source_type: format!("{} + 1", current),
                    target_type: std::any::type_name::<T>().to_string(),
                    value: "overflow".to_string(),
                });
            }
        }

        Ok(result)
    }
}

pub struct RawVertexView<'a, VR: VertexRef>(pub(crate) &'a [VertexIndex<VR>]);

impl<'a, VR: VertexRef> std::ops::Deref for RawVertexView<'a, VR> {
    type Target = [VR];

    fn deref(&self) -> &Self::Target {
        debug_assert_eq!(size_of::<VertexIndex<VR>>(), size_of::<VR>());
        debug_assert_eq!(align_of::<VertexIndex<VR>>(), align_of::<VR>());

        unsafe { std::slice::from_raw_parts(self.0.as_ptr() as *const VR, self.0.len()) }
    }
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

//------------------------------------------------------------------------------
// Tests
//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cityjson::core::coordinate::RealWorldCoordinate;
    use crate::cityjson::core::vertex::VertexIndexVec;
    use crate::cityjson::core::vertex::VertexIndicesSequence;
    use std::collections::HashSet;

    #[test]
    fn test_vertex_index_creation() {
        let idx16 = VertexIndex16::new(42u16);
        let idx32 = VertexIndex32::new(42u32);
        let idx64 = VertexIndex64::new(42u64);

        assert_eq!(idx16.value(), 42u16);
        assert_eq!(idx32.value(), 42u32);
        assert_eq!(idx64.value(), 42u64);
    }

    #[test]
    fn test_vertex_index_conversion() {
        // Small to large conversions (should always succeed)
        let idx16 = VertexIndex16::new(42u16);
        let idx32: VertexIndex32 = idx16.try_into().unwrap();
        let idx64: VertexIndex64 = idx16.try_into().unwrap();
        assert_eq!(idx32.value(), 42u32);
        assert_eq!(idx64.value(), 42u64);

        // Also test direct u32 to u64
        let idx32 = VertexIndex32::new(50000u32);
        let idx64: VertexIndex64 = idx32.try_into().unwrap();
        assert_eq!(idx64.value(), 50000u64);

        // Large to small conversions (should fail for large values)
        let large_idx = VertexIndex32::new((u16::MAX as u32) + 1);
        let result: Result<VertexIndex16> = large_idx.try_into();
        assert!(result.is_err());
        if let Err(Error::IndexConversion {
            source_type,
            target_type,
            value,
        }) = result
        {
            assert_eq!(source_type, "u32");
            assert_eq!(target_type, "u16");
            assert_eq!(value, "65536");
        }

        // Test u64 to smaller types
        let huge_idx = VertexIndex64::new((u32::MAX as u64) + 1); // Too big for u32
        assert!(VertexIndex32::try_from(huge_idx).is_err());
        assert!(VertexIndex16::try_from(huge_idx).is_err());
    }

    #[test]
    #[should_panic(expected = "index addition overflow")]
    fn test_vertex_index_overflow() {
        let mut idx = VertexIndex16::new(u16::MAX);
        idx += VertexIndex16::new(1);
    }

    #[test]
    fn test_vertex_coordinate() {
        let coord = RealWorldCoordinate::new(1.0, 2.0, 3.0);

        assert_eq!(coord.x(), 1.0);
        assert_eq!(coord.y(), 2.0);
        assert_eq!(coord.z(), 3.0);
    }

    #[test]
    fn test_integer_to_vertex_index_conversion() {
        // Test u16 conversions (all infallible)
        let idx16: VertexIndex<u16> = 42u16.into();
        assert_eq!(idx16.value(), 42);

        let idx32: VertexIndex<u32> = 42u16.into();
        assert_eq!(idx32.value(), 42);

        let idx64: VertexIndex<u64> = 42u16.into();
        assert_eq!(idx64.value(), 42);

        // Test u32 conversions
        let idx32: VertexIndex<u32> = 42u32.into();
        assert_eq!(idx32.value(), 42);

        let idx64: VertexIndex<u64> = 42u32.into();
        assert_eq!(idx64.value(), 42);

        let idx16: Result<VertexIndex<u16>> = 65536u32.try_into();
        assert!(idx16.is_err());

        // Test u64 conversions
        let idx64: VertexIndex<u64> = 42u64.into();
        assert_eq!(idx64.value(), 42);

        let idx32: Result<VertexIndex<u32>> = 0x100000000u64.try_into();
        assert!(idx32.is_err());

        // Test usize conversions
        let idx16: Result<VertexIndex<u16>> = 42usize.try_into();
        assert!(idx16.is_ok());
        assert_eq!(idx16.unwrap().value(), 42);

        let idx32: Result<VertexIndex<u32>> = 42usize.try_into();
        assert!(idx32.is_ok());
        assert_eq!(idx32.unwrap().value(), 42);

        let idx64: Result<VertexIndex<u64>> = 42usize.try_into();
        assert!(idx64.is_ok());
        assert_eq!(idx64.unwrap().value(), 42);
    }

    #[test]
    fn test_vertex_index_vec_trait() {
        // Test conversion of Vec<u16> to Vec<VertexIndex<u16>>
        let raw_indices = vec![0u16, 1, 2, 3, 4];
        let vertex_indices = raw_indices.to_vertex_indices();

        assert_eq!(vertex_indices.len(), 5);
        for i in 0..5 {
            assert_eq!(vertex_indices[i].value(), i as u16);
        }

        // Test conversion of Vec<u32> to Vec<VertexIndex<u32>>
        let raw_indices = vec![100u32, 200, 300];
        let vertex_indices = raw_indices.to_vertex_indices();

        assert_eq!(vertex_indices.len(), 3);
        assert_eq!(vertex_indices[0].value(), 100);
        assert_eq!(vertex_indices[1].value(), 200);
        assert_eq!(vertex_indices[2].value(), 300);
    }

    #[test]
    fn test_vertex_index_from_u32() {
        // Valid conversions
        let idx16 = VertexIndex16::from_u32(42).unwrap();
        assert_eq!(idx16.value(), 42);

        let idx32 = VertexIndex32::from_u32(70000).unwrap();
        assert_eq!(idx32.value(), 70000);

        let idx64 = VertexIndex64::from_u32(u32::MAX).unwrap();
        assert_eq!(idx64.value(), u32::MAX as u64);

        // Invalid conversion (too large for u16)
        let result = VertexIndex16::from_u32(70000);
        assert!(result.is_none());
    }

    #[test]
    fn test_vertex_index_helpers() {
        // Test is_max
        let max_idx16 = VertexIndex16::new(u16::MAX);
        assert!(max_idx16.is_max());
        let not_max_idx16 = VertexIndex16::new(100);
        assert!(!not_max_idx16.is_max());

        // Test is_zero
        let zero_idx32 = VertexIndex32::new(0);
        assert!(zero_idx32.is_zero());
        let not_zero_idx32 = VertexIndex32::new(1);
        assert!(!not_zero_idx32.is_zero());

        // Test next
        let idx = VertexIndex16::new(42);
        let next = idx.next().unwrap();
        assert_eq!(next.value(), 43);

        // Test next at maximum value
        let max_idx = VertexIndex16::new(u16::MAX);
        assert!(max_idx.next().is_none());
    }

    #[test]
    fn test_vertex_indices_sequence() {
        // Create a sequence of indices
        let indices = VertexIndex16::sequence(10, 5).unwrap();

        assert_eq!(indices.len(), 5);
        assert_eq!(indices[0].value(), 10);
        assert_eq!(indices[1].value(), 11);
        assert_eq!(indices[2].value(), 12);
        assert_eq!(indices[3].value(), 13);
        assert_eq!(indices[4].value(), 14);

        // Test sequence that would overflow
        let result = VertexIndex16::sequence(u16::MAX - 2, 5);
        assert!(result.is_err());

        // Test empty sequence
        let empty = VertexIndex32::sequence(0, 0).unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_hash_and_equality() {
        // Test that VertexIndex can be used as a key in a HashSet
        let mut set = HashSet::new();

        set.insert(VertexIndex16::new(1));
        set.insert(VertexIndex16::new(2));
        set.insert(VertexIndex16::new(3));

        // Adding a duplicate shouldn't increase the size
        set.insert(VertexIndex16::new(1));
        assert_eq!(set.len(), 3);

        assert!(set.contains(&VertexIndex16::new(1)));
        assert!(set.contains(&VertexIndex16::new(2)));
        assert!(set.contains(&VertexIndex16::new(3)));
        assert!(!set.contains(&VertexIndex16::new(4)));
    }

    #[test]
    fn test_to_usize_conversion() {
        // Test conversion to usize for different index sizes
        let idx16 = VertexIndex16::new(42);
        let idx32 = VertexIndex32::new(42);
        let idx64 = VertexIndex64::new(42);

        assert_eq!(idx16.to_usize(), 42usize);
        assert_eq!(idx32.to_usize(), 42usize);
        assert_eq!(idx64.to_usize(), 42usize);

        // Test with larger values
        let large_idx = VertexIndex32::new(100000);
        assert_eq!(large_idx.to_usize(), 100000usize);
    }
}
