use crate::Boundary;
use crate::vertex::{VertexInteger, VertexIndex, VertexIndices};

// Type aliases for u16
pub type BoundaryNestedMultiPoint16 = Vec<u16>;
pub type BoundaryNestedMultiLineString16 = Vec<BoundaryNestedMultiPoint16>;
pub type BoundaryNestedMultiOrCompositeSurface16 = Vec<BoundaryNestedMultiLineString16>;
pub type BoundaryNestedSolid16 = Vec<BoundaryNestedMultiOrCompositeSurface16>;
pub type BoundaryNestedMultiOrCompositeSolid16 = Vec<BoundaryNestedSolid16>;

// Type aliases for u32
pub type BoundaryNestedMultiPoint32 = Vec<u32>;
pub type BoundaryNestedMultiLineString32 = Vec<BoundaryNestedMultiPoint32>;
pub type BoundaryNestedMultiOrCompositeSurface32 = Vec<BoundaryNestedMultiLineString32>;
pub type BoundaryNestedSolid32 = Vec<BoundaryNestedMultiOrCompositeSurface32>;
pub type BoundaryNestedMultiOrCompositeSolid32 = Vec<BoundaryNestedSolid32>;

// Type aliases for u64
pub type BoundaryNestedMultiPoint64 = Vec<u64>;
pub type BoundaryNestedMultiLineString64 = Vec<BoundaryNestedMultiPoint64>;
pub type BoundaryNestedMultiOrCompositeSurface64 = Vec<BoundaryNestedMultiLineString64>;
pub type BoundaryNestedSolid64 = Vec<BoundaryNestedMultiOrCompositeSurface64>;
pub type BoundaryNestedMultiOrCompositeSolid64 = Vec<BoundaryNestedSolid64>;

// Generic type aliases (for use in trait implementations)
pub type BoundaryNestedMultiPoint<T> = Vec<T>;
pub type BoundaryNestedMultiLineString<T> = Vec<BoundaryNestedMultiPoint<T>>;
pub type BoundaryNestedMultiOrCompositeSurface<T> = Vec<BoundaryNestedMultiLineString<T>>;
pub type BoundaryNestedSolid<T> = Vec<BoundaryNestedMultiOrCompositeSurface<T>>;
pub type BoundaryNestedMultiOrCompositeSolid<T> = Vec<BoundaryNestedSolid<T>>;


impl<T: VertexInteger> From<BoundaryNestedMultiPoint<T>> for Boundary<T> {
    fn from(value: BoundaryNestedMultiPoint<T>) -> Self {
        if value.is_empty() {
            Self::default()
        } else {
            Self {
                vertices: value.iter().map(|v| VertexIndex::new(*v)).collect(),
                ..Self::default()
            }
        }
    }
}

impl<T: VertexInteger> From<BoundaryNestedMultiLineString<T>> for Boundary<T> {
    fn from(value: BoundaryNestedMultiLineString<T>) -> Self {
        if value.is_empty() {
            Self::default()
        } else {
            let mut vertices = VertexIndices::new();
            let mut rings = VertexIndices::with_capacity(T::try_from(value.len()).unwrap());
            let mut ring_start = VertexIndex::new(T::zero());
            for ring in &value {
                rings.push(ring_start);
                for vertex in ring {
                    vertices.push(VertexIndex::new(*vertex));
                    ring_start += VertexIndex::new(T::one());
                }
            }
            Self {
                vertices,
                rings,
                ..Self::default()
            }
        }
    }
}

impl<T: VertexInteger> From<BoundaryNestedMultiOrCompositeSurface<T>> for Boundary<T> {
    fn from(value: BoundaryNestedMultiOrCompositeSurface<T>) -> Self {
        if value.is_empty() {
            return Self::default();
        }

        let mut boundary = Self::with_capacity(
            value
                .iter()
                .map(|surface| surface.iter().map(|ring| ring.len()).sum::<usize>())
                .sum::<usize>().try_into().unwrap(),
            value.iter().map(|surface| surface.len()).sum::<usize>().try_into().unwrap(),
            value.len().try_into().unwrap(),
            T::zero(),
            T::zero(),
        );

        let mut vertex_idx = VertexIndex::new(T::zero());

        for surface in value {
            boundary
                .surfaces
                .push(VertexIndex::new(boundary.rings.len()));

            for ring in surface {
                boundary.rings.push(vertex_idx);
                for vertex in ring {
                    boundary.vertices.push(VertexIndex::new(vertex));
                    vertex_idx += VertexIndex::new(T::one());
                }
            }
        }

        boundary
    }
}

impl<T: VertexInteger> From<BoundaryNestedSolid<T>> for Boundary<T> {
    fn from(value: BoundaryNestedSolid<T>) -> Self {
        if value.is_empty() {
            return Self::default();
        }

        // Pre-calculate capacities
        let vertices_cap = value
            .iter()
            .map(|shell| {
                shell
                    .iter()
                    .map(|surface| surface.iter().map(|ring| ring.len()).sum::<usize>())
                    .sum::<usize>()
            })
            .sum::<usize>();

        let rings_cap = value
            .iter()
            .map(|shell| shell.iter().map(|surface| surface.len()).sum::<usize>())
            .sum::<usize>();

        let surfaces_cap = value.iter().map(|shell| shell.len()).sum::<usize>();

        let mut boundary = Self::with_capacity(
            vertices_cap.try_into().unwrap(),
            rings_cap.try_into().unwrap(),
            surfaces_cap.try_into().unwrap(),
            value.len().try_into().unwrap(),
            T::zero(),
        );

        let mut vertex_idx = VertexIndex::new(T::zero());

        for shell in value {
            boundary
                .shells
                .push(VertexIndex::new(boundary.surfaces.len()));

            for surface in shell {
                boundary
                    .surfaces
                    .push(VertexIndex::new(boundary.rings.len()));

                for ring in surface {
                    boundary.rings.push(vertex_idx);
                    for vertex in ring {
                        boundary.vertices.push(VertexIndex::new(vertex));
                        vertex_idx += VertexIndex::new(T::one());
                    }
                }
            }
        }

        boundary
    }
}

impl<T: VertexInteger> From<BoundaryNestedMultiOrCompositeSolid<T>> for Boundary<T> {
    fn from(value: BoundaryNestedMultiOrCompositeSolid<T>) -> Self {
        if value.is_empty() {
            return Self::default();
        }

        // Pre-calculate capacities
        let vertices_cap = value
            .iter()
            .map(|solid| {
                solid
                    .iter()
                    .map(|shell| {
                        shell
                            .iter()
                            .map(|surface| surface.iter().map(|ring| ring.len()).sum::<usize>())
                            .sum::<usize>()
                    })
                    .sum::<usize>()
            })
            .sum::<usize>();

        let rings_cap = value
            .iter()
            .map(|solid| {
                solid
                    .iter()
                    .map(|shell| shell.iter().map(|surface| surface.len()).sum::<usize>())
                    .sum::<usize>()
            })
            .sum::<usize>();

        let surfaces_cap = value
            .iter()
            .map(|solid| solid.iter().map(|shell| shell.len()).sum::<usize>())
            .sum::<usize>();

        let shells_cap = value.iter().map(|solid| solid.len()).sum::<usize>();

        let mut boundary = Self::with_capacity(
            vertices_cap.try_into().unwrap(),
            rings_cap.try_into().unwrap(),
            surfaces_cap.try_into().unwrap(),
            shells_cap.try_into().unwrap(),
            value.len().try_into().unwrap(),
        );

        let mut vertex_idx = VertexIndex::new(T::zero());

        for solid in value {
            boundary
                .solids
                .push(VertexIndex::new(boundary.shells.len()));

            for shell in solid {
                boundary
                    .shells
                    .push(VertexIndex::new(boundary.surfaces.len()));

                for surface in shell {
                    boundary
                        .surfaces
                        .push(VertexIndex::new(boundary.rings.len()));

                    for ring in surface {
                        boundary.rings.push(vertex_idx);
                        for vertex in ring {
                            boundary.vertices.push(VertexIndex::new(vertex));
                            vertex_idx += VertexIndex::new(T::one());
                        }
                    }
                }
            }
        }

        boundary
    }
}
