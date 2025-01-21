use crate::indices::{GeometryIndex, GeometryIndices};
use crate::Boundary;

/// The boundary of a `MultiPoint`, `LineString` or `Ring` represented as nested vectors.
///
/// # Examples
/// ```
/// # use cityjson::boundary_nested::*;
/// # use cityjson::Boundary;
/// # use cityjson::errors;
/// # fn main() -> Result<(), errors::Error> {
/// let mp_nested: BoundaryNestedMultiPoint = vec![0, 1, 2, 3];
/// let boundary = Boundary::from(mp_nested.clone());
/// let mp_nested_rev: BoundaryNestedMultiPoint = boundary.to_nested_multi_point()?;
/// assert_eq!(mp_nested, mp_nested_rev);
/// # Ok(())
/// # }
pub type BoundaryNestedMultiPoint = Vec<u32>;

/// The boundary of a `MultiLineString`, or `Surface` represented as nested vectors.
///
/// # Examples
/// ```
/// # use cityjson::boundary_nested::*;
/// # use cityjson::Boundary;
/// # use cityjson::errors;
/// # fn main() -> Result<(), errors::Error> {
/// let ml_nested: BoundaryNestedMultiLineString = vec![vec![0, 1, 2, 3]];
/// let boundary = Boundary::from(ml_nested.clone());
/// let ml_nested_rev: BoundaryNestedMultiLineString = boundary.to_nested_multi_linestring()?;
/// assert_eq!(ml_nested, ml_nested_rev);
/// # Ok(())
/// # }
pub type BoundaryNestedMultiLineString = Vec<BoundaryNestedMultiPoint>;

/// The boundary of a `MultiSurface`, `CompositeSurface` or `Shell` represented as nested vectors.
///
/// # Examples
/// ```
/// # use cityjson::boundary_nested::*;
/// # use cityjson::Boundary;
/// # use cityjson::errors;
/// # fn main() -> Result<(), errors::Error> {
/// let aggregatesurface_nested: BoundaryNestedMultiOrCompositeSurface = vec![vec![vec![0, 1, 2, 3]]];
/// let boundary = Boundary::from(aggregatesurface_nested.clone());
/// let aggregatesurface_nested_rev: BoundaryNestedMultiOrCompositeSurface = boundary.to_nested_multi_or_composite_surface()?;
/// assert_eq!(aggregatesurface_nested, aggregatesurface_nested_rev);
/// # Ok(())
/// # }
pub type BoundaryNestedMultiOrCompositeSurface = Vec<BoundaryNestedMultiLineString>;

/// The boundary of a `Solid`, represented as nested vectors.
///
/// # Examples
/// ```
/// # use cityjson::boundary_nested::*;
/// # use cityjson::Boundary;
/// # use cityjson::errors;
/// # fn main() -> Result<(), errors::Error> {
/// let so_nested: BoundaryNestedSolid = vec![vec![vec![vec![0, 1, 2, 3]]]];
/// let boundary = Boundary::from(so_nested.clone());
/// let so_nested_rev: BoundaryNestedSolid = boundary.to_nested_solid()?;
/// assert_eq!(so_nested, so_nested_rev);
/// # Ok(())
/// # }
pub type BoundaryNestedSolid = Vec<BoundaryNestedMultiOrCompositeSurface>;

/// The boundary of a `MultiSolid` or a `CompositeSolid`, represented as nested vectors.
///
/// # Examples
/// ```
/// # use cityjson::boundary_nested::*;
/// # use cityjson::Boundary;
/// # use cityjson::errors;
/// # fn main() -> Result<(), errors::Error> {
/// let aso_nested: BoundaryNestedMultiOrCompositeSolid = vec![vec![vec![vec![vec![0, 1, 2, 3]]]]];
/// let boundary = Boundary::from(aso_nested.clone());
/// let aso_nested_rev: BoundaryNestedMultiOrCompositeSolid = boundary.to_nested_multi_or_composite_solid()?;
/// assert_eq!(aso_nested, aso_nested_rev);
/// # Ok(())
/// # }
pub type BoundaryNestedMultiOrCompositeSolid = Vec<BoundaryNestedSolid>;


impl From<BoundaryNestedMultiPoint> for Boundary {
    fn from(value: BoundaryNestedMultiPoint) -> Self {
        if value.is_empty() {
            Self::default()
        } else {
            Self {
                vertices: value.iter().map(|v| GeometryIndex::new(*v)).collect(),
                ..Self::default()
            }
        }
    }
}

impl From<BoundaryNestedMultiLineString> for Boundary {
    fn from(value: BoundaryNestedMultiLineString) -> Self {
        if value.is_empty() {
            Self::default()
        } else {
            let mut vertices = GeometryIndices::new();
            let mut rings = GeometryIndices::with_capacity(value.len() as u32);
            let mut ring_start = GeometryIndex::new(0);
            for ring in &value {
                rings.push(ring_start);
                for vertex in ring {
                    vertices.push(GeometryIndex::new(*vertex));
                    ring_start += GeometryIndex::new(1);
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

impl From<BoundaryNestedMultiOrCompositeSurface> for Boundary {
    fn from(value: BoundaryNestedMultiOrCompositeSurface) -> Self {
        if value.is_empty() {
            return Self::default();
        }

        let mut boundary = Self::with_capacity(
            value
                .iter()
                .map(|surface| surface.iter().map(|ring| ring.len()).sum::<usize>())
                .sum::<usize>() as u32,
            value.iter().map(|surface| surface.len()).sum::<usize>() as u32,
            value.len() as u32,
            0,
            0,
        );

        let mut vertex_idx = GeometryIndex::new(0);

        for surface in value {
            boundary
                .surfaces
                .push(GeometryIndex::from(boundary.rings.len()));

            for ring in surface {
                boundary.rings.push(vertex_idx);
                for vertex in ring {
                    boundary.vertices.push(GeometryIndex::new(vertex));
                    vertex_idx += GeometryIndex::new(1);
                }
            }
        }

        boundary
    }
}

impl From<BoundaryNestedSolid> for Boundary {
    fn from(value: BoundaryNestedSolid) -> Self {
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
            vertices_cap as u32,
            rings_cap as u32,
            surfaces_cap as u32,
            value.len() as u32,
            0,
        );

        let mut vertex_idx = GeometryIndex::new(0);

        for shell in value {
            boundary
                .shells
                .push(GeometryIndex::from(boundary.surfaces.len()));

            for surface in shell {
                boundary
                    .surfaces
                    .push(GeometryIndex::from(boundary.rings.len()));

                for ring in surface {
                    boundary.rings.push(vertex_idx);
                    for vertex in ring {
                        boundary.vertices.push(GeometryIndex::new(vertex));
                        vertex_idx += GeometryIndex::new(1);
                    }
                }
            }
        }

        boundary
    }
}

impl From<BoundaryNestedMultiOrCompositeSolid> for Boundary {
    fn from(value: BoundaryNestedMultiOrCompositeSolid) -> Self {
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
            vertices_cap as u32,
            rings_cap as u32,
            surfaces_cap as u32,
            shells_cap as u32,
            value.len() as u32,
        );

        let mut vertex_idx = GeometryIndex::new(0);

        for solid in value {
            boundary
                .solids
                .push(GeometryIndex::from(boundary.shells.len()));

            for shell in solid {
                boundary
                    .shells
                    .push(GeometryIndex::from(boundary.surfaces.len()));

                for surface in shell {
                    boundary
                        .surfaces
                        .push(GeometryIndex::from(boundary.rings.len()));

                    for ring in surface {
                        boundary.rings.push(vertex_idx);
                        for vertex in ring {
                            boundary.vertices.push(GeometryIndex::new(vertex));
                            vertex_idx += GeometryIndex::new(1);
                        }
                    }
                }
            }
        }

        boundary
    }
}
