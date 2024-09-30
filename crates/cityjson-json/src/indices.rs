//! Coordinate and index definitions, used as boundary, semantics and appearance indices.

#[cfg(feature = "datasize")]
use datasize::DataSize;
use derive_more::{AddAssign, Deref, DerefMut, Display, From, IntoIterator};
use serde::{Deserialize, Serialize};
use std::ops::Index;

type LargeIndexType = u32;

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
#[derive(
    AddAssign,
    Copy,
    Clone,
    Default,
    Debug,
    Deref,
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
pub struct LargeIndex(LargeIndexType);

impl From<&LargeIndex> for u32 {
    fn from(value: &LargeIndex) -> Self {
        value.0
    }
}

impl TryFrom<LargeIndex> for usize {
    type Error = std::num::TryFromIntError;

    fn try_from(value: LargeIndex) -> Result<Self, Self::Error> {
        usize::try_from(u32::from(&value))
    }
}

impl TryFrom<usize> for LargeIndex {
    type Error = std::num::TryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        u32::try_from(value).map(LargeIndex)
    }
}

impl LargeIndex {
    pub fn new(value: u32) -> Self {
        Self(value)
    }
}

/// A vector of [LargeIndex].
#[derive(
    Clone,
    Default,
    Debug,
    Deref,
    DerefMut,
    IntoIterator,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
    Hash,
    Deserialize,
    Serialize,
)]
#[into_iterator(owned, ref)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct LargeIndexVec(pub(crate) Vec<LargeIndex>);

impl FromIterator<LargeIndex> for LargeIndexVec {
    fn from_iter<T: IntoIterator<Item = LargeIndex>>(iter: T) -> Self {
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
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    // pub(crate) fn get(&self, idx: Range<LargeIndex>) -> Option<&[LargeIndex]> {
    //     self.0.get(idx.start.0 as usize..idx.end.0 as usize)
    // }
}

pub type OptionalLargeIndex = Option<LargeIndex>;

// #[derive(
//     Copy,
//     Clone,
//     Default,
//     Debug,
//     Deref,
//     From,
//     Deserialize,
//     Serialize,
//     Eq,
//     Ord,
//     PartialOrd,
//     PartialEq,
//     Hash,
// )]
// #[cfg_attr(feature = "datasize", derive(DataSize))]
// pub struct OptionalLargeIndex(Option<LargeIndexType>);
//
// impl Display for OptionalLargeIndex {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         if let Some(ref val) = self.0 {
//             write!(f, "{}", val)
//         } else {
//             write!(f, "none")
//         }
//     }
// }
