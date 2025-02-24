//! # Vertex types
//!
//! This module provides efficient vertex indexing functionality for CityJSON geometries.
//! It supports different integer sizes (u16, u32, u64) for memory efficiency while
//! maintaining zero-cost abstractions on 64-bit platforms.
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
//!
//! ## Examples
//!
//! ```
//! use cityjson::index::*;
//!
//! // Create indices with different sizes
//! let small_idx = VertexIndex16::new(42u16);
//! let default_idx = VertexIndex32::new(1000u32);
//!
//! // Collect indices
//! let mut indices = VertexIndices32::new();
//! indices.push(default_idx);
//!
//! // Safe conversion between sizes
//! let larger_idx: VertexIndex32 = small_idx.try_into().unwrap();
//! ```

#[cfg(not(target_pointer_width = "64"))]
compile_error!("This crate only supports 64-bit platforms");

use crate::errors::{Error, Result};
use num::{CheckedAdd, FromPrimitive, Unsigned};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::mem::size_of;
use std::num::TryFromIntError;
use std::ops::{AddAssign, Index as IndexOp, IndexMut, Range};
//------------------------------------------------------------------------------
// Core integer trait and implementations
//------------------------------------------------------------------------------

/// An integer reference that can be used for vertex indexing.
///
/// This trait is implemented for u16, u32, and u64 to allow flexibility in
/// memory usage while maintaining performance on 64-bit platforms.
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
    const MAX: Self;
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

// Type aliases for common uses
pub type VertexIndex16 = VertexIndex<u16>;
pub type VertexIndex32 = VertexIndex<u32>;
pub type VertexIndex64 = VertexIndex<u64>;

/// A generic index type for vertices that can use different integer sizes.
///
/// # Platform Requirements
///
/// This type assumes a 64-bit platform where all integer types (u16, u32, u64)
/// can be safely converted to usize for indexing operations.
///
/// # Examples
///
/// ```
/// # use cityjson::index::*;
/// // Create indices of different sizes
/// let idx16 = VertexIndex16::new(42u16);
/// let idx32 = VertexIndex32::new(70000u32);
///
/// // Convert from smaller to larger (always succeeds)
/// let larger: VertexIndex32 = idx16.try_into().unwrap();
/// assert_eq!(larger.value(), 42);
///
/// // Convert from larger to smaller (may fail)
/// let result: Result<VertexIndex16, _> = idx32.try_into();
/// assert!(result.is_err());
/// ```
#[derive(Copy, Clone, Default, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
#[repr(transparent)]
pub struct VertexIndex<T: VertexRef>(T);

impl<T: VertexRef> VertexIndex<T> {
    /// Create a new vertex index.
    #[inline]
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Get the underlying value.
    #[inline]
    pub fn value(&self) -> T {
        self.0
    }

    /// Convert to usize for internal indexing operations.
    ///
    /// SAFETY: This is safe on 64-bit platforms as all our integer types
    /// (u16, u32, u64) fit within usize (u64)
    /// todo: fix this as self.0 as usize
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

    #[inline(always)]
    pub fn from_u32(value: u32) -> Option<Self> {
        T::from_u32(value).map(|v| Self::new(v))
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

/// A trait for converting a `Vec<T>` to a `Vec<VertexIndex<T>>`.
///
/// This trait provides a convenient way to convert a vector of raw indices into a vector of wrapped indices.
pub trait VertexIndexVec<T>
where
    T: VertexRef,
{
    /// Convert a `Vec<T>` into a `Vec<VertexIndex<T>>`.
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
    use crate::cityjson::coordinate::RealWorldCoordinate;

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
}
