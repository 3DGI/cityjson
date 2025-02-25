pub mod nested;

use crate::cityjson::geometry::boundary::nested::*;
use crate::cityjson::vertex::{VertexIndex, VertexRef};
use crate::errors;

/// A generic Boundary type that can represent any CityJSON boundary.
/// The Boundary does not have the Geometry type information, so it should be used in
/// conjunction with its parent Geometry.
#[repr(C)]
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[allow(unused)]
pub struct Boundary<VR: VertexRef> {
    /// Vertex indices that point to the global Vertices buffer.
    pub(crate) vertices: Vec<VertexIndex<VR>>,
    /// Vertex offsets that mark the start of each ring. The values point to this Boundary's vertices.
    pub(crate) rings: Vec<VertexIndex<VR>>,
    /// Ring offsets that mark the start of each surface. The values point to this Boundary's rings.
    pub(crate) surfaces: Vec<VertexIndex<VR>>,
    /// Surface offsets that mark the start of each shell. The values point to this Boundary's surfaces.
    pub(crate) shells: Vec<VertexIndex<VR>>,
    /// Shell offsets that mark the start of each solid. The values point to this Boundary's shells.
    pub(crate) solids: Vec<VertexIndex<VR>>,
}

impl<VR: VertexRef> Boundary<VR> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn with_capacity(
        vertices: VertexIndex<VR>,
        rings: VertexIndex<VR>,
        surfaces: VertexIndex<VR>,
        shells: VertexIndex<VR>,
        solids: VertexIndex<VR>,
    ) -> Self {
        Self {
            vertices: Vec::with_capacity(vertices.to_usize()),
            rings: Vec::with_capacity(rings.to_usize()),
            surfaces: Vec::with_capacity(surfaces.to_usize()),
            shells: Vec::with_capacity(shells.to_usize()),
            solids: Vec::with_capacity(solids.to_usize()),
        }
    }

    /// Convert to a nested MultiPoint boundary representation, if the Boundary can be interpreted
    /// as a MultiPoint boundary.
    pub fn to_nested_multi_point(&self) -> errors::Result<BoundaryNestedMultiPoint<VR>> {
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
    pub fn to_nested_multi_linestring(&self) -> errors::Result<BoundaryNestedMultiLineString<VR>> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::MultiLineString {
            let mut counter = BoundaryCounter::<VR>::default();
            let mut ml = BoundaryNestedMultiLineString::with_capacity(self.rings.len());
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
    ) -> errors::Result<BoundaryNestedMultiOrCompositeSurface<VR>> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::MultiOrCompositeSurface {
            let mut counter = BoundaryCounter::<VR>::default();
            let mut mc_surface =
                BoundaryNestedMultiOrCompositeSurface::with_capacity(self.surfaces.len());
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
    pub fn to_nested_solid(&self) -> errors::Result<BoundaryNestedSolid<VR>> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::Solid {
            let mut counter = BoundaryCounter::<VR>::default();
            let mut solid = BoundaryNestedSolid::with_capacity(self.shells.len());
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
    ) -> errors::Result<BoundaryNestedMultiOrCompositeSolid<VR>> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::MultiOrCompositeSolid {
            let mut counter = BoundaryCounter::<VR>::default();
            let mut mc_solid =
                BoundaryNestedMultiOrCompositeSolid::with_capacity(self.solids.len());
            for &shells_start_i in &self.solids {
                let shells_len = VertexIndex::<VR>::try_from(self.shells.len())?;
                let shells_end_i = self
                    .solids
                    .get(counter.increment_solid_idx().to_usize())
                    .copied()
                    .unwrap_or(shells_len);

                if let Some(shells) = self
                    .shells
                    .get(shells_start_i.to_usize()..shells_end_i.to_usize())
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
        shells: &[VertexIndex<VR>],
        solid: &mut Vec<BoundaryNestedMultiOrCompositeSurface<VR>>,
        counter: &mut BoundaryCounter<VR>,
    ) {
        for &surfaces_start_i in shells {
            let surfaces_len = VertexIndex::<VR>::try_from(self.surfaces.len()).unwrap();
            let surfaces_end_i = self
                .shells
                .get(counter.increment_shell_idx().to_usize())
                .copied()
                .unwrap_or(surfaces_len);

            if let Some(surfaces) = self
                .surfaces
                .get(surfaces_start_i.to_usize()..surfaces_end_i.to_usize())
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
        surfaces: &[VertexIndex<VR>],
        mc_surface: &mut BoundaryNestedMultiOrCompositeSurface<VR>,
        counter: &mut BoundaryCounter<VR>,
    ) {
        for &ring_start_i in surfaces {
            let rings_len = VertexIndex::<VR>::try_from(self.rings.len()).unwrap();
            let ring_end_i = self
                .surfaces
                .get(counter.increment_surface_idx().to_usize())
                .copied()
                .unwrap_or(rings_len);

            if let Some(rings) = self
                .rings
                .get(ring_start_i.to_usize()..ring_end_i.to_usize())
            {
                let mut surface = BoundaryNestedMultiLineString::with_capacity(rings.len());
                self.push_rings_to_surface(rings, &mut surface, counter);
                mc_surface.push(surface);
            }
        }
    }

    fn push_rings_to_surface(
        &self,
        rings: &[VertexIndex<VR>],
        surface: &mut BoundaryNestedMultiLineString<VR>,
        counter: &mut BoundaryCounter<VR>,
    ) {
        for &vertices_start_i in rings {
            let vertices_len = VertexIndex::<VR>::try_from(self.vertices.len()).unwrap();
            let vertices_end_i = self
                .rings
                .get(counter.increment_ring_idx().to_usize())
                .copied()
                .unwrap_or(vertices_len);
            if let Some(vertices) = self
                .vertices
                .get(vertices_start_i.to_usize()..vertices_end_i.to_usize())
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
        for window in self.rings.windows(2) {
            let start = window[0].to_usize();
            let end = window[1].to_usize();

            if start >= end || end > vertices_len {
                return false;
            }
        }

        // Check surface indices point to valid rings
        for window in self.surfaces.windows(2) {
            let start = window[0].to_usize();
            let end = window[1].to_usize();

            if start >= end || end > rings_len {
                return false;
            }
        }

        // Check shell indices point to valid surfaces
        for window in self.shells.windows(2) {
            let start = window[0].to_usize();
            let end = window[1].to_usize();

            if start >= end || end > surfaces_len {
                return false;
            }
        }

        // Check solid indices point to valid shells
        for window in self.solids.windows(2) {
            let start = window[0].to_usize();
            let end = window[1].to_usize();

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
pub(crate) struct BoundaryCounter<VR: VertexRef> {
    pub(crate) vertex_offset: VertexIndex<VR>, // Current position in vertex list
    pub(crate) ring_offset: VertexIndex<VR>,   // Current position in ring list
    pub(crate) surface_offset: VertexIndex<VR>, // Current position in surface list
    pub(crate) shell_offset: VertexIndex<VR>,  // Current position in shell list
    pub(crate) solid_offset: VertexIndex<VR>,  // Current position in solid list
}

impl<VR: VertexRef> BoundaryCounter<VR> {
    // Increment methods - return new position after incrementing
    pub(crate) fn increment_vertex_idx(&mut self) -> VertexIndex<VR> {
        self.vertex_offset += VertexIndex::new(VR::one());
        self.vertex_offset
    }

    pub(crate) fn increment_ring_idx(&mut self) -> VertexIndex<VR> {
        self.ring_offset += VertexIndex::new(VR::one());
        self.ring_offset
    }

    pub(crate) fn increment_surface_idx(&mut self) -> VertexIndex<VR> {
        self.surface_offset += VertexIndex::new(VR::one());
        self.surface_offset
    }

    pub(crate) fn increment_shell_idx(&mut self) -> VertexIndex<VR> {
        self.shell_offset += VertexIndex::new(VR::one());
        self.shell_offset
    }

    pub(crate) fn increment_solid_idx(&mut self) -> VertexIndex<VR> {
        self.solid_offset += VertexIndex::new(VR::one());
        self.solid_offset
    }

    // Get current offsets without incrementing
    pub(crate) fn vertex_offset(&self) -> VertexIndex<VR> {
        self.vertex_offset
    }

    pub(crate) fn ring_offset(&self) -> VertexIndex<VR> {
        self.ring_offset
    }

    pub(crate) fn surface_offset(&self) -> VertexIndex<VR> {
        self.surface_offset
    }

    pub(crate) fn shell_offset(&self) -> VertexIndex<VR> {
        self.shell_offset
    }

    pub(crate) fn solid_offset(&self) -> VertexIndex<VR> {
        self.solid_offset
    }
}

// Type aliases for convenience
pub type Boundary16 = Boundary<u16>;
pub type Boundary32 = Boundary<u32>;
pub type Boundary64 = Boundary<u64>;

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::index::VertexIndexVec;
//
//     #[test]
//     fn multipoint() {
//         let boundary = Boundary {
//             vertices: vec![0u32, 3, 2, 1].to_vertex_indices(),
//             ..Default::default()
//         };
//         let mp_nested = boundary.to_nested_multi_point().unwrap();
//         assert_eq!(mp_nested, vec![0, 3, 2, 1]);
//     }
//
//     #[test]
//     fn multilinestring_basic() {
//         let boundary = Boundary {
//             vertices: vec![0u32, 3, 2, 1, 4, 5, 6, 7, 8].to_vertex_indices(),
//             rings: vec![0u32, 4, 7].to_vertex_indices(),
//             ..Default::default()
//         };
//         let nested = boundary.to_nested_multi_linestring().unwrap();
//         assert_eq!(nested, vec![vec![0, 3, 2, 1], vec![4, 5, 6], vec![7, 8]]);
//     }
//
//     #[test]
//     fn multilinestring_empty() {
//         let boundary = Boundary {
//             vertices: vec![0u32, 3, 2, 1, 4, 5, 6, 7].to_vertex_indices(),
//             rings: vec![0u32, 4, 4, 8].to_vertex_indices(),
//             ..Default::default()
//         };
//         let nested = boundary.to_nested_multi_linestring().unwrap();
//         assert_eq!(
//             nested,
//             vec![vec![0, 3, 2, 1], vec![], vec![4, 5, 6, 7], vec![]]
//         );
//     }
//
//     #[test]
//     fn from_multilinestring_empty_last() {
//         let ml_nested: BoundaryNestedMultiLineString<u32> = vec![vec![0, 1, 2, 3], vec![]];
//         let boundary = Boundary::from(ml_nested);
//         assert_eq!(boundary.rings, vec![0u32, 4].to_vertex_indices())
//     }
//
//     #[test]
//     fn from_multilinestring_empty_inner() {
//         let ml_nested: BoundaryNestedMultiLineString<u32> =
//             vec![vec![0, 1, 2, 3], vec![], vec![0, 1, 2, 3], vec![0, 1, 2, 3]];
//         let boundary = Boundary::from(ml_nested);
//         assert_eq!(boundary.rings, vec![0u32, 4, 4, 8].to_vertex_indices())
//     }
//
//     #[test]
//     fn multi_or_composite_surface_inner_ring() {
//         let boundary = Boundary {
//             vertices: vec![
//                 0u32, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
//             ]
//             .to_vertex_indices(),
//             rings: vec![0u32, 4, 8, 12, 16, 19].to_vertex_indices(),
//             surfaces: vec![0u32, 3, 4].to_vertex_indices(),
//             ..Default::default()
//         };
//         let nested = boundary.to_nested_multi_or_composite_surface().unwrap();
//         assert_eq!(
//             nested,
//             vec![
//                 vec![vec![0, 1, 2, 3], vec![4, 5, 6, 7], vec![8, 9, 10, 11]],
//                 vec![vec![12, 13, 14, 15]],
//                 vec![vec![16, 17, 18], vec![19, 20, 21, 22]]
//             ]
//         );
//     }
//
//     #[test]
//     fn solid() {
//         let boundary = Boundary {
//             vertices: vec![
//                 0u32, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
//             ]
//             .to_vertex_indices(),
//             rings: vec![0u32, 4, 8, 12, 16, 19].to_vertex_indices(),
//             surfaces: vec![0u32, 3, 4].to_vertex_indices(),
//             shells: vec![0u32, 2].to_vertex_indices(),
//             ..Default::default()
//         };
//         let nested = boundary.to_nested_solid().unwrap();
//         assert_eq!(
//             nested,
//             vec![
//                 vec![
//                     vec![vec![0, 1, 2, 3], vec![4, 5, 6, 7], vec![8, 9, 10, 11]],
//                     vec![vec![12, 13, 14, 15]]
//                 ],
//                 vec![vec![vec![16, 17, 18], vec![19, 20, 21, 22]]]
//             ]
//         );
//     }
//
//     #[test]
//     fn multi_or_composite_solid() {
//         let boundary = Boundary {
//             vertices: vec![
//                 0u32, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
//                 22, 23, 24, 25, 26, 27, 28,
//             ]
//             .to_vertex_indices(),
//             rings: vec![0u32, 4, 8, 12, 16, 19, 23, 26].to_vertex_indices(),
//             surfaces: vec![0u32, 3, 4, 6, 7].to_vertex_indices(),
//             shells: vec![0u32, 2, 3].to_vertex_indices(),
//             solids: vec![0u32, 2].to_vertex_indices(),
//         };
//         let nested = boundary.to_nested_multi_or_composite_solid().unwrap();
//         assert_eq!(
//             nested,
//             vec![
//                 vec![
//                     vec![
//                         vec![vec![0, 1, 2, 3], vec![4, 5, 6, 7], vec![8, 9, 10, 11]],
//                         vec![vec![12, 13, 14, 15]]
//                     ],
//                     vec![vec![vec![16, 17, 18], vec![19, 20, 21, 22]]]
//                 ],
//                 vec![vec![vec![vec![23, 24, 25]], vec![vec![26, 27, 28]]]]
//             ]
//         );
//     }
// }
