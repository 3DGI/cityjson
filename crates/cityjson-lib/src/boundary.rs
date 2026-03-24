pub type Coordinate = [f64; 3];

#[derive(Debug, Clone)]
pub struct Boundary {
    vertices: Vec<Coordinate>,
    rings: Vec<usize>,
    surfaces: Vec<usize>,
    shells: Vec<usize>,
    solids: Vec<usize>,
}

impl Default for Boundary {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            rings: Vec::new(),
            surfaces: Vec::new(),
            shells: Vec::new(),
            solids: Vec::new(),
        }
    }
}

impl Boundary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(
        vertices: usize,
        rings: usize,
        surfaces: usize,
        shells: usize,
        solids: usize,
    ) -> Self {
        Self {
            vertices: Vec::with_capacity(vertices),
            rings: Vec::with_capacity(rings),
            surfaces: Vec::with_capacity(surfaces),
            shells: Vec::with_capacity(shells),
            solids: Vec::with_capacity(solids),
        }
    }

    // Convert from serde_cityjson Boundary
    pub fn from_serde_boundary(
        boundary: &serde_cityjson::boundary::Boundary,
        vertices: &[Coordinate], // Reference to city model vertices
    ) -> Self {
        let mut result = Self::default();

        // Map vertex indices to actual coordinates
        result.vertices = boundary
            .vertices
            .iter()
            .map(|idx| vertices[idx.value() as usize])
            .collect();

        // Convert index arrays
        result.rings = boundary
            .rings
            .iter()
            .map(|idx| idx.value() as usize)
            .collect();

        result.surfaces = boundary
            .surfaces
            .iter()
            .map(|idx| idx.value() as usize)
            .collect();

        result.shells = boundary
            .shells
            .iter()
            .map(|idx| idx.value() as usize)
            .collect();

        result.solids = boundary
            .solids
            .iter()
            .map(|idx| idx.value() as usize)
            .collect();

        result
    }

    // Helper methods similar to serde_cityjson
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
            && self.rings.is_empty()
            && self.surfaces.is_empty()
            && self.shells.is_empty()
            && self.solids.is_empty()
    }

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

    // Verify internal consistency
    pub fn is_consistent(&self) -> bool {
        let vertices_len = self.vertices.len();
        let rings_len = self.rings.len();
        let surfaces_len = self.surfaces.len();
        let shells_len = self.shells.len();

        // Check ring indices point to valid vertices
        for window in self.rings.windows(2) {
            let start = window[0];
            let end = window[1];
            if start >= end || end > vertices_len {
                return false;
            }
        }

        // Check surface indices point to valid rings
        for window in self.surfaces.windows(2) {
            let start = window[0];
            let end = window[1];
            if start >= end || end > rings_len {
                return false;
            }
        }

        // Check shell indices point to valid surfaces
        for window in self.shells.windows(2) {
            let start = window[0];
            let end = window[1];
            if start >= end || end > surfaces_len {
                return false;
            }
        }

        // Check solid indices point to valid shells
        for window in self.solids.windows(2) {
            let start = window[0];
            let end = window[1];
            if start >= end || end > shells_len {
                return false;
            }
        }

        true
    }

    // Accessor methods
    pub fn vertices(&self) -> &[Coordinate] {
        &self.vertices
    }

    pub fn rings(&self) -> &[usize] {
        &self.rings
    }

    pub fn surfaces(&self) -> &[usize] {
        &self.surfaces
    }

    pub fn shells(&self) -> &[usize] {
        &self.shells
    }

    pub fn solids(&self) -> &[usize] {
        &self.solids
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub enum BoundaryType {
    MultiOrCompositeSolid,
    Solid,
    MultiOrCompositeSurface,
    MultiLineString,
    MultiPoint,
    #[default]
    None,
}
