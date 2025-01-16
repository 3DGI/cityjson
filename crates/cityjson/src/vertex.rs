use std::fmt::Debug;

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

#[derive(Clone, Debug)]
pub struct VertexIndex(u32);

impl Vertex for VertexIndex {}

impl Index for VertexIndex {
    type Index = u32;

    #[inline]
    fn index(&self) -> u32 {
        self.0
    }

    #[inline]
    fn to_usize(&self) -> Option<usize> {
        usize::try_from(self.0).ok()
    }
}

#[test]
fn test_vertex_coordinate() {
    let p = VertexCoordinate {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    assert_eq!(p.x(), 1.0);
    assert_eq!(p.y(), 2.0);
    assert_eq!(p.z(), 3.0);
}

#[test]
fn test_vertex_index() {
    let idx = VertexIndex(42);
    assert_eq!(idx.index(), 42);
}