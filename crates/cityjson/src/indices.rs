use std::fmt;
use std::ops::{AddAssign, Index, IndexMut, Range};

/// Index type for geometry elements. Uses u32 internally to reduce memory usage while
/// maintaining compatibility with typical CityJSON datasets.
///
/// # Examples
/// ```
/// # use cityjson::indices::*;
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
/// A GeometryIndex wraps a u32 and is used for indexing geometry elements.
#[derive(Copy, Clone, Default, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct GeometryIndex(u32);

/// Type alias for optional geometry elements.
pub type OptionalGeometryIndex = Option<GeometryIndex>;

impl GeometryIndex {
    #[inline]
    pub fn new(value: u32) -> Self {
        Self(value)
    }

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

/// A generic container for items of type T, stored in a Vec<T>.
///
/// It allows up to u32::MAX elements, and all indexing and length checks are done with u32.
#[derive(Clone, Default, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct GenericIndices<T>(Vec<T>);

impl<T> GenericIndices<T> {
    /// Create a new empty container.
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Create a new container with the specified capacity (in u32).
    #[inline]
    pub fn with_capacity(capacity: u32) -> Self {
        Self(Vec::with_capacity(capacity as usize))
    }

    /// Returns the number of elements in the container (as u32).
    #[inline]
    pub fn len(&self) -> u32 {
        self.0.len().try_into().unwrap_or(u32::MAX)
    }

    /// Returns the number of elements in the container as usize (for internal Rust usage).
    #[inline]
    pub fn len_usize(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the container contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the capacity of the underlying Vec (in u32).
    #[inline]
    pub fn capacity(&self) -> u32 {
        self.0.capacity().try_into().unwrap_or(u32::MAX)
    }

    /// Reserves capacity for at least additional more elements.
    #[inline]
    pub fn reserve(&mut self, additional: u32) {
        self.0.reserve(additional as usize)
    }

    /// Reserves the minimum capacity for exactly additional more elements.
    #[inline]
    pub fn reserve_exact(&mut self, additional: u32) {
        self.0.reserve_exact(additional as usize)
    }

    /// Shrinks the capacity of the vector as much as possible.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    /// Shrinks the capacity of the vector with a lower bound.
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: u32) {
        self.0.shrink_to(min_capacity as usize)
    }

    /// Clears the container, removing all elements.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Appends an element to the back of the container.
    #[inline]
    pub fn push(&mut self, value: T) {
        self.0.push(value)
    }

    /// Removes the last element and returns it, or None if it is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }

    /// Removes and returns the element at position index (u32).
    #[inline]
    pub fn remove(&mut self, index: u32) -> T {
        self.0.remove(index as usize)
    }

    /// Returns a reference to an element at the given index (u32).
    #[inline]
    pub fn get(&self, index: u32) -> Option<&T> {
        self.0.get(index as usize)
    }

    /// Returns a reference to an element at the given index without bounds checking.
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: u32) -> &T {
        self.0.get_unchecked(index as usize)
    }

    /// Returns a mutable reference to an element at the given index (u32).
    #[inline]
    pub fn get_mut(&mut self, index: u32) -> Option<&mut T> {
        self.0.get_mut(index as usize)
    }

    /// Returns a mutable reference to an element at the given index without bounds checking.
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: u32) -> &mut T {
        self.0.get_unchecked_mut(index as usize)
    }

    /// Returns a slice containing the entire underlying vector.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }

    /// Returns a mutable slice containing the entire underlying vector.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.0.as_mut_slice()
    }

    /// Returns a reference to a contiguous subsequence, given a Range of u32 indices.
    #[inline]
    pub fn get_range(&self, range: Range<u32>) -> Option<&[T]> {
        self.0.get(range.start as usize..range.end as usize)
    }

    /// Returns a reference to a contiguous subsequence without bounds checking.
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds range is undefined behavior.
    #[inline]
    pub unsafe fn get_range_unchecked(&self, range: Range<u32>) -> &[T] {
        self.0
            .get_unchecked(range.start as usize..range.end as usize)
    }

    /// Extends the container with elements from a slice of T.
    #[inline]
    pub fn extend_from_slice(&mut self, other: &[T])
    where
        T: Clone,
    {
        self.0.extend_from_slice(other)
    }

    /// Returns an iterator over the container.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }

    /// Returns a mutable iterator over the container.
    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.0.iter_mut()
    }

    /// Returns an iterator over chunks of size chunk_size (in u32) of the container.
    #[inline]
    pub fn chunks(&self, chunk_size: u32) -> GenericIndicesChunks<'_, T> {
        GenericIndicesChunks {
            vec: self,
            chunk_size,
            index: 0,
        }
    }

    /// Returns an iterator over windows of size window_size (in u32) of the container.
    ///
    /// # Panics
    ///
    /// Panics if window_size == 0.
    #[inline]
    pub fn windows(&self, window_size: u32) -> GenericIndicesWindows<'_, T> {
        assert!(window_size > 0);
        GenericIndicesWindows {
            vec: self,
            window_size,
            index: 0,
        }
    }
}

impl<T> Index<u32> for GenericIndices<T> {
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<T> IndexMut<u32> for GenericIndices<T> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl<'a, T> IntoIterator for &'a GenericIndices<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut GenericIndices<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<T> Extend<T> for GenericIndices<T> {
    fn extend<U: IntoIterator<Item = T>>(&mut self, iter: U) {
        self.0.extend(iter);
    }
}

/// Allows collecting an iterator of T into a GenericIndices<T>.
impl<T> FromIterator<T> for GenericIndices<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

/// From<Vec<T>> is an infallible conversion if T in Vec<T> matches our T.
impl<T> From<Vec<T>> for GenericIndices<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

/// Specialization for building GeometryIndices from Vec<u32>.
impl From<Vec<u32>> for GenericIndices<GeometryIndex> {
    fn from(value: Vec<u32>) -> Self {
        Self(value.into_iter().map(GeometryIndex::new).collect())
    }
}

/// Specialization for infallibly building OptionalGeometryIndices from Vec<Option<u32>>,
/// or more commonly you might want to accept a Vec<Option<u32>>. Here we just show how
/// one might do it if needed. If not needed, you can remove or adapt this.
impl From<Vec<Option<u32>>> for GenericIndices<Option<GeometryIndex>> {
    fn from(value: Vec<Option<u32>>) -> Self {
        Self(
            value
                .into_iter()
                .map(|maybe_val| maybe_val.map(GeometryIndex::new))
                .collect(),
        )
    }
}

/// Allows building GeometryIndices from a Vec<usize> with possible overflow checks.
impl TryFrom<Vec<usize>> for GenericIndices<GeometryIndex> {
    type Error = std::num::TryFromIntError;

    fn try_from(value: Vec<usize>) -> Result<Self, Self::Error> {
        let mut vec = Self::with_capacity(value.len() as u32);
        for v in value {
            vec.push(GeometryIndex::try_from(v)?);
        }
        Ok(vec)
    }
}

/// Type alias for the "regular" geometry indices = GenericIndices<GeometryIndex>.
pub type GeometryIndices = GenericIndices<GeometryIndex>;

/// Type alias for an optional-geometry variant = GenericIndices<Option<GeometryIndex>>.
pub type OptionalGeometryIndices = GenericIndices<OptionalGeometryIndex>;

/// Iterator over chunks of a GenericIndices<T>.
pub struct GenericIndicesChunks<'a, T> {
    vec: &'a GenericIndices<T>,
    chunk_size: u32,
    index: u32,
}

impl<'a, T> Iterator for GenericIndicesChunks<'a, T> {
    type Item = &'a [T];

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

/// Iterator over windows of a GenericIndices<T>.
pub struct GenericIndicesWindows<'a, T> {
    vec: &'a GenericIndices<T>,
    window_size: u32,
    index: u32,
}

impl<'a, T> Iterator for GenericIndicesWindows<'a, T> {
    type Item = &'a [T];

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
        // Test addition overflow for GeometryIndex (will panic in debug mode)
        let mut index1 = GeometryIndex::new(u32::MAX);
        let index2 = GeometryIndex::new(1);
        index1 += index2; // Overflows in debug, wrapping in release
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
        // Test removing from an empty vector, which should panic
        let result = std::panic::catch_unwind(|| {
            let mut indices = GeometryIndices::new();
            indices.remove(0);
        });
        assert!(result.is_err()); // Should panic

        // Test getting a range that exceeds the vector length
        let mut indices = GeometryIndices::new();
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

    #[test]
    fn test_optional_geometry_indices() {
        let mut indices = OptionalGeometryIndices::new();
        indices.push(Some(GeometryIndex::new(10)));
        indices.push(None);
        indices.push(Some(GeometryIndex::new(20)));
        assert_eq!(indices.len(), 3);
        assert_eq!(indices[0], Some(GeometryIndex::new(10)));
        assert_eq!(indices[1], None);
        assert_eq!(indices[2], Some(GeometryIndex::new(20)));
    }
}