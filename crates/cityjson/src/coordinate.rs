use crate::errors::{Error, Result};
use crate::vertex::VertexInteger;
use crate::VertexIndex;
use std::marker::PhantomData;

/// 3D vertex coordinate
#[repr(C, align(32))]
#[derive(Clone, Debug)]
pub struct VertexCoordinate {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
}

impl VertexCoordinate {
    #[inline]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[inline]
    pub fn y(&self) -> f64 {
        self.y
    }

    #[inline]
    pub fn z(&self) -> f64 {
        self.z
    }
}

/// Container for vertex coordinates with size limited by the chosen index type.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Vertices<T: VertexInteger> {
    coordinates: Vec<VertexCoordinate>,
    _phantom: PhantomData<T>,
}

impl<T: VertexInteger> Vertices<T> {
    /// Creates a new empty Vertices collection
    #[inline]
    pub fn new() -> Self {
        Self {
            coordinates: Vec::new(),
            _phantom: PhantomData::default(),
        }
    }

    /// Adds a new coordinate to the collection
    pub fn push(&mut self, coordinate: VertexCoordinate) -> Result<VertexIndex<T>> {
        if self.coordinates.len() >= T::MAX.try_into().unwrap_or(usize::MAX) {
            return Err(Error::TooManyVertices {
                attempted: self.coordinates.len() + 1,
                maximum: T::MAX.try_into().unwrap_or(usize::MAX),
            });
        }
        let index = VertexIndex::<T>::try_from(self.coordinates.len())?;
        self.coordinates.push(coordinate);
        Ok(index)
    }

    /// Returns a reference to the coordinate at the specified index
    #[inline]
    pub fn get(&self, index: VertexIndex<T>) -> Option<&VertexCoordinate> {
        self.coordinates.get(index.to_usize())
    }

    /// Returns true if the collection is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.coordinates.is_empty()
    }

    /// Returns a slice of all coordinates
    #[inline]
    pub fn as_slice(&self) -> &[VertexCoordinate] {
        &self.coordinates
    }
}

// Type aliases for convenience
pub type Vertices16 = Vertices<u16>;
pub type Vertices32 = Vertices<u32>;
pub type Vertices64 = Vertices<u64>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertices16_limit() {
        let mut vertices = Vertices16::new();

        // Add vertices and get valid indices
        for i in 0..5 {
            let _ = vertices
                .push(VertexCoordinate {
                    x: i as f64,
                    y: 0.0,
                    z: 0.0,
                })
                .unwrap();
        }

        // Fill up to u16::MAX
        for _ in 5..u16::MAX as usize {
            vertices
                .push(VertexCoordinate {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                })
                .unwrap();
        }

        // One more should fail
        let result = vertices.push(VertexCoordinate {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_vertices_indexing() {
        let mut vertices = Vertices16::new();
        let idx = vertices
            .push(VertexCoordinate {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            })
            .unwrap();

        let coord = vertices.get(idx).unwrap();
        assert_eq!(coord.x(), 1.0);
        assert_eq!(coord.y(), 2.0);
        assert_eq!(coord.z(), 3.0);
    }
}
