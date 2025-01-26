use crate::common::index::{VertexIndex, VertexInteger};
use crate::errors::{Error, Result};
use std::marker::PhantomData;

pub trait Vertex {}

/// 3D vertex coordinate
#[repr(C, align(32))]
#[derive(Clone, Debug)]
pub struct RealWorldCoordinate {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
}

impl RealWorldCoordinate {
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

impl Vertex for RealWorldCoordinate {}

#[repr(C, align(32))]
#[derive(Clone, Debug)]
pub struct UVCoordinate {
    pub(crate) u: f32,
    pub(crate) v: f32,
}

impl UVCoordinate {
    #[inline]
    pub fn u(&self) -> f32 {
        self.u
    }

    #[inline]
    pub fn v(&self) -> f32 {
        self.v
    }
}

impl Vertex for UVCoordinate {}

/// Container for vertex coordinates with size limited by the chosen index type.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Vertices<VI: VertexInteger, V: Vertex> {
    coordinates: Vec<V>,
    _phantom: PhantomData<VI>,
}

impl<VI: VertexInteger, V: Vertex> Vertices<VI, V> {
    /// Creates a new empty Vertices collection
    #[inline]
    pub fn new() -> Self {
        Self {
            coordinates: Vec::new(),
            _phantom: PhantomData::default(),
        }
    }

    /// Adds a new coordinate to the collection
    pub fn push(&mut self, coordinate: V) -> Result<VertexIndex<VI>> {
        if self.coordinates.len() >= VI::MAX.try_into().unwrap_or(usize::MAX) {
            return Err(Error::TooManyVertices {
                attempted: self.coordinates.len() + 1,
                maximum: VI::MAX.try_into().unwrap_or(usize::MAX),
            });
        }
        let index = VertexIndex::<VI>::try_from(self.coordinates.len())?;
        self.coordinates.push(coordinate);
        Ok(index)
    }

    /// Returns a reference to the coordinate at the specified index
    #[inline]
    pub fn get(&self, index: VertexIndex<VI>) -> Option<&V> {
        self.coordinates.get(index.to_usize())
    }

    /// Returns true if the collection is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.coordinates.is_empty()
    }

    /// Returns a slice of all coordinates
    #[inline]
    pub fn as_slice(&self) -> &[V] {
        &self.coordinates
    }
}

// Type aliases for convenience
pub type GeometryVertices16 = Vertices<u16, RealWorldCoordinate>;
pub type GeometryVertices32 = Vertices<u32, RealWorldCoordinate>;
pub type GeometryVertices64 = Vertices<u64, RealWorldCoordinate>;

pub type UVVertices16 = Vertices<u16, UVCoordinate>;
pub type UVVertices32 = Vertices<u32, UVCoordinate>;
pub type UVVertices64 = Vertices<u64, UVCoordinate>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertices16_limit() {
        let mut vertices = GeometryVertices16::new();

        // Add vertices and get valid indices
        for i in 0..5 {
            let _ = vertices
                .push(RealWorldCoordinate {
                    x: i as f64,
                    y: 0.0,
                    z: 0.0,
                })
                .unwrap();
        }

        // Fill up to u16::MAX
        for _ in 5..u16::MAX as usize {
            vertices
                .push(RealWorldCoordinate {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                })
                .unwrap();
        }

        // One more should fail
        let result = vertices.push(RealWorldCoordinate {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_vertices_indexing() {
        let mut vertices = GeometryVertices16::new();
        let idx = vertices
            .push(RealWorldCoordinate {
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
