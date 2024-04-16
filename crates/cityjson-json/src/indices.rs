//! Coordinate and index definitions, used as boundary, semantics and appearance indices.

use std::fmt;
use std::fmt::Display;
use std::ops::{Index};
use std::slice::SliceIndex;
use derive_more::{Display, Deref, From, IntoIterator, DerefMut, AddAssign, Into};
use serde::{Deserialize, Serialize};
#[cfg(feature = "datasize")]
use datasize::DataSize;

/// A floating-point coordinate value..
pub struct CoordinateFloat(f64);

/// A signed integer coordinate value.
pub struct CoordinateInt(i64);

type LargeIndexType = u32;
type SmallIndexType = u16;

/// Index with large values.
///
/// # Examples
/// ```
/// # use serde_cityjson::indices::*;
/// # use std::ops::Deref;
/// # fn main() -> Result<(), String> {
/// let _: LargeIndex = 0u32.into();
/// let _: LargeIndex = 0usize.try_into().unwrap();
/// assert_eq!(LargeIndex::new(0), 0u32.into());
/// assert_eq!(*LargeIndex::new(0), 0u32);
/// let _ = LargeIndex::from(0u32);
/// let _ = LargeIndex::try_from(0usize).unwrap();
/// let _: usize = usize::try_from(LargeIndex::new(0)).unwrap();
/// # Ok(())
/// # }
/// ```
#[derive(AddAssign, Copy, Clone, Default, Debug, Deref, Display, From, Deserialize, Serialize, Eq, Ord, PartialOrd, PartialEq, Hash)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct LargeIndex(LargeIndexType);

impl From<&LargeIndex> for u32 {
    fn from(value: &LargeIndex) -> Self {
        value.0 as u32
    }
}

impl TryFrom<LargeIndex> for usize {
    type Error = std::num::TryFromIntError;

    fn try_from(value: LargeIndex) -> Result<Self, Self::Error> {
        usize::try_from(u32::from(&value))
    }
}

/// LargeIndex can be `u64` or `u32`.
impl TryFrom<usize> for LargeIndex {
    type Error = std::num::TryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        u32::try_from(value).map(|v| LargeIndex(v))
    }
}

impl LargeIndex {
    pub fn new(value: u32) -> Self {
        Self(value)
    }
}

/// A vector of [LargeIndex].
#[derive(Clone, Default, Debug, Deref, DerefMut, IntoIterator, Eq, Ord, PartialOrd, PartialEq, Hash)]
#[into_iterator(owned, ref)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub(crate) struct LargeIndexVec(pub(crate) Vec<LargeIndex>);

impl FromIterator<LargeIndex> for LargeIndexVec {
    fn from_iter<T: IntoIterator<Item=LargeIndex>>(iter: T) -> Self {
        let mut c = Self::new();
        for v in iter {
            c.0.push(v)
        }
        c
    }
}

impl Index<LargeIndex> for LargeIndexVec {
    type Output = LargeIndex;

    fn index(&self, index: LargeIndex) -> &Self::Output {
        &self.0[index.0 as usize]
    }
}

// impl<Idx> Index<Idx> for LargeIndexVec
// where
//     Idx: SliceIndex<[LargeIndex]>
// {
//     type Output = Idx::Output;
//
//     fn index(&self, index: Idx) -> &Self::Output {
//         &self.0[index]
//     }
// }

// impl Index<u32> for LargeIndexVec {
//     type Output = LargeIndex;
//
//     fn index(&self, index: u32) -> &Self::Output {
//         &self.0[index as usize]
//     }
// }

impl From<Vec<u32>> for LargeIndexVec {
    fn from(value: Vec<u32>) -> Self {
        LargeIndexVec(value.iter().map(|v| LargeIndex::from(*v)).collect())
    }
}

impl TryFrom<Vec<usize>> for LargeIndexVec {
    type Error = std::num::TryFromIntError;

    fn try_from(value: Vec<usize>) -> Result<Self, Self::Error> {
        let mut vec = LargeIndexVec::with_capacity(value.len());
        for v in value {
            vec.push(LargeIndex::try_from(v)?)
        }
        Ok(vec)
    }
}

impl LargeIndexVec {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }
}

#[test]
fn test_large_index_vec() {
    let v = LargeIndexVec::from(vec![0u32, 1, 2, 3,]);
    assert_eq!(v[LargeIndex::new(0)], LargeIndex::new(0));
}


#[derive(Copy, Clone, Default, Debug, Deref, From, Deserialize, Serialize, Eq, Ord, PartialOrd, PartialEq, Hash)]

pub struct OptionalLargeIndex(Option<LargeIndexType>);

#[derive(Copy, Clone, Default, Debug, Deref, Display, From, Deserialize, Serialize, Eq, Ord, PartialOrd, PartialEq, Hash)]

pub struct SmallIndex(SmallIndexType);

#[derive(Copy, Clone, Default, Debug, Deref, From, Deserialize, Serialize, Eq, Ord, PartialOrd, PartialEq, Hash)]

pub struct OptionalSmallIndex(Option<SmallIndexType>);

#[derive(Clone, Default, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]

pub(crate) struct OptionalLargeIndexVec(Vec<OptionalLargeIndex>);
#[derive(Clone, Default, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]

pub(crate) struct SmallIndexVec(Vec<SmallIndex>);
#[derive(Clone, Default, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]

pub(crate) struct OptionalSmallIndexVec(Vec<OptionalSmallIndex>);

impl Display for OptionalLargeIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref val) = self.0 {
            write!(f, "{}", val)
        } else {
            write!(f, "none")
        }
    }
}

impl Display for OptionalSmallIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref val) = self.0 {
            write!(f, "{}", val)
        } else {
            write!(f, "none")
        }
    }
}
