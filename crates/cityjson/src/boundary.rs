use crate::boundary_nested::*;
use crate::errors;
use crate::indices::{GeometryIndex, GeometryIndices};

/// A generic Boundary type that can represent any CityJSON boundary.
/// The Boundary does not have the Geometry type information, so it should be used in
/// conjunction with its parent Geometry.
#[repr(C)]
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[allow(unused)]
pub struct Boundary {
    /// Vertex indices that point to the global Vertices buffer.
    pub(crate) vertices: GeometryIndices,
    /// Vertex offsets that mark the start of each ring. The values point to this Boundary's vertices.
    pub(crate) rings: GeometryIndices,
    /// Ring offsets that mark the start of each surface. The values point to this Boundary's rings.
    pub(crate) surfaces: GeometryIndices,
    /// Surface offsets that mark the start of each shell. The values point to this Boundary's surfaces.
    pub(crate) shells: GeometryIndices,
    /// Shell offsets that mark the start of each solid. The values point to this Boundary's shells.
    pub(crate) solids: GeometryIndices,
}

impl Boundary {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn with_capacity(
        vertices: u32,
        rings: u32,
        surfaces: u32,
        shells: u32,
        solids: u32,
    ) -> Self {
        Self {
            vertices: GeometryIndices::with_capacity(vertices),
            rings: GeometryIndices::with_capacity(rings),
            surfaces: GeometryIndices::with_capacity(surfaces),
            shells: GeometryIndices::with_capacity(shells),
            solids: GeometryIndices::with_capacity(solids),
        }
    }

    /// Convert to a nested MultiPoint boundary representation, if the Boundary can be interpreted
    /// as a MultiPoint boundary.
    pub fn to_nested_multipoint(&self) -> errors::Result<BoundaryNestedMultiPoint> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::MultiPoint {
            Ok(self.vertices.iter().map(|v| v.value()).collect())
        } else {
            Err(errors::Error::IncompatibleBoundary(
                boundary_type.to_string(),
                "MultiPoint".to_string(),
            ))
        }
    }

    /// Convert to a nested MultiLineString boundary representation, if the Boundary can be
    /// interpreted as a MultiLineString boundary.
    pub fn to_nested_multilinestring(&self) -> errors::Result<BoundaryNestedMultiLineString> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::MultiLineString {
            let mut counter = BoundaryCounter::default();
            let mut ml = BoundaryNestedMultiLineString::with_capacity(self.rings.len_usize());
            self.push_rings_to_surface(self.rings.as_slice(), &mut ml, &mut counter);
            Ok(ml)
        } else {
            Err(errors::Error::IncompatibleBoundary(
                boundary_type.to_string(),
                "MultiLineString".to_string(),
            ))
        }
    }

    /// Convert to a nested Multi- or CompositeSurface boundary representation, if the Boundary can be
    /// interpreted as a Multi- or CompositeSurface boundary.
    pub fn to_nested_multi_or_composite_surface(
        &self,
    ) -> errors::Result<BoundaryNestedMultiOrCompositeSurface> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::MultiOrCompositeSurface {
            let mut counter = BoundaryCounter::default();
            let mut mc_surface =
                BoundaryNestedMultiOrCompositeSurface::with_capacity(self.surfaces.len_usize());
            self.push_surfaces_to_multi_surface(
                self.surfaces.as_slice(),
                &mut mc_surface,
                &mut counter,
            );
            Ok(mc_surface)
        } else {
            Err(errors::Error::IncompatibleBoundary(
                boundary_type.to_string(),
                "MultiOrCompositeSurface".to_string(),
            ))
        }
    }

    /// Convert to a nested Solid boundary representation, if the Boundary can be
    /// interpreted as a Solid boundary.
    pub fn to_nested_solid(&self) -> errors::Result<BoundaryNestedSolid> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::Solid {
            let mut counter = BoundaryCounter::default();
            let mut solid = BoundaryNestedSolid::with_capacity(self.shells.len_usize());
            self.push_shells_to_solid(self.shells.as_slice(), &mut solid, &mut counter);
            Ok(solid)
        } else {
            Err(errors::Error::IncompatibleBoundary(
                boundary_type.to_string(),
                "Solid".to_string(),
            ))
        }
    }

    /// Convert to a nested Multi- or CompositeSolid boundary representation, if the Boundary can be
    /// interpreted as a Multi- or CompositeSolid boundary.
    pub fn to_nested_multi_or_composite_solid(
        &self,
    ) -> errors::Result<BoundaryNestedMultiOrCompositeSolid> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::MultiOrCompositeSolid {
            let mut counter = BoundaryCounter::default();
            let mut mc_solid =
                BoundaryNestedMultiOrCompositeSolid::with_capacity(self.solids.len_usize());
            for shells_start_i in &self.solids {
                let shells_len = GeometryIndex::try_from(self.shells.len_usize()).unwrap();
                let shells_end_i = self
                    .solids
                    .get(counter.next_solid_i())
                    .unwrap_or(&shells_len);

                if let Some(shells) = self
                    .shells
                    .get_range(shells_start_i.value()..shells_end_i.value())
                {
                    let mut solid = BoundaryNestedSolid::with_capacity(shells.len());
                    self.push_shells_to_solid(shells, &mut solid, &mut counter);
                    mc_solid.push(solid);
                }
            }
            Ok(mc_solid)
        } else {
            Err(errors::Error::IncompatibleBoundary(
                boundary_type.to_string(),
                "MultiOrCompositeSolid".to_string(),
            ))
        }
    }

    fn push_shells_to_solid(
        &self,
        shells: &[GeometryIndex],
        solid: &mut Vec<BoundaryNestedMultiOrCompositeSurface>,
        counter: &mut BoundaryCounter,
    ) {
        for surfaces_start_i in shells {
            let surfaces_len = GeometryIndex::try_from(self.surfaces.len_usize()).unwrap();
            let surfaces_end_i = self
                .shells
                .get(counter.next_shell_i())
                .unwrap_or(&surfaces_len);

            if let Some(surfaces) = self
                .surfaces
                .get_range(surfaces_start_i.value()..surfaces_end_i.value())
            {
                let mut mc_surface =
                    BoundaryNestedMultiOrCompositeSurface::with_capacity(surfaces.len());
                self.push_surfaces_to_multi_surface(surfaces, &mut mc_surface, counter);
                solid.push(mc_surface);
            }
        }
    }

    fn push_surfaces_to_multi_surface(
        &self,
        surfaces: &[GeometryIndex],
        mc_surface: &mut BoundaryNestedMultiOrCompositeSurface,
        counter: &mut BoundaryCounter,
    ) {
        for ring_start_i in surfaces {
            let rings_len = GeometryIndex::try_from(self.rings.len_usize()).unwrap();
            let ring_end_i = self
                .surfaces
                .get(counter.next_surface_i())
                .unwrap_or(&rings_len);

            if let Some(rings) = self
                .rings
                .get_range(ring_start_i.value()..ring_end_i.value())
            {
                let mut surface = BoundaryNestedMultiLineString::with_capacity(rings.len());
                self.push_rings_to_surface(rings, &mut surface, counter);
                mc_surface.push(surface);
            }
        }
    }

    fn push_rings_to_surface(
        &self,
        rings: &[GeometryIndex],
        surface: &mut BoundaryNestedMultiLineString,
        counter: &mut BoundaryCounter,
    ) {
        for vertices_start_i in rings {
            let vertices_len = GeometryIndex::try_from(self.vertices.len_usize()).unwrap();
            let vertices_end_i = self
                .rings
                .get(counter.next_ring_i())
                .unwrap_or(&vertices_len);

            if let Some(vertices) = self
                .vertices
                .get_range(vertices_start_i.value()..vertices_end_i.value())
            {
                surface.push(vertices.iter().map(|v| v.value()).collect());
            }
        }
    }

    /// Hint what type of boundary is stored in the Boundary.
    pub fn check_type(&self) -> BoundaryType {
        if !self.solids.is_empty() {
            BoundaryType::MultiOrCompositeSolid
        } else if !self.shells.is_empty() {
            BoundaryType::Solid
        } else if !self.surfaces.is_empty() {
            BoundaryType::MultiOrCompositeSurface
        } else if !self.rings.is_empty() {
            BoundaryType::MultiLineString
        } else if !self.vertices.is_empty() {
            BoundaryType::MultiPoint
        } else {
            BoundaryType::None
        }
    }

    /// Verify that the internal representation of the boundary is consistent that there are no
    /// dangling indices.
    pub fn is_consistent(&self) -> bool {
        // Check that all indices are within bounds
        let vertices_len = self.vertices.len();
        let rings_len = self.rings.len();
        let surfaces_len = self.surfaces.len();
        let shells_len = self.shells.len();

        // Check ring indices point to valid vertices
        for (i, window) in self.rings.windows(2).enumerate() {
            let start = window[0].value();
            let end = if i == self.rings.len_usize() - 1 {
                vertices_len
            } else {
                window[1].value()
            };

            if start >= end || end > vertices_len {
                return false;
            }
        }

        // Check surface indices point to valid rings
        for (i, window) in self.surfaces.windows(2).enumerate() {
            let start = window[0].value();
            let end = if i == self.surfaces.len_usize() - 1 {
                rings_len
            } else {
                window[1].value()
            };

            if start >= end || end > rings_len {
                return false;
            }
        }

        // Check shell indices point to valid surfaces
        for (i, window) in self.shells.windows(2).enumerate() {
            let start = window[0].value();
            let end = if i == self.shells.len_usize() - 1 {
                surfaces_len
            } else {
                window[1].value()
            };

            if start >= end || end > surfaces_len {
                return false;
            }
        }

        // Check solid indices point to valid shells
        for (i, window) in self.solids.windows(2).enumerate() {
            let start = window[0].value();
            let end = if i == self.solids.len_usize() - 1 {
                shells_len
            } else {
                window[1].value()
            };

            if start >= end || end > shells_len {
                return false;
            }
        }

        true
    }
}

#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum BoundaryType {
    MultiOrCompositeSolid,
    Solid,
    MultiOrCompositeSurface,
    MultiLineString,
    MultiPoint,
    /// Represents an empty Boundary.
    #[default]
    None,
}

impl std::fmt::Display for BoundaryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BoundaryType::MultiOrCompositeSolid => "MultiOrCompositeSolid",
            BoundaryType::Solid => "Solid",
            BoundaryType::MultiOrCompositeSurface => "MultiOrCompositeSurface",
            BoundaryType::MultiLineString => "MultiLineString",
            BoundaryType::MultiPoint => "MultiPoint",
            BoundaryType::None => "None",
        };
        write!(f, "{}", s)
    }
}

#[derive(Default)]
pub(crate) struct BoundaryCounter {
    pub(crate) ring_i: u32,
    pub(crate) surface_i: u32,
    pub(crate) shell_i: u32,
    pub(crate) solid_i: u32,
}

impl BoundaryCounter {
    pub(crate) fn next_ring_i(&mut self) -> u32 {
        self.ring_i += 1;
        self.ring_i
    }

    pub(crate) fn next_surface_i(&mut self) -> u32 {
        self.surface_i += 1;
        self.surface_i
    }

    pub(crate) fn next_shell_i(&mut self) -> u32 {
        self.shell_i += 1;
        self.shell_i
    }

    pub(crate) fn next_solid_i(&mut self) -> u32 {
        self.solid_i += 1;
        self.solid_i
    }
}
