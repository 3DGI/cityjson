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
#[derive(
    Copy,
    Clone,
    Default,
    Debug,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
    Hash,
)]

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

    /// Returns a reference to a contiguous sequence of elements
    #[inline]
    pub fn get_range(&self, range: Range<u32>) -> Option<&[GeometryIndex]> {
        self.0.get(range.start as usize..range.end as usize)
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
    fn test_geometry_index_creation() {
        let index = GeometryIndex::new(10);
        assert_eq!(index.value(), 10);
    }

    #[test]
    fn test_geometry_index_from_u32() {
        let index: GeometryIndex = 20u32.into();
        assert_eq!(index.value(), 20);
    }

    #[test]
    fn test_geometry_index_try_from_usize() {
        let index = GeometryIndex::try_from(30usize).unwrap();
        assert_eq!(index.value(), 30);
    }

    #[test]
    fn test_geometry_index_add_assign() {
        let mut index1 = GeometryIndex::new(5);
        let index2 = GeometryIndex::new(10);
        index1 += index2;
        assert_eq!(index1.value(), 15);
    }

    #[test]
    fn test_geometry_index_display() {
        let index = GeometryIndex::new(42);
        assert_eq!(format!("{}", index), "42");
    }

    #[test]
    fn test_geometry_indices_creation() {
        let indices = GeometryIndices::new();
        assert!(indices.is_empty());
    }

    #[test]
    fn test_geometry_indices_push_pop() {
        let mut indices = GeometryIndices::new();
        let index = GeometryIndex::new(1);
        indices.push(index);
        assert_eq!(indices.len(), 1);
        assert_eq!(indices.pop(), Some(index));
        assert!(indices.is_empty());
    }

    #[test]
    fn test_geometry_indices_index() {
        let mut indices = GeometryIndices::new();
        indices.push(GeometryIndex::new(1));
        indices.push(GeometryIndex::new(2));
        assert_eq!(indices[0], GeometryIndex::new(1));
        assert_eq!(indices[1], GeometryIndex::new(2));
    }

    #[test]
    fn test_geometry_indices_remove() {
        let mut indices = GeometryIndices::new();
        indices.push(GeometryIndex::new(1));
        indices.push(GeometryIndex::new(2));
        let removed = indices.remove(0);
        assert_eq!(removed, GeometryIndex::new(1));
        assert_eq!(indices.len(), 1);
        assert_eq!(indices[0], GeometryIndex::new(2));
    }

    #[test]
    fn test_geometry_indices_get_range() {
        let mut indices = GeometryIndices::new();
        indices.push(GeometryIndex::new(1));
        indices.push(GeometryIndex::new(2));
        indices.push(GeometryIndex::new(3));
        let range = indices.get_range(0..2).unwrap();
        assert_eq!(range, &[GeometryIndex::new(1), GeometryIndex::new(2)]);
    }
}
