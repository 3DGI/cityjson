//! Zero-copy view types for raw data access.

/// A type-safe view over a contiguous slice.
#[derive(Debug, Clone, Copy)]
pub struct RawSliceView<'a, T> {
    data: &'a [T],
}

impl<'a, T> RawSliceView<'a, T> {
    #[inline]
    pub fn new(data: &'a [T]) -> Self {
        Self { data }
    }

    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &'a [T] {
        self.data
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&'a T> {
        self.data.get(index)
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'a, T> {
        self.into_iter()
    }
}

impl<'a, T> IntoIterator for RawSliceView<'a, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a, T> IntoIterator for &'_ RawSliceView<'a, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

/// A raw view over a resource pool.
///
/// Includes free slots (`None`) and generation counters.
#[derive(Debug, Clone, Copy)]
pub struct RawPoolView<'a, T> {
    resources: &'a [Option<T>],
    generations: &'a [u16],
}

impl<'a, T> RawPoolView<'a, T> {
    #[inline]
    pub fn new(resources: &'a [Option<T>], generations: &'a [u16]) -> Self {
        Self {
            resources,
            generations,
        }
    }

    #[inline]
    #[must_use]
    pub fn resources(&self) -> &'a [Option<T>] {
        self.resources
    }

    #[inline]
    #[must_use]
    pub fn generations(&self) -> &'a [u16] {
        self.generations
    }

    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.resources.len()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.resources.iter().filter(|r| r.is_some()).count()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter_occupied(&self) -> impl Iterator<Item = (usize, &'a T)> {
        self.resources
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_ref().map(|value| (i, value)))
    }
}

/// Columnar view of quantized coordinates.
#[derive(Debug, Clone, Copy)]
pub struct ColumnarCoordinates<'a> {
    pub x: &'a [i64],
    pub y: &'a [i64],
    pub z: &'a [i64],
}

/// Columnar view of real-world coordinates.
#[derive(Debug, Clone, Copy)]
pub struct ColumnarRealCoordinates<'a> {
    pub x: &'a [f64],
    pub y: &'a [f64],
    pub z: &'a [f64],
}
