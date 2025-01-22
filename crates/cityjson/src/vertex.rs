//! # Vertex and Index Types
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
//! - Memory layout is optimized for alignment and cache efficiency
//! - Coordinate types are aligned for efficient SIMD operations
//!
//! ## Examples
//!
//! ```
//! use cityjson::vertex::*;
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
use num::{CheckedAdd, Unsigned};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::num::TryFromIntError;
use std::ops::{AddAssign, Index as IndexOp, IndexMut, Range};

//------------------------------------------------------------------------------
// Core integer trait and implementations
//------------------------------------------------------------------------------

/// Integer types that can be used for vertex indexing.
///
/// This trait is implemented for u16, u32, and u64 to allow flexibility in
/// memory usage while maintaining performance on 64-bit platforms.
pub trait VertexInteger:
    Unsigned
    + TryInto<usize>
    + TryFrom<usize, Error = TryFromIntError>
    + CheckedAdd
    + Copy
    + Debug
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

impl VertexInteger for u16 {
    const MAX: Self = u16::MAX;
    const MIN: Self = u16::MIN;
}

impl VertexInteger for u32 {
    const MAX: Self = u32::MAX;
    const MIN: Self = u32::MIN;
}

impl VertexInteger for u64 {
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
/// Default vertex index type for most use cases
pub type DefaultVertexIndex = VertexIndex32;

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
/// # use cityjson::vertex::*;
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
pub struct VertexIndex<T: VertexInteger>(T);

impl<T: VertexInteger> VertexIndex<T> {
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
    #[inline(always)]
    fn to_usize(&self) -> usize {
        unsafe {
            match std::mem::size_of::<T>() {
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
}

impl<T: VertexInteger> Display for VertexIndex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: VertexInteger> AddAssign for VertexIndex<T> {
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
// Collection types and implementations
//------------------------------------------------------------------------------

pub type VertexIndices16 = VertexIndices<u16>;
pub type VertexIndices32 = VertexIndices<u32>;
pub type VertexIndices64 = VertexIndices<u64>;

/// A generic container for vertex indices that can use different integer sizes.
///
/// # Examples
///
/// ```
/// # use cityjson::vertex::*;
/// let mut indices = VertexIndices32::new();
///
/// // Add some indices
/// indices.push(VertexIndex32::new(1));
/// indices.push(VertexIndex32::new(2));
///
/// // Access by index
/// assert_eq!(indices[0u32].value(), 1);
///
/// // Iterate over indices
/// for idx in &indices {
///     println!("Index: {}", idx.value());
/// }
/// ```
#[derive(Clone, Default, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct VertexIndices<T: VertexInteger>(Vec<VertexIndex<T>>);

impl<T: VertexInteger> VertexIndices<T> {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn with_capacity(capacity: T) -> Self {
        Self(Vec::with_capacity(VertexIndex::new(capacity).to_usize()))
    }

    #[inline]
    pub fn len(&self) -> T {
        T::try_from(self.0.len()).unwrap_or(T::MIN)
    }

    #[inline]
    pub fn len_usize(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn capacity(&self) -> T {
        T::try_from(self.0.capacity()).unwrap_or(T::MIN)
    }

    #[inline]
    pub fn push(&mut self, value: VertexIndex<T>) {
        self.0.push(value)
    }

    #[inline]
    pub fn pop(&mut self) -> Option<VertexIndex<T>> {
        self.0.pop()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    #[inline]
    pub fn get(&self, index: T) -> Option<&VertexIndex<T>> {
        self.0.get(VertexIndex::new(index).to_usize())
    }

    #[inline]
    pub fn get_mut(&mut self, index: T) -> Option<&mut VertexIndex<T>> {
        self.0.get_mut(VertexIndex::new(index).to_usize())
    }

    /// Get a reference to an element without bounds checking.
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: T) -> &VertexIndex<T> {
        self.0.get_unchecked(VertexIndex::new(index).to_usize())
    }

    /// Get a mutable reference to an element without bounds checking.
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: T) -> &mut VertexIndex<T> {
        self.0.get_unchecked_mut(VertexIndex::new(index).to_usize())
    }

    #[inline]
    pub fn as_slice(&self) -> &[VertexIndex<T>] {
        self.0.as_slice()
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, VertexIndex<T>> {
        self.0.iter()
    }

    #[inline]
    pub fn get_range(&self, range: Range<T>) -> Option<&[VertexIndex<T>]> {
        self.0
            .get(VertexIndex::new(range.start).to_usize()..VertexIndex::new(range.end).to_usize())
    }
}

impl<T: VertexInteger> IndexOp<T> for VertexIndices<T> {
    type Output = VertexIndex<T>;

    #[inline]
    fn index(&self, index: T) -> &Self::Output {
        &self.0[VertexIndex::new(index).to_usize()]
    }
}

impl<T: VertexInteger> IndexMut<T> for VertexIndices<T> {
    #[inline]
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        &mut self.0[VertexIndex::new(index).to_usize()]
    }
}

impl<T: VertexInteger> FromIterator<VertexIndex<T>> for VertexIndices<T> {
    fn from_iter<I: IntoIterator<Item = VertexIndex<T>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'a, T: VertexInteger> IntoIterator for &'a VertexIndices<T> {
    type Item = &'a VertexIndex<T>;
    type IntoIter = std::slice::Iter<'a, VertexIndex<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

//------------------------------------------------------------------------------
// Coordinate types
//------------------------------------------------------------------------------

/// Container for vertex coordinates.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Vertices(Vec<VertexCoordinate>);

/// 3D vertex coordinate
#[repr(C, align(32))]
#[derive(Clone, Debug)]
pub struct VertexCoordinate {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
}

impl VertexCoordinate {
    #[inline]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[inline]
    pub fn y(&self) -> f64 {
        self.y
    }

    #[inline]
    pub fn z(&self) -> f64 {
        self.z
    }
}

//------------------------------------------------------------------------------
// Tests
//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
        let large_idx = VertexIndex32::new(65536u32); // Just over u16::MAX
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
        let huge_idx = VertexIndex64::new(0x100000000u64); // Too big for u32
        assert!(VertexIndex32::try_from(huge_idx).is_err());
        assert!(VertexIndex16::try_from(huge_idx).is_err());
    }

    #[test]
    fn test_vertex_indices_basic() {
        let mut indices = VertexIndices32::new();
        assert!(indices.is_empty());

        indices.push(VertexIndex32::new(1));
        indices.push(VertexIndex32::new(2));
        assert_eq!(indices.len(), 2u32);

        assert_eq!(indices[0u32].value(), 1u32);
        assert_eq!(indices[1u32].value(), 2u32);
    }

    #[test]
    fn test_vertex_indices_iteration() {
        let mut indices = VertexIndices32::new();
        indices.push(VertexIndex32::new(1));
        indices.push(VertexIndex32::new(2));

        let mut sum = 0u32;
        for idx in &indices {
            sum += idx.value();
        }
        assert_eq!(sum, 3u32);
    }

    #[test]
    #[should_panic(expected = "index addition overflow")]
    fn test_vertex_index_overflow() {
        let mut idx = VertexIndex16::new(u16::MAX);
        idx += VertexIndex16::new(1);
    }

    #[test]
    fn test_vertex_indices_bounds() {
        let mut indices = VertexIndices32::new();
        indices.push(VertexIndex32::new(0));
        indices.push(VertexIndex32::new(1));

        // Test bounds checking methods
        assert!(indices.get(0u32).is_some());
        assert!(indices.get(1u32).is_some());
        assert!(indices.get(2u32).is_none());

        // Test range access
        let range = indices.get_range(0u32..2u32).unwrap();
        assert_eq!(range.len(), 2);
        assert_eq!(range[0].value(), 0u32);
        assert_eq!(range[1].value(), 1u32);

        assert!(indices.get_range(0u32..3u32).is_none());
        assert!(indices.get_range(2u32..4u32).is_none());

        // Test unchecked access (only in unsafe block)
        unsafe {
            assert_eq!(indices.get_unchecked(0u32).value(), 0u32);
            assert_eq!(indices.get_unchecked(1u32).value(), 1u32);
        }
    }

    #[test]
    fn test_vertex_indices_capacity() {
        let indices = VertexIndices32::with_capacity(10u32);
        assert!(indices.capacity() >= 10u32);
        assert!(indices.is_empty());

        // Test with different sizes
        let indices16 = VertexIndices16::with_capacity(10u16);
        assert!(indices16.capacity() >= 10u16);

        let indices64 = VertexIndices64::with_capacity(10u64);
        assert!(indices64.capacity() >= 10u64);
    }

    #[test]
    fn test_vertex_indices_clear() {
        let mut indices = VertexIndices32::new();
        indices.push(VertexIndex32::new(1));
        assert!(!indices.is_empty());

        indices.clear();
        assert!(indices.is_empty());
        assert_eq!(indices.len(), 0u32);
    }

    #[test]
    fn test_vertex_coordinate() {
        let coord = VertexCoordinate {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };

        assert_eq!(coord.x(), 1.0);
        assert_eq!(coord.y(), 2.0);
        assert_eq!(coord.z(), 3.0);
    }

    #[test]
    fn test_vertices_container() {
        let vertices = Vertices(vec![
            VertexCoordinate {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            VertexCoordinate {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        ]);

        assert_eq!(vertices.0.len(), 2);
    }
}
