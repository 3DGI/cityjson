use crate::error;
use crate::error::Error;
use crate::v2_0::vertex::{VertexIndex, VertexRef};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Abstraction over a resource identifier.
///
/// A resource identifier combines an index (position in the storage) with a generation count
/// that is incremented each time a resource slot is reused. This prevents use-after-free bugs
/// by ensuring that old references to a slot that has been reused are invalid.
pub(crate) trait ResourceId:
    Copy + Debug + Default + Display + PartialEq + Eq + PartialOrd + Ord + Hash
{
    /// Creates an instance of the resource reference with the given index and generation.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the resource in the storage
    /// * `generation` - The generation counter for the resource slot
    fn new(index: u32, generation: u16) -> Self;

    /// Returns the underlying index.
    fn index(&self) -> u32;

    /// Returns the generation.
    fn generation(&self) -> u16;

    /// Maximum index representable by this reference type.
    #[must_use]
    fn max_index() -> u32 {
        u32::MAX
    }
}

/// A 32-bit resource identifier that combines a 32-bit index with a 16-bit generation counter.
///
/// This structure allows for up to 2^32 (approximately 4.2 billion) unique resource slots,
/// and each slot can be reused up to 2^16 (65,536) times. When a slot reaches generation
/// `u16::MAX`, it is retired and will not be reused, preventing generation counter overflow.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub(crate) struct ResourceId32 {
    /// The index of the resource in the storage
    index: u32,
    /// The generation counter, incremented each time a slot is reused
    generation: u16,
}

impl ResourceId32 {
    /// Creates a new `ResourceId32` with the given index and generation.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the resource in the storage
    /// * `generation` - The generation counter for the resource slot
    #[must_use]
    pub(crate) fn new(index: u32, generation: u16) -> Self {
        Self { index, generation }
    }

    /// Returns the index part of the identifier.
    #[must_use]
    pub(crate) fn index(self) -> u32 {
        self.index
    }

    /// Returns the generation part of the identifier.
    #[must_use]
    pub(crate) fn generation(self) -> u16 {
        self.generation
    }

    /// Convert the resource index to a [`VertexIndex`].
    ///
    /// This is useful when the resource pool is storing vertices or related entities
    /// that can be referenced by vertex indices.
    ///
    /// # Arguments
    ///
    /// # Returns
    ///
    /// A Result containing the converted `VertexIndex` or an error if conversion fails
    ///
    /// # Errors
    ///
    /// Returns [`Error::IndexConversion`] when `self.index` cannot be represented by
    /// the target vertex reference type `T`.
    #[allow(dead_code)]
    pub(crate) fn to_vertex_index<T: VertexRef>(self) -> error::Result<VertexIndex<T>> {
        T::from_u32(self.index)
            .map(|v| VertexIndex::new(v))
            .ok_or(Error::IndexConversion {
                source_type: "u32".to_string(),
                target_type: std::any::type_name::<T>().to_string(),
                value: self.index.to_string(),
            })
    }
}

impl Display for ResourceId32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "index: {}, generation: {}", self.index, self.generation)
    }
}

impl ResourceId for ResourceId32 {
    fn new(index: u32, generation: u16) -> Self {
        ResourceId32 { index, generation }
    }
    fn index(&self) -> u32 {
        self.index
    }
    fn generation(&self) -> u16 {
        self.generation
    }
}

#[inline]
pub(crate) fn usize_to_resource_index<RR: ResourceId>(index: usize) -> Option<u32> {
    let index_u32 = u32::try_from(index).ok()?;
    if index_u32 <= RR::max_index() {
        Some(index_u32)
    } else {
        None
    }
}

#[test]
fn test_conversion() {
    let vi: VertexIndex<u16> = ResourceId32::new(1, 0).to_vertex_index().unwrap();
    assert_eq!(vi.value(), 1u16);
}
