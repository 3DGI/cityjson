use crate::errors::{Error, Result};
use num::{CheckedAdd, Unsigned};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::{AddAssign, Index as IndexOp, IndexMut, Range};
use std::convert::TryInto;
use std::num::TryFromIntError;

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

// Type aliases for common uses
pub type VertexIndex16 = VertexIndex<u16>;
pub type VertexIndex32 = VertexIndex<u32>;
pub type VertexIndex64 = VertexIndex<u64>;

pub type VertexIndices16 = VertexIndices<u16>;
pub type VertexIndices32 = VertexIndices<u32>;
pub type VertexIndices64 = VertexIndices<u64>;

/// A generic index type for vertices that can use different integer sizes.
///
/// # Examples
/// ```
/// # use cityjson::vertex::*;
/// let idx16: VertexIndex16 = VertexIndex::new(42u16);
/// let idx32: VertexIndex32 = VertexIndex::new(42u32);
/// let idx64: VertexIndex64 = VertexIndex::new(42u64);
///
/// // Convert between sizes where possible
/// let idx32_from_16: VertexIndex32 = idx16.try_into().unwrap();
/// ```
#[derive(Copy, Clone, Default, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct VertexIndex<T: VertexInteger>(T);

impl<T: VertexInteger> VertexIndex<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self(value)
    }

    #[inline]
    pub fn value(&self) -> T {
        self.0
    }
}

impl<T: VertexInteger> Display for VertexIndex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: VertexInteger> AddAssign for VertexIndex<T> {
    fn add_assign(&mut self, other: Self) {
        self.0 = self.0.checked_add(&other.0).expect("index addition overflow");
    }
}

// Specific TryFrom implementations for VertexIndex conversions
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
        u16::try_from(value.0).map(VertexIndex).map_err(|_| Error::IndexConversion {
            source_type: "u32".to_string(),
            target_type: "u16".to_string(),
            value: value.0.to_string(),
        })
    }
}

impl TryFrom<VertexIndex<u64>> for VertexIndex<u32> {
    type Error = Error;

    fn try_from(value: VertexIndex<u64>) -> Result<Self> {
        u32::try_from(value.0).map(VertexIndex).map_err(|_| Error::IndexConversion {
            source_type: "u64".to_string(),
            target_type: "u32".to_string(),
            value: value.0.to_string(),
        })
    }
}

impl TryFrom<VertexIndex<u64>> for VertexIndex<u16> {
    type Error = Error;

    fn try_from(value: VertexIndex<u64>) -> Result<Self> {
        u16::try_from(value.0).map(VertexIndex).map_err(|_| Error::IndexConversion {
            source_type: "u64".to_string(),
            target_type: "u16".to_string(),
            value: value.0.to_string(),
        })
    }
}

impl<T: VertexInteger> Vertex for VertexIndex<T> {}

impl<T: VertexInteger> Index for VertexIndex<T> {
    type Index = T;

    #[inline]
    fn index(&self) -> Self::Index {
        self.0
    }

    #[inline]
    fn to_usize(&self) -> Option<usize> {
        self.0.try_into().ok()
    }
}

/// A generic container for vertex indices that can use different integer sizes.
///
/// # Examples
/// ```
/// # use cityjson::vertex::*;
/// let mut indices32: VertexIndices32 = VertexIndices::new();
/// indices32.push(VertexIndex::new(42u32));
///
/// // Access elements
/// assert_eq!(indices32[0].value(), 42);
///
/// // Iterate over indices
/// for idx in &indices32 {
///     println!("Index: {}", idx);
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
        Self(Vec::with_capacity(
            capacity.try_into().unwrap_or(0)
        ))
    }

    #[inline]
    pub fn len(&self) -> T {
        T::try_from(self.0.len()).unwrap_or_else(|_| T::MIN)
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
        T::try_from(self.0.capacity()).unwrap_or_else(|_| T::MIN)
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
        let idx = index.try_into().ok()?;
        self.0.get(idx)
    }

    #[inline]
    pub fn get_mut(&mut self, index: T) -> Option<&mut VertexIndex<T>> {
        let idx = index.try_into().ok()?;
        self.0.get_mut(idx)
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
        let start = range.start.try_into().ok()?;
        let end = range.end.try_into().ok()?;
        self.0.get(start..end)
    }
}

impl<T: VertexInteger> IndexOp<T> for VertexIndices<T> {
    type Output = VertexIndex<T>;

    fn index(&self, index: T) -> &Self::Output {
        let idx = index.try_into().unwrap_or(0);
        &self.0[idx]
    }
}

impl<T: VertexInteger> IndexMut<T> for VertexIndices<T> {
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        let idx = index.try_into().unwrap_or(0);
        &mut self.0[idx]
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

/// Base trait for any type that can reference a vertex
pub trait Vertex: Clone + Debug {}

/// Trait for 3D vertex coordinates
pub trait Coordinate: Vertex {
    type Value: Copy + PartialOrd;

    fn x(&self) -> Self::Value;
    fn y(&self) -> Self::Value;
    fn z(&self) -> Self::Value;
}

/// Trait for vertex references
pub trait Index: Vertex {
    type Index: Copy;

    fn index(&self) -> Self::Index;

    fn to_usize(&self) -> Option<usize>;
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Vertices(Vec<VertexCoordinate>);

/// Align to 32 bytes to work well with arrow.
#[repr(C, align(32))]
#[derive(Clone, Debug)]
pub struct VertexCoordinate {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
}

impl Vertex for VertexCoordinate {}

impl Coordinate for VertexCoordinate {
    type Value = f64;

    #[inline]
    fn x(&self) -> f64 {
        self.x
    }

    #[inline]
    fn y(&self) -> f64 {
        self.y
    }

    #[inline]
    fn z(&self) -> f64 {
        self.z
    }
}

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
        let idx16 = VertexIndex16::new(42u16);

        // Convert to larger types
        let idx32: VertexIndex32 = idx16.try_into().unwrap();
        let idx64: VertexIndex64 = idx16.try_into().unwrap();

        assert_eq!(idx32.value(), 42u32);
        assert_eq!(idx64.value(), 42u64);

        // Converting to smaller type should fail if value is too large
        let large_idx = VertexIndex32::new(u32::MAX);
        let result: Result<VertexIndex16> = large_idx.try_into();
        assert!(result.is_err());
        if let Err(Error::IndexConversion { source_type, target_type, value }) = result {
            assert_eq!(source_type, "u32");
            assert_eq!(target_type, "u16");
            assert_eq!(value, u32::MAX.to_string());
        }
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
}