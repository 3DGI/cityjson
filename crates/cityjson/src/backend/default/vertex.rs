//! Vertex index types for `CityJSON` geometries.
//!
//! Supports `u16`, `u32`, and `u64` index widths. Widening conversions always succeed;
//! narrowing conversions use `try_into` and may fail.
//! Converting an index to `usize` may fail on narrower targets when the chosen
//! index width exceeds the platform pointer width.

use crate::error::{Error, Result};
use num::{CheckedAdd, FromPrimitive, Unsigned};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::num::TryFromIntError;
use std::ops::AddAssign;

//------------------------------------------------------------------------------
// Core integer trait and implementations
//------------------------------------------------------------------------------

/// Unsigned integer type used for vertex indexing (`u16`, `u32`, or `u64`).
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

/// A 16-bit vertex index (up to 65,535 vertices)
pub type VertexIndex16 = VertexIndex<u16>;

/// A 32-bit vertex index (up to 4,294,967,295 vertices)
pub type VertexIndex32 = VertexIndex<u32>;

/// A 64-bit vertex index (virtually unlimited vertices)
pub type VertexIndex64 = VertexIndex<u64>;

/// Typed vertex index. `#[repr(transparent)]` over the underlying integer.
///
/// `+=` saturates on overflow; use [`VertexIndex::checked_add`] when overflow must be detected.
///
/// ```
/// # use cityjson::v2_0::VertexIndex16;
/// let mut idx = VertexIndex16::new(10);
/// idx += VertexIndex16::new(5);
/// assert_eq!(idx.value(), 15);
///
/// let mut max_idx = VertexIndex16::new(u16::MAX);
/// max_idx += VertexIndex16::new(1);
/// assert_eq!(max_idx.value(), u16::MAX); // saturated
///
/// let mut checked = VertexIndex16::new(u16::MAX);
/// assert!(checked.try_add_assign(VertexIndex16::new(1)).is_err());
/// ```
#[derive(Copy, Clone, Default, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
#[repr(transparent)]
pub struct VertexIndex<T: VertexRef>(T);

impl<T: VertexRef> VertexIndex<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self(value)
    }

    #[inline]
    pub fn value(&self) -> T {
        self.0
    }

    /// Converts this index to `usize`.
    ///
    /// # Errors
    ///
    /// Returns an error if the index value does not fit in `usize` on the current target.
    #[inline]
    pub fn try_to_usize(&self) -> Result<usize> {
        self.0.try_into().map_err(|_| Error::IndexConversion {
            source_type: std::any::type_name::<T>().to_string(),
            target_type: "usize".to_string(),
            value: self.0.to_string(),
        })
    }

    /// Converts this index to `usize`.
    ///
    /// This is infallible for the crate's default `u32` model configuration.
    /// For wider index types on narrower platforms, prefer
    /// [`VertexIndex::try_to_usize`] and handle overflow explicitly.
    ///
    /// # Panics
    ///
    /// Panics if the vertex index does not fit in `usize` on the current target.
    /// Use [`VertexIndex::try_to_usize`] to handle this fallibly.
    #[inline]
    pub fn to_usize(&self) -> usize {
        self.try_to_usize().unwrap_or_else(|_| {
            panic!(
                "vertex index {} does not fit in usize on this target; use try_to_usize()",
                self.0
            )
        })
    }

    /// Constructs from a `u32` value, returning `None` if the value doesn't fit in `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cityjson::v2_0::{VertexIndex16, VertexIndex32, VertexIndex64};
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
    #[inline]
    #[must_use]
    pub fn from_u32(value: u32) -> Option<Self> {
        T::from_u32(value).map(|v| Self::new(v))
    }

    /// Returns true if this index is at the maximum value for its type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cityjson::v2_0::VertexIndex16;
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
    /// # Examples
    ///
    /// ```
    /// # use cityjson::v2_0::VertexIndex32;
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

    /// Returns the next index, or `None` on overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cityjson::v2_0::VertexIndex16;
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

    /// Returns `self + other`, or `None` on overflow.
    #[inline]
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(&other.0).map(Self::new)
    }

    /// Adds `other` to this index and reports overflow as a typed error.
    ///
    /// # Errors
    ///
    /// Returns [`Error::IndexOverflow`] if the addition would overflow.
    #[inline]
    pub fn try_add_assign(&mut self, other: Self) -> Result<()> {
        let sum = self
            .checked_add(other)
            .ok_or_else(|| Error::IndexOverflow {
                index_type: std::any::type_name::<T>().to_string(),
                value: self.value().to_string(),
            })?;
        *self = sum;
        Ok(())
    }
}

impl<T: VertexRef> Display for VertexIndex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: VertexRef> AddAssign for VertexIndex<T> {
    fn add_assign(&mut self, other: Self) {
        *self = self
            .checked_add(other)
            .unwrap_or_else(|| VertexIndex::new(T::MAX));
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

/// Creates a sequence of consecutive vertex indices.
pub trait VertexIndicesSequence<T>
where
    T: VertexRef,
{
    /// Returns a vector of `count` sequential indices starting at `start`.
    ///
    /// # Errors
    ///
    /// Returns an index-conversion error when the sequence would overflow `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use cityjson::v2_0::{VertexIndex16, VertexIndicesSequence};
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
                    source_type: format!("{current} + 1"),
                    target_type: std::any::type_name::<T>().to_string(),
                    value: "overflow".to_string(),
                });
            }
        }

        Ok(result)
    }
}

pub struct RawVertexView<'a, VR: VertexRef>(pub(crate) &'a [VertexIndex<VR>]);

impl<VR: VertexRef> std::ops::Deref for RawVertexView<'_, VR> {
    type Target = [VR];

    fn deref(&self) -> &Self::Target {
        const {
            assert!(std::mem::size_of::<VertexIndex<VR>>() == std::mem::size_of::<VR>());
            assert!(std::mem::align_of::<VertexIndex<VR>>() == std::mem::align_of::<VR>());
        }

        // SAFETY: `VertexIndex<VR>` is `#[repr(transparent)]` over `VR`, so a slice of
        // `VertexIndex<VR>` has identical layout to a slice of `VR`.
        unsafe { std::slice::from_raw_parts(self.0.as_ptr().cast::<VR>(), self.0.len()) }
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
/// use cityjson::v2_0::{VertexIndex16, VertexIndexVec};
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

#[cfg(test)]
mod tests {
    use super::VertexIndex32;
    #[cfg(target_pointer_width = "32")]
    use super::VertexIndex64;

    #[test]
    fn u32_index_converts_to_usize() {
        let index = VertexIndex32::new(42);
        assert_eq!(index.try_to_usize().unwrap(), 42);
        assert_eq!(index.to_usize(), 42);
    }

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn u64_index_overflow_is_reported_on_32_bit_targets() {
        let index = VertexIndex64::new(u64::from(u32::MAX) + 1);
        let error = index.try_to_usize().unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "failed to convert index from {} to usize: value {}",
                std::any::type_name::<u64>(),
                u64::from(u32::MAX) + 1
            )
        );
    }
}
