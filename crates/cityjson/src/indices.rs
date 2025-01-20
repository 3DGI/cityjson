//! Coordinate and index definitions for CityJSON geometry boundaries, semantics, and appearance indices.
//!
//! The indices are stored internally as u32 values to reduce memory usage while maintaining
//! compatibility with typical CityJSON datasets. The implementation is optimized for performance
//! with SIMD operations where available and specialized methods for common operations.

use std::fmt;
use std::ops::{AddAssign, Index, IndexMut, Range};

/// Index type for geometry elements. Uses u32 internally to reduce memory usage while
/// maintaining compatibility with typical CityJSON datasets.
///
/// # Examples
/// ```
/// # use cjgeometry::indices::*;
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
#[derive(Copy, Clone, Default, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]

pub struct GeometryIndex(u32);

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

impl From<u32> for GeometryIndex {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

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

impl AddAssign for GeometryIndex {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl fmt::Display for GeometryIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A vector of geometry indices, optimized for u32-based indexing.
#[derive(Clone, Default, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
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
}

impl Extend<GeometryIndex> for GeometryIndices {
    /// Extends the GeometryIndices with elements from an iterator
    #[inline]
    fn extend<T: IntoIterator<Item = GeometryIndex>>(&mut self, iter: T) {
        self.0.extend(iter);
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

/// Optional index for geometry elements
pub type OptionalGeometryIndex = Option<GeometryIndex>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geometry_index_creation() {
        // Test creation of GeometryIndex from u32
        let index = GeometryIndex::new(10);
        assert_eq!(index.value(), 10);

        // Test conversion from u32 to GeometryIndex
        let index_from_u32: GeometryIndex = 20u32.into();
        assert_eq!(index_from_u32.value(), 20);

        // Test conversion from usize to GeometryIndex
        let index_try_from_usize = GeometryIndex::try_from(30usize).unwrap();
        assert_eq!(index_try_from_usize.value(), 30);
    }

    #[test]
    fn test_geometry_index_operations() {
        // Test addition of GeometryIndex values
        let mut index1 = GeometryIndex::new(5);
        let index2 = GeometryIndex::new(10);
        index1 += index2;
        assert_eq!(index1.value(), 15);

        // Test display formatting of GeometryIndex
        let index_display = GeometryIndex::new(42);
        assert_eq!(format!("{}", index_display), "42");
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn test_geometry_index_addition_overflow() {
        // Test addition overflow for GeometryIndex
        let mut index1 = GeometryIndex::new(u32::MAX);
        let index2 = GeometryIndex::new(1);
        index1 += index2; // This should panic due to overflow, but only in debug. In release, it wraps around.
    }

    #[test]
    fn test_geometry_index_edge_cases() {
        // Test creation of GeometryIndex with minimum value
        let min_index = GeometryIndex::new(0);
        assert_eq!(min_index.value(), 0);

        // Test creation of GeometryIndex with maximum value for u32
        let max_index = GeometryIndex::new(u32::MAX);
        assert_eq!(max_index.value(), u32::MAX);

        // Test conversion from usize::MAX to GeometryIndex
        let result = GeometryIndex::try_from(usize::MAX);
        assert!(result.is_err()); // Should fail due to overflow
    }

    #[test]
    fn test_geometry_indices_basic_operations() {
        let mut indices = GeometryIndices::new();
        assert!(indices.is_empty()); // Test if new GeometryIndices is empty

        let index = GeometryIndex::new(1);
        indices.push(index);
        assert_eq!(indices.len(), 1); // Test length after push
        assert_eq!(indices.pop(), Some(index)); // Test pop operation
        assert!(indices.is_empty()); // Test if empty after pop

        indices.push(GeometryIndex::new(1));
        indices.push(GeometryIndex::new(2));
        assert_eq!(indices[0], GeometryIndex::new(1)); // Test indexing
        assert_eq!(indices[1], GeometryIndex::new(2));

        let removed = indices.remove(0);
        assert_eq!(removed, GeometryIndex::new(1)); // Test remove operation
        assert_eq!(indices.len(), 1);
        assert_eq!(indices[0], GeometryIndex::new(2));

        indices.push(GeometryIndex::new(3));
        let range = indices.get_range(0..2).unwrap();
        assert_eq!(range, &[GeometryIndex::new(2), GeometryIndex::new(3)]); // Test get_range
    }

    #[test]
    fn test_geometry_indices_edge_cases() {
        let mut indices = GeometryIndices::new();

        // Test removing from an empty vector, which should panic
        let result = std::panic::catch_unwind(|| {
            let mut indices = GeometryIndices::new();
            indices.remove(0);
        });
        assert!(result.is_err()); // Should panic

        // Test getting a range that exceeds the vector length
        indices.push(GeometryIndex::new(1));
        let result = indices.get_range(0..2);
        assert!(result.is_none()); // Should return None
    }

    #[test]
    fn test_geometry_indices_reserve() {
        let mut indices = GeometryIndices::new();
        indices.reserve(10);
        assert!(indices.capacity() >= 10); // Test reserve capacity
    }

    #[test]
    fn test_geometry_indices_reserve_exact() {
        let mut indices = GeometryIndices::new();
        indices.reserve_exact(10);
        assert!(indices.capacity() >= 10); // Test reserve_exact capacity
    }

    #[test]
    fn test_geometry_indices_shrink_to_fit() {
        let mut indices = GeometryIndices::with_capacity(10);
        indices.push(GeometryIndex::new(1));
        indices.shrink_to_fit();
        assert!(indices.capacity() >= 1); // Test shrink_to_fit
    }

    #[test]
    fn test_geometry_indices_shrink_to() {
        let mut indices = GeometryIndices::with_capacity(10);
        indices.push(GeometryIndex::new(1));
        indices.shrink_to(5);
        assert!(indices.capacity() >= 5); // Test shrink_to
    }

    #[test]
    fn test_geometry_indices_extend_from_slice() {
        let mut indices = GeometryIndices::new();
        let slice = &[GeometryIndex::new(1), GeometryIndex::new(2)];
        indices.extend_from_slice(slice);
        assert_eq!(indices.len(), 2); // Test extend_from_slice
        assert_eq!(indices[0], GeometryIndex::new(1));
        assert_eq!(indices[1], GeometryIndex::new(2));
    }

    #[test]
    fn test_geometry_indices_chunks() {
        let mut indices = GeometryIndices::new();
        for i in 0..6 {
            indices.push(GeometryIndex::new(i));
        }
        let chunks: Vec<_> = indices.chunks(2).collect();
        assert_eq!(chunks.len(), 3); // Test chunks
        assert_eq!(chunks[0], &[GeometryIndex::new(0), GeometryIndex::new(1)]);
        assert_eq!(chunks[1], &[GeometryIndex::new(2), GeometryIndex::new(3)]);
        assert_eq!(chunks[2], &[GeometryIndex::new(4), GeometryIndex::new(5)]);
    }

    #[test]
    fn test_geometry_indices_windows() {
        let mut indices = GeometryIndices::new();
        for i in 0..5 {
            indices.push(GeometryIndex::new(i));
        }
        let windows: Vec<_> = indices.windows(3).collect();
        assert_eq!(windows.len(), 3); // Test windows
        assert_eq!(
            windows[0],
            &[
                GeometryIndex::new(0),
                GeometryIndex::new(1),
                GeometryIndex::new(2)
            ]
        );
        assert_eq!(
            windows[1],
            &[
                GeometryIndex::new(1),
                GeometryIndex::new(2),
                GeometryIndex::new(3)
            ]
        );
        assert_eq!(
            windows[2],
            &[
                GeometryIndex::new(2),
                GeometryIndex::new(3),
                GeometryIndex::new(4)
            ]
        );
    }

    #[test]
    fn test_geometry_indices_from_vec_u32() {
        let vec = vec![1u32, 2, 3];
        let indices: GeometryIndices = vec.into();
        assert_eq!(indices.len(), 3); // Test conversion from Vec<u32>
        assert_eq!(indices[0], GeometryIndex::new(1));
        assert_eq!(indices[1], GeometryIndex::new(2));
        assert_eq!(indices[2], GeometryIndex::new(3));
    }

    #[test]
    fn test_geometry_indices_try_from_vec_usize() {
        let vec = vec![1usize, 2, 3];
        let indices = GeometryIndices::try_from(vec).unwrap();
        assert_eq!(indices.len(), 3); // Test conversion from Vec<usize>
        assert_eq!(indices[0], GeometryIndex::new(1));
        assert_eq!(indices[1], GeometryIndex::new(2));
        assert_eq!(indices[2], GeometryIndex::new(3));

        let vec = vec![usize::MAX];
        let result = GeometryIndices::try_from(vec);
        assert!(result.is_err()); // Should fail due to overflow
    }
}
