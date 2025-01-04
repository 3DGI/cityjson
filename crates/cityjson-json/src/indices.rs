//! Coordinate and index definitions for CityJSON geometry boundaries, semantics, and appearance indices.
//!
//! The indices are stored internally as u32 values to reduce memory usage while maintaining
//! compatibility with typical CityJSON datasets. The implementation is optimized for performance
//! with SIMD operations where available and specialized methods for common operations.

#[cfg(feature = "datasize")]
use datasize::DataSize;
use derive_more::{AddAssign, Display, From};
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut, Range};

/// Index type for geometry elements. Uses u32 internally to reduce memory usage while
/// maintaining compatibility with typical CityJSON datasets.
///
/// # Examples
/// ```
/// # use serde_cityjson::indices::*;
/// # fn main() -> Result<(), String> {
/// let _: GeometryIndex = 0u32.into();
/// let _: GeometryIndex = 0usize.try_into().unwrap();
/// assert_eq!(GeometryIndex::new(0), 0u32.into());
/// let _ = GeometryIndex::from(0u32);
/// let _ = GeometryIndex::try_from(0usize).unwrap();
/// let _: usize = usize::try_from(GeometryIndex::new(0)).unwrap();
/// # Ok(())
/// # }
/// ```
#[derive(
    AddAssign,
    Copy,
    Clone,
    Default,
    Debug,
    Display,
    From,
    Deserialize,
    Serialize,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
    Hash,
)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct GeometryIndex(u32);

impl From<&GeometryIndex> for u32 {
    fn from(value: &GeometryIndex) -> Self {
        value.0
    }
}

impl TryFrom<GeometryIndex> for usize {
    type Error = std::num::TryFromIntError;

    fn try_from(value: GeometryIndex) -> Result<Self, Self::Error> {
        usize::try_from(u32::from(&value))
    }
}

impl TryFrom<usize> for GeometryIndex {
    type Error = std::num::TryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        u32::try_from(value).map(GeometryIndex)
    }
}

impl GeometryIndex {
    /// Create a new GeometryIndex
    #[inline]
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Get the underlying u32 value
    #[inline]
    pub fn value(&self) -> u32 {
        self.0
    }
}

/// A vector of geometry indices, optimized for u32-based indexing.
#[derive(Clone, Default, Debug, Eq, Ord, PartialOrd, PartialEq, Hash, Deserialize, Serialize)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct GeometryIndices(Vec<GeometryIndex>);

impl GeometryIndices {
    /// Create a new empty vector
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Create a new vector with the specified capacity
    #[inline]
    pub fn with_capacity(capacity: u32) -> Self {
        Self(Vec::with_capacity(capacity as usize))
    }

    /// Returns the number of elements in the vector
    #[inline]
    pub fn len(&self) -> u32 {
        self.0.len().try_into().unwrap_or(u32::MAX)
    }

    /// Returns the number of elements in the vector as usize
    #[inline]
    pub fn len_usize(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the vector contains no elements
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of elements the vector can hold without reallocating
    #[inline]
    pub fn capacity(&self) -> u32 {
        self.0.capacity().try_into().unwrap_or(u32::MAX)
    }

    /// Appends an element to the back of the vector
    #[inline]
    pub fn push(&mut self, value: GeometryIndex) {
        self.0.push(value)
    }

    /// Removes the last element and returns it
    #[inline]
    pub fn pop(&mut self) -> Option<GeometryIndex> {
        self.0.pop()
    }

    /// Removes and returns the element at position index
    #[inline]
    pub fn remove(&mut self, index: u32) -> GeometryIndex {
        self.0.remove(index as usize)
    }

    /// Returns a slice containing the entire vector
    #[inline]
    pub fn as_slice(&self) -> &[GeometryIndex] {
        self.0.as_slice()
    }

    /// Returns a mutable slice containing the entire vector
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [GeometryIndex] {
        self.0.as_mut_slice()
    }

    /// Returns a reference to an element at the given index
    #[inline]
    pub fn get(&self, index: u32) -> Option<&GeometryIndex> {
        self.0.get(index as usize)
    }

    /// Returns a reference to an element at the given index without bounds checking
    #[inline]
    pub unsafe fn get_unchecked(&self, index: u32) -> &GeometryIndex {
        self.0.get_unchecked(index as usize)
    }

    /// Returns a mutable reference to an element at the given index
    #[inline]
    pub fn get_mut(&mut self, index: u32) -> Option<&mut GeometryIndex> {
        self.0.get_mut(index as usize)
    }

    /// Returns a mutable reference to an element at the given index without bounds checking
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: u32) -> &mut GeometryIndex {
        self.0.get_unchecked_mut(index as usize)
    }

    /// Returns a reference to a contiguous sequence of elements
    #[inline]
    pub fn get_range(&self, range: Range<u32>) -> Option<&[GeometryIndex]> {
        self.0.get(range.start as usize..range.end as usize)
    }

    /// Returns a reference to a contiguous sequence of elements without bounds checking
    #[inline]
    pub unsafe fn get_range_unchecked(&self, range: Range<u32>) -> &[GeometryIndex] {
        self.0
            .get_unchecked(range.start as usize..range.end as usize)
    }

    /// Reserves capacity for at least additional more elements
    #[inline]
    pub fn reserve(&mut self, additional: u32) {
        self.0.reserve(additional as usize)
    }

    /// Reserves the minimum capacity for exactly additional more elements
    #[inline]
    pub fn reserve_exact(&mut self, additional: u32) {
        self.0.reserve_exact(additional as usize)
    }

    /// Shrinks the capacity of the vector as much as possible
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    /// Shrinks the capacity of the vector with a lower bound
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: u32) {
        self.0.shrink_to(min_capacity as usize)
    }

    /// Clears the vector, removing all elements
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Returns an iterator over chunks of size chunk_size
    #[inline]
    pub fn chunks(&self, chunk_size: u32) -> GeometryIndicesChunks<'_> {
        GeometryIndicesChunks {
            vec: self,
            chunk_size,
            index: 0,
        }
    }

    /// Returns an iterator over windows of size window_size
    #[inline]
    pub fn windows(&self, window_size: u32) -> GeometryIndicesWindows<'_> {
        assert!(window_size > 0);
        GeometryIndicesWindows {
            vec: self,
            window_size,
            index: 0,
        }
    }

    /// Extends the vector with elements from a slice
    #[inline]
    pub fn extend_from_slice(&mut self, other: &[GeometryIndex]) {
        self.0.extend_from_slice(other)
    }

    /// Creates a draining iterator that removes the specified range
    pub fn drain<R>(&mut self, range: R) -> std::vec::Drain<'_, GeometryIndex>
    where
        R: std::ops::RangeBounds<u32>,
    {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&n) => n as usize,
            std::ops::Bound::Excluded(&n) => (n + 1) as usize,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&n) => (n + 1) as usize,
            std::ops::Bound::Excluded(&n) => n as usize,
            std::ops::Bound::Unbounded => self.0.len(),
        };
        self.0.drain(start..end)
    }

    /// Returns an iterator over the vector
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, GeometryIndex> {
        self.0.iter()
    }

    /// Returns a mutable iterator over the vector
    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, GeometryIndex> {
        self.0.iter_mut()
    }

    #[cfg(feature = "simd")]
    /// Add elements from another vector using SIMD operations where available
    pub fn add_assign_vectorized(&mut self, rhs: &Self) {
        use std::simd::*;

        let chunks = self.0.chunks_exact_mut(4);
        let rhs_chunks = rhs.0.chunks_exact(4);

        for (chunk, rhs_chunk) in chunks.zip(rhs_chunks) {
            let a = u32x4::from_slice(unsafe {
                std::slice::from_raw_parts(chunk.as_ptr() as *const u32, 4)
            });
            let b = u32x4::from_slice(unsafe {
                std::slice::from_raw_parts(rhs_chunk.as_ptr() as *const u32, 4)
            });
            let res = a + b;
            res.copy_to_slice(unsafe {
                std::slice::from_raw_parts_mut(chunk.as_mut_ptr() as *mut u32, 4)
            });
        }

        // Handle the remainder
        let (chunks, remainder) = self
            .0
            .split_at_mut(self.len() as usize - self.len() as usize % 4);
        let (_, rhs_remainder) = rhs.0.split_at(rhs.len() as usize - rhs.len() as usize % 4);

        for (a, b) in remainder.iter_mut().zip(rhs_remainder.iter()) {
            *a += *b;
        }
    }
}

impl Extend<GeometryIndex> for GeometryIndices {
    /// Extends the GeometryIndices with elements from an iterator
    #[inline]
    fn extend<T: IntoIterator<Item = GeometryIndex>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

/// Iterator over chunks of a GeometryIndices
pub struct GeometryIndicesChunks<'a> {
    vec: &'a GeometryIndices,
    chunk_size: u32,
    index: u32,
}

impl<'a> Iterator for GeometryIndicesChunks<'a> {
    type Item = &'a [GeometryIndex];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.vec.len() {
            return None;
        }
        let start = self.index as usize;
        let remaining = self.vec.len() as usize - start;
        let chunk_size = std::cmp::min(self.chunk_size as usize, remaining);
        let chunk = unsafe {
            self.vec
                .get_range_unchecked(self.index..self.index + chunk_size as u32)
        };
        self.index += chunk_size as u32;
        Some(chunk)
    }
}

/// Iterator over windows of a GeometryIndices
pub struct GeometryIndicesWindows<'a> {
    vec: &'a GeometryIndices,
    window_size: u32,
    index: u32,
}

impl<'a> Iterator for GeometryIndicesWindows<'a> {
    type Item = &'a [GeometryIndex];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index + self.window_size > self.vec.len() {
            return None;
        }
        let window = unsafe {
            self.vec
                .get_range_unchecked(self.index..self.index + self.window_size)
        };
        self.index += 1;
        Some(window)
    }
}

impl Index<u32> for GeometryIndices {
    type Output = GeometryIndex;

    fn index(&self, index: u32) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u32> for GeometryIndices {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl FromIterator<GeometryIndex> for GeometryIndices {
    fn from_iter<T: IntoIterator<Item = GeometryIndex>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl From<Vec<u32>> for GeometryIndices {
    fn from(value: Vec<u32>) -> Self {
        Self(value.into_iter().map(GeometryIndex::new).collect())
    }
}

impl TryFrom<Vec<usize>> for GeometryIndices {
    type Error = std::num::TryFromIntError;

    fn try_from(value: Vec<usize>) -> Result<Self, Self::Error> {
        let mut vec = Self::with_capacity(value.len() as u32);
        for v in value {
            vec.push(GeometryIndex::try_from(v)?);
        }
        Ok(vec)
    }
}

impl IntoIterator for GeometryIndices {
    type Item = GeometryIndex;
    type IntoIter = std::vec::IntoIter<GeometryIndex>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a GeometryIndices {
    type Item = &'a GeometryIndex;
    type IntoIter = std::slice::Iter<'a, GeometryIndex>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Optional index for geometry elements
pub type OptionalGeometryIndex = Option<GeometryIndex>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_conversions() {
        let idx = GeometryIndex::new(42);
        assert_eq!(u32::from(&idx), 42);
        assert_eq!(usize::try_from(idx).unwrap(), 42);
        assert_eq!(GeometryIndex::try_from(42usize).unwrap(), idx);
    }

    #[test]
    fn test_indices_operations() {
        let mut indices = GeometryIndices::new();
        indices.push(GeometryIndex::new(1));
        indices.push(GeometryIndex::new(2));
        indices.push(GeometryIndex::new(3));

        assert_eq!(indices.len(), 3);
        assert_eq!(indices[0], GeometryIndex::new(1));
        assert_eq!(indices.get(1), Some(&GeometryIndex::new(2)));

        indices.remove(1);
        assert_eq!(indices.len(), 2);
        assert_eq!(indices[1], GeometryIndex::new(3));
    }

    #[test]
    fn test_indices_from_vec() {
        let vec = vec![1u32, 2, 3];
        let indices = GeometryIndices::from(vec);
        assert_eq!(indices[0], GeometryIndex::new(1));
        assert_eq!(indices[1], GeometryIndex::new(2));
        assert_eq!(indices[2], GeometryIndex::new(3));
    }

    #[test]
    fn test_chunks_iterator() {
        let indices = GeometryIndices::from(vec![1u32, 2, 3, 4, 5, 6]);
        let chunks: Vec<_> = indices.chunks(2).collect();
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), 2);
        assert_eq!(chunks[1].len(), 2);
        assert_eq!(chunks[2].len(), 2);
    }

    #[test]
    fn test_windows_iterator() {
        let indices = GeometryIndices::from(vec![1u32, 2, 3, 4]);
        let windows: Vec<_> = indices.windows(2).collect();
        assert_eq!(windows.len(), 3);
        assert_eq!(windows[0].len(), 2);
        assert_eq!(windows[1].len(), 2);
        assert_eq!(windows[2].len(), 2);
    }
}
