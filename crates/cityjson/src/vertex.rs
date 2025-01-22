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
//! - Memory layout is optimized for alignment, and cache efficiency
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
// Integer to VertexIndex conversions
//------------------------------------------------------------------------------

// u16 conversions
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

// u32 conversions
impl From<u32> for VertexIndex<u32> {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

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

impl From<u32> for VertexIndex<u64> {
    fn from(value: u32) -> Self {
        Self(u64::from(value))
    }
}

// u64 conversions
impl From<u64> for VertexIndex<u64> {
    fn from(value: u64) -> Self {
        Self(value)
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
impl<T: VertexInteger> TryFrom<usize> for VertexIndex<T> {
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

// impl TryFrom<usize> for VertexIndex<u16> {
//     type Error = Error;
//
//     fn try_from(value: usize) -> Result<Self> {
//         u16::try_from(value)
//             .map(Self)
//             .map_err(|_| Error::IndexConversion {
//                 source_type: "usize".to_string(),
//                 target_type: "u16".to_string(),
//                 value: value.to_string(),
//             })
//     }
// }
//
// impl TryFrom<usize> for VertexIndex<u32> {
//     type Error = Error;
//
//     fn try_from(value: usize) -> Result<Self> {
//         u32::try_from(value)
//             .map(Self)
//             .map_err(|_| Error::IndexConversion {
//                 source_type: "usize".to_string(),
//                 target_type: "u32".to_string(),
//                 value: value.to_string(),
//             })
//     }
// }
//
// impl TryFrom<usize> for VertexIndex<u64> {
//     type Error = Error;
//
//     fn try_from(value: usize) -> Result<Self> {
//         u64::try_from(value)
//             .map(Self)
//             .map_err(|_| Error::IndexConversion {
//                 source_type: "usize".to_string(),
//                 target_type: "u64".to_string(),
//                 value: value.to_string(),
//             })
//     }
// }

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
    pub fn with_capacity(capacity: VertexIndex<T>) -> Self {
        Self(Vec::with_capacity(capacity.to_usize()))
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
    pub fn get(&self, index: VertexIndex<T>) -> Option<&VertexIndex<T>> {
        self.0.get(index.to_usize())
    }

    #[inline]
    pub fn get_mut(&mut self, index: VertexIndex<T>) -> Option<&mut VertexIndex<T>> {
        self.0.get_mut(index.to_usize())
    }

    /// Get a reference to an element without bounds checking.
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: VertexIndex<T>) -> &VertexIndex<T> {
        self.0.get_unchecked(index.to_usize())
    }

    /// Get a mutable reference to an element without bounds checking.
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: VertexIndex<T>) -> &mut VertexIndex<T> {
        self.0.get_unchecked_mut(index.to_usize())
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
    pub fn get_range(&self, range: Range<VertexIndex<T>>) -> Option<&[VertexIndex<T>]> {
        self.0.get(range.start.to_usize()..range.end.to_usize())
    }

    /// Removes and returns the element at position `index`.
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn remove(&mut self, index: usize) -> VertexIndex<T> {
        self.0.remove(index)
    }

    /// Returns a mutable slice containing the entire underlying vector.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [VertexIndex<T>] {
        self.0.as_mut_slice()
    }

    /// Returns a reference to a contiguous subsequence without bounds checking.
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds range is undefined behavior.
    #[inline]
    pub unsafe fn get_range_unchecked(&self, range: Range<VertexIndex<T>>) -> &[VertexIndex<T>] {
        self.0
            .get_unchecked(range.start.to_usize()..range.end.to_usize())
    }

    /// Extends the container by appending all the elements in the given slice.
    #[inline]
    pub fn extend_from_slice(&mut self, other: &[VertexIndex<T>]) {
        self.0.extend_from_slice(other)
    }

    /// Returns an iterator over sub-slices of length `chunk_size`,
    /// starting at the beginning of the collection.
    #[inline]
    pub fn chunks(&self, chunk_size: usize) -> VertexIndicesChunks<'_, T> {
        VertexIndicesChunks {
            vec: self,
            chunk_size,
            index: 0,
        }
    }

    /// Returns an iterator over all contiguous windows of length `window_size`.
    /// Panics if `window_size` is zero.
    #[inline]
    pub fn windows(&self, window_size: usize) -> VertexIndicesWindows<'_, T> {
        VertexIndicesWindows {
            vec: self,
            window_size,
            index: 0,
        }
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

impl<T: VertexInteger> IndexOp<VertexIndex<T>> for VertexIndices<T> {
    type Output = VertexIndex<T>;

    #[inline]
    fn index(&self, index: VertexIndex<T>) -> &Self::Output {
        &self.0[index.to_usize()]
    }
}

impl<T: VertexInteger> IndexMut<VertexIndex<T>> for VertexIndices<T> {
    #[inline]
    fn index_mut(&mut self, index: VertexIndex<T>) -> &mut Self::Output {
        &mut self.0[index.to_usize()]
    }
}

impl<T: VertexInteger> From<Vec<T>> for VertexIndices<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value.into_iter().map(VertexIndex::new).collect())
    }
}

impl<T: VertexInteger> FromIterator<VertexIndex<T>> for VertexIndices<T> {
    fn from_iter<I: IntoIterator<Item = VertexIndex<T>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: VertexInteger> FromIterator<T> for VertexIndices<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().map(VertexIndex::new).collect())
    }
}

impl<'a, T: VertexInteger> IntoIterator for &'a VertexIndices<T> {
    type Item = &'a VertexIndex<T>;
    type IntoIter = std::slice::Iter<'a, VertexIndex<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

pub struct VertexIndicesChunks<'a, T: VertexInteger> {
    vec: &'a VertexIndices<T>,
    chunk_size: usize,
    index: usize,
}

impl<'a, T: VertexInteger> Iterator for VertexIndicesChunks<'a, T> {
    type Item = &'a [VertexIndex<T>];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.vec.len_usize() {
            return None;
        }
        let start = self.index;
        let remaining = self.vec.len_usize() - start;
        let size = self.chunk_size.min(remaining);
        let chunk = &self.vec.as_slice()[start..start + size];
        self.index += size;
        Some(chunk)
    }
}

pub struct VertexIndicesWindows<'a, T: VertexInteger> {
    vec: &'a VertexIndices<T>,
    window_size: usize,
    index: usize,
}

impl<'a, T: VertexInteger> Iterator for VertexIndicesWindows<'a, T> {
    type Item = &'a [VertexIndex<T>];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index + self.window_size > self.vec.len_usize() {
            return None;
        }
        let window = &self.vec.as_slice()[self.index..self.index + self.window_size];
        self.index += 1;
        Some(window)
    }
}

//------------------------------------------------------------------------------
// Collection types with optional values and their implementations
//------------------------------------------------------------------------------
/// A container for optional vertex indices that maintains both the indices and their presence.
#[derive(Clone, Default, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct OptionalVertexIndices<T: VertexInteger> {
    indices: VertexIndices<T>,
    mask: Vec<bool>,
}

impl<T: VertexInteger> OptionalVertexIndices<T> {
    /// Creates a new empty OptionalVertexIndices.
    #[inline]
    pub fn new() -> Self {
        Self {
            indices: VertexIndices::new(),
            mask: Vec::new(),
        }
    }

    /// Creates a new OptionalVertexIndices with the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: VertexIndex<T>) -> Self {
        Self {
            indices: VertexIndices::with_capacity(capacity),
            mask: Vec::with_capacity(capacity.to_usize()),
        }
    }

    /// Returns the number of elements.
    #[inline]
    pub fn len(&self) -> T {
        self.indices.len()
    }

    /// Returns true if the container is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Returns the capacity of the container.
    #[inline]
    pub fn capacity(&self) -> T {
        self.indices.capacity()
    }

    /// Adds an optional vertex index to the end of the container.
    #[inline]
    pub fn push(&mut self, value: Option<VertexIndex<T>>) {
        match value {
            Some(idx) => {
                self.indices.push(idx);
                self.mask.push(true);
            }
            None => {
                self.indices.push(VertexIndex::new(T::MIN));
                self.mask.push(false);
            }
        }
    }

    /// Removes and returns the last element.
    #[inline]
    pub fn pop(&mut self) -> Option<Option<VertexIndex<T>>> {
        self.mask.pop().map(|present| {
            let idx = self.indices.pop().unwrap();
            if present {
                Some(idx)
            } else {
                None
            }
        })
    }

    /// Removes all elements from the container.
    #[inline]
    pub fn clear(&mut self) {
        self.indices.clear();
        self.mask.clear();
    }

    /// Returns a reference to the element at the given index, or None if the index is out of bounds.
    #[inline]
    pub fn get(&self, index: VertexIndex<T>) -> Option<Option<&VertexIndex<T>>> {
        let idx = index.to_usize();
        if idx >= self.mask.len() {
            return None;
        }
        Some(if self.mask[idx] {
            Some(&self.indices[index])
        } else {
            None
        })
    }

    /// Returns a mutable reference to the element at the given index.
    #[inline]
    pub fn get_mut(&mut self, index: VertexIndex<T>) -> Option<Option<&mut VertexIndex<T>>> {
        let idx = index.to_usize();
        if idx >= self.mask.len() {
            return None;
        }
        Some(if self.mask[idx] {
            Some(self.indices.get_mut(index).unwrap())
        } else {
            None
        })
    }

    /// Returns a slice of the underlying indices.
    #[inline]
    pub fn as_slice(&self) -> &[VertexIndex<T>] {
        self.indices.as_slice()
    }

    /// Returns a mutable slice of the underlying indices.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [VertexIndex<T>] {
        self.indices.as_mut_slice()
    }

    /// Returns an iterator over the optional references to indices.
    #[inline]
    pub fn iter(&self) -> OptionalVertexIndicesIter<'_, T> {
        OptionalVertexIndicesIter {
            indices: &self.indices,
            mask: &self.mask,
            index: 0,
        }
    }

    /// Returns an iterator over mutable optional references to indices.
    #[inline]
    pub fn iter_mut(&mut self) -> OptionalVertexIndicesIterMut<'_, T> {
        OptionalVertexIndicesIterMut {
            indices: self.indices.as_mut_slice(),
            mask: &self.mask,
            index: 0,
        }
    }

    /// Returns a range of optional indices.
    #[inline]
    pub fn get_range(
        &self,
        range: Range<VertexIndex<T>>,
    ) -> Option<OptionalVertexIndicesRange<'_, T>> {
        let start = range.start.to_usize();
        let end = range.end.to_usize();
        if end > self.mask.len() {
            return None;
        }
        Some(OptionalVertexIndicesRange {
            indices: self.indices.get_range(range)?,
            mask: &self.mask[start..end],
        })
    }

    /// Returns an iterator over chunks of optional indices.
    #[inline]
    pub fn chunks(&self, chunk_size: usize) -> OptionalVertexIndicesChunks<'_, T> {
        OptionalVertexIndicesChunks {
            indices: self.indices.chunks(chunk_size),
            mask: self.mask.chunks(chunk_size),
        }
    }

    /// Sets whether an index is present or not.
    #[inline]
    pub fn set_present(&mut self, index: VertexIndex<T>, present: bool) {
        let idx = index.to_usize();
        if idx < self.mask.len() {
            self.mask[idx] = present;
        }
    }

    /// Returns whether an index is present.
    #[inline]
    pub fn is_present(&self, index: VertexIndex<T>) -> bool {
        let idx = index.to_usize();
        idx < self.mask.len() && self.mask[idx]
    }
}

// Iterator implementations
pub struct OptionalVertexIndicesIter<'a, T: VertexInteger> {
    indices: &'a VertexIndices<T>,
    mask: &'a [bool],
    index: usize,
}

impl<'a, T: VertexInteger> Iterator for OptionalVertexIndicesIter<'a, T> {
    type Item = Option<&'a VertexIndex<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.mask.len() {
            return None;
        }
        let current = if self.mask[self.index] {
            Some(&self.indices[T::try_from(self.index).unwrap()])
        } else {
            None
        };
        self.index += 1;
        Some(current)
    }
}

pub struct OptionalVertexIndicesIterMut<'a, T: VertexInteger> {
    indices: &'a mut [VertexIndex<T>],
    mask: &'a [bool],
    index: usize,
}

impl<'a, T: VertexInteger> Iterator for OptionalVertexIndicesIterMut<'a, T> {
    type Item = Option<&'a mut VertexIndex<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.mask.len() {
            return None;
        }
        let index = self.index;
        self.index += 1;

        // SAFETY: We're maintaining the invariant that we never return multiple mutable references
        // to the same element because we increment the index each time.
        Some(if self.mask[index] {
            Some(unsafe {
                let indices = &mut *(self.indices as *mut [VertexIndex<T>]);
                &mut indices[index]
            })
        } else {
            None
        })
    }
}

pub struct OptionalVertexIndicesRange<'a, T: VertexInteger> {
    indices: &'a [VertexIndex<T>],
    mask: &'a [bool],
}

impl<'a, T: VertexInteger> OptionalVertexIndicesRange<'a, T> {
    #[inline]
    pub fn iter(&self) -> OptionalVertexIndicesRangeIter<'a, T> {
        OptionalVertexIndicesRangeIter {
            indices: self.indices,
            mask: self.mask,
            index: 0,
        }
    }
}

pub struct OptionalVertexIndicesRangeIter<'a, T: VertexInteger> {
    indices: &'a [VertexIndex<T>],
    mask: &'a [bool],
    index: usize,
}

impl<'a, T: VertexInteger> Iterator for OptionalVertexIndicesRangeIter<'a, T> {
    type Item = Option<&'a VertexIndex<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.mask.len() {
            return None;
        }
        let current = if self.mask[self.index] {
            Some(&self.indices[self.index])
        } else {
            None
        };
        self.index += 1;
        Some(current)
    }
}

pub struct OptionalVertexIndicesChunks<'a, T: VertexInteger> {
    indices: VertexIndicesChunks<'a, T>,
    mask: std::slice::Chunks<'a, bool>,
}

impl<'a, T: VertexInteger> Iterator for OptionalVertexIndicesChunks<'a, T> {
    type Item = OptionalVertexIndicesRange<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let indices = self.indices.next()?;
        let mask = self.mask.next()?;
        Some(OptionalVertexIndicesRange { indices, mask })
    }
}

// Implementation of common traits
impl<T: VertexInteger> FromIterator<Option<VertexIndex<T>>> for OptionalVertexIndices<T> {
    fn from_iter<I: IntoIterator<Item = Option<VertexIndex<T>>>>(iter: I) -> Self {
        let mut indices = Self::new();
        for item in iter {
            indices.push(item);
        }
        indices
    }
}

impl<'a, T: VertexInteger> IntoIterator for &'a OptionalVertexIndices<T> {
    type Item = Option<&'a VertexIndex<T>>;
    type IntoIter = OptionalVertexIndicesIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: VertexInteger> IndexOp<VertexIndex<T>> for OptionalVertexIndices<T> {
    type Output = Option<VertexIndex<T>>;

    fn index(&self, _: VertexIndex<T>) -> &Self::Output {
        unimplemented!("Cannot directly index OptionalVertexIndices. Use get() instead.")
    }
}

//------------------------------------------------------------------------------
// Tests
//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::VertexCoordinate;

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
    fn test_vertex_indices_basic() {
        let mut indices = VertexIndices32::new();
        assert!(indices.is_empty());

        let vi1 = VertexIndex32::new(1);
        let vi2 = VertexIndex32::new(2);
        indices.push(vi1);
        indices.push(vi2);
        assert_eq!(indices.len(), 2u32);

        assert_eq!(indices[0u32], vi1);
        assert_eq!(indices[vi1], vi2);
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
        assert!(indices.get(0u32.into()).is_some());
        assert!(indices.get(1u32.into()).is_some());
        assert!(indices.get(2u32.into()).is_none());

        // Test range access
        let range = indices.get_range(0u32.into()..2u32.into()).unwrap();
        assert_eq!(range.len(), 2);
        assert_eq!(range[0].value(), 0u32);
        assert_eq!(range[1].value(), 1u32);

        assert!(indices.get_range(0u32.into()..3u32.into()).is_none());
        assert!(indices.get_range(2u32.into()..4u32.into()).is_none());

        // Test unchecked access (only in unsafe block)
        unsafe {
            assert_eq!(indices.get_unchecked(0u32.into()).value(), 0u32);
            assert_eq!(indices.get_unchecked(1u32.into()).value(), 1u32);
        }
    }

    #[test]
    fn test_vertex_indices_capacity() {
        let indices = VertexIndices32::with_capacity(10u32.into());
        assert!(indices.capacity() >= 10u32);
        assert!(indices.is_empty());

        // Test with different sizes
        let indices16 = VertexIndices16::with_capacity(10u16.into());
        assert!(indices16.capacity() >= 10u16);

        let indices64 = VertexIndices64::with_capacity(10u64.into());
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
    fn test_vertex_indices_remove() {
        let mut indices = VertexIndices32::new();
        indices.push(VertexIndex32::new(11));
        indices.push(VertexIndex32::new(22));
        indices.push(VertexIndex32::new(33));

        // Remove the element at index 1
        let removed = indices.remove(1);
        assert_eq!(removed.value(), 22);
        assert_eq!(indices.len(), 2u32);
        assert_eq!(indices[0u32].value(), 11);
        assert_eq!(indices[1u32].value(), 33);
    }

    #[test]
    fn test_vertex_indices_as_mut_slice() {
        let mut indices = VertexIndices32::new();
        indices.push(VertexIndex32::new(42));
        indices.push(VertexIndex32::new(99));

        // Mutate via as_mut_slice
        let slice = indices.as_mut_slice();
        slice[0] = VertexIndex32::new(100);

        assert_eq!(indices[0u32].value(), 100);
        assert_eq!(indices[1u32].value(), 99);
    }

    #[test]
    fn test_vertex_indices_get_range_unchecked() {
        let mut indices = VertexIndices32::new();
        indices.push(VertexIndex32::new(1));
        indices.push(VertexIndex32::new(2));
        indices.push(VertexIndex32::new(3));

        // Use get_range_unchecked in an unsafe block
        unsafe {
            let range = indices.get_range_unchecked(VertexIndex(0u32)..VertexIndex(2u32));
            assert_eq!(range.len(), 2);
            assert_eq!(range[0].value(), 1);
            assert_eq!(range[1].value(), 2);
        }
    }

    #[test]
    fn test_vertex_indices_extend_from_slice() {
        let mut indices = VertexIndices32::new();
        indices.push(VertexIndex32::new(10));

        let extra = &[
            VertexIndex32::new(20),
            VertexIndex32::new(30),
            VertexIndex32::new(40),
        ];
        indices.extend_from_slice(extra);

        assert_eq!(indices.len(), 4u32);
        assert_eq!(indices[1u32].value(), 20);
        assert_eq!(indices[2u32].value(), 30);
        assert_eq!(indices[3u32].value(), 40);
    }

    #[test]
    fn test_vertex_indices_chunks() {
        let mut indices = VertexIndices32::new();
        for i in 0..6 {
            indices.push(VertexIndex32::new(i));
        }

        let mut chunk_iter = indices.chunks(2);

        let chunk1 = chunk_iter.next().unwrap();
        assert_eq!(chunk1.len(), 2);
        assert_eq!(chunk1[0].value(), 0);
        assert_eq!(chunk1[1].value(), 1);

        let chunk2 = chunk_iter.next().unwrap();
        assert_eq!(chunk2.len(), 2);
        assert_eq!(chunk2[0].value(), 2);
        assert_eq!(chunk2[1].value(), 3);

        let chunk3 = chunk_iter.next().unwrap();
        assert_eq!(chunk3.len(), 2);
        assert_eq!(chunk3[0].value(), 4);
        assert_eq!(chunk3[1].value(), 5);

        assert!(chunk_iter.next().is_none());
    }

    #[test]
    fn test_vertex_indices_windows() {
        let mut indices = VertexIndices32::new();
        for i in 0..5 {
            indices.push(VertexIndex32::new(i));
        }

        let mut window_iter = indices.windows(3);

        let w1 = window_iter.next().unwrap();
        assert_eq!(w1.len(), 3);
        assert_eq!(w1[0].value(), 0);
        assert_eq!(w1[1].value(), 1);
        assert_eq!(w1[2].value(), 2);

        let w2 = window_iter.next().unwrap();
        assert_eq!(w2.len(), 3);
        assert_eq!(w2[0].value(), 1);
        assert_eq!(w2[1].value(), 2);
        assert_eq!(w2[2].value(), 3);

        let w3 = window_iter.next().unwrap();
        assert_eq!(w3.len(), 3);
        assert_eq!(w3[0].value(), 2);
        assert_eq!(w3[1].value(), 3);
        assert_eq!(w3[2].value(), 4);

        assert!(window_iter.next().is_none());
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

#[cfg(test)]
mod tests_optional {
    use super::*;

    #[test]
    fn test_optional_vertex_indices_basic() {
        let mut indices = OptionalVertexIndices::<u32>::new();

        indices.push(Some(VertexIndex32::new(1)));
        indices.push(None);
        indices.push(Some(VertexIndex32::new(3)));

        assert_eq!(indices.len(), 3u32);
        assert!(!indices.is_empty());

        // Test get
        assert_eq!(
            indices.get(VertexIndex32::new(0)).unwrap().unwrap().value(),
            1
        );
        assert_eq!(indices.get(VertexIndex32::new(1)).unwrap(), None);
        assert_eq!(
            indices.get(VertexIndex32::new(2)).unwrap().unwrap().value(),
            3
        );
    }

    #[test]
    fn test_optional_vertex_indices_iteration() {
        let mut indices = OptionalVertexIndices::<u32>::new();
        indices.push(Some(VertexIndex32::new(1)));
        indices.push(None);
        indices.push(Some(VertexIndex32::new(3)));

        let mut iter = indices.iter();
        assert_eq!(iter.next().unwrap().unwrap().value(), 1);
        assert_eq!(iter.next().unwrap(), None);
        assert_eq!(iter.next().unwrap().unwrap().value(), 3);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_optional_vertex_indices_chunks() {
        let mut indices = OptionalVertexIndices::<u32>::new();
        for i in 0..6 {
            indices.push(if i % 2 == 0 {
                Some(VertexIndex32::new(i))
            } else {
                None
            });
        }

        let chunks: Vec<_> = indices.chunks(2).collect();
        assert_eq!(chunks.len(), 3);
    }

    #[test]
    fn test_optional_vertex_indices_clear() {
        let mut indices = OptionalVertexIndices::<u32>::new();
        indices.push(Some(VertexIndex32::new(1)));
        indices.push(None);

        assert_eq!(indices.len(), 2u32);
        indices.clear();
        assert_eq!(indices.len(), 0u32);
        assert!(indices.is_empty());
    }

    #[test]
    fn test_optional_vertex_indices_presence() {
        let mut indices = OptionalVertexIndices::<u32>::new();
        indices.push(Some(VertexIndex32::new(1)));
        indices.push(None);

        assert!(indices.is_present(VertexIndex32::new(0)));
        assert!(!indices.is_present(VertexIndex32::new(1)));

        indices.set_present(VertexIndex32::new(1), true);
        assert!(indices.is_present(VertexIndex32::new(1)));
    }
}
