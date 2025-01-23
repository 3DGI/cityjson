use crate::errors::{Error, Result};
use crate::resource_pool::ResourcePool;
use crate::v1_1::semantics::{Semantic, SemanticType};
use crate::vertex::{OptionalVertexIndices, VertexIndices, VertexInteger};
use crate::{
    Boundary, GenericCityModel, GeometryType, LoD, SemanticMaterialMap, VertexCoordinate,
    VertexIndex,
};

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Geometry<T: VertexInteger> {
    type_geometry: GeometryType,
    lod: Option<LoD>,
    boundaries: Option<Boundary<T>>,
    semantics: Option<SemanticMaterialMap<T>>,
    template_boundaries: Option<usize>,
    template_transformation_matrix: Option<[f64; 16]>,
}

/// Represents a surface under construction with one outer ring and optional inner rings
#[derive(Default)]
struct SurfaceInProgress {
    outer_ring: Option<usize>,      // index to outer ring
    inner_rings: Vec<usize>,        // indices to inner rings
    semantic: Option<SemanticType>, // semantic type for the whole surface
}

#[derive(Default)]
struct ShellInProgress {
    outer_surfaces: Vec<usize>, // indices to outer surfaces
    inner_surfaces: Vec<usize>, // indices to inner surfaces (voids)
}

#[derive(Default)]
struct SolidInProgress {
    outer_shell: Option<usize>, // index to outer shell
    inner_shells: Vec<usize>,   // indices to inner shells (voids)
}

pub struct GeometryBuilder<'a, T: VertexInteger, P: ResourcePool<Semantic<T>>> {
    model: &'a mut GenericCityModel<T, P>,
    type_geometry: GeometryType,
    lod: Option<LoD>,
    vertices: Vec<VertexCoordinate>,
    rings: Vec<Vec<usize>>,           // indices into vertices
    surfaces: Vec<SurfaceInProgress>, // surfaces with their rings
    shells: Vec<ShellInProgress>,     // shells with their surfaces
    solids: Vec<SolidInProgress>,     // solids with their shells
    current_surface: Option<usize>,   // current surface being built
    current_shell: Option<usize>,     // current shell being built
    current_solid: Option<usize>,     // current solid being built
}

impl<'a, T: VertexInteger, P: ResourcePool<Semantic<T>>> GeometryBuilder<'a, T, P> {
    pub fn new(model: &'a mut GenericCityModel<T, P>, type_geometry: GeometryType) -> Self {
        Self {
            model,
            type_geometry,
            lod: None,
            vertices: Vec::new(),
            rings: Vec::new(),
            surfaces: Vec::new(),
            shells: Vec::new(),
            solids: Vec::new(),
            current_surface: None,
            current_shell: None,
            current_solid: None,
        }
    }

    /// Set the Level of Detail on the Geometry.
    pub fn with_lod(mut self, lod: LoD) -> Self {
        self.lod = Some(lod);
        self
    }

    /// Adds a new vertex to the CityModel
    ///
    /// Returns the index of the new vertex.
    pub fn add_vertex(&mut self, x: f64, y: f64, z: f64) -> usize {
        self.vertices.push(VertexCoordinate { x, y, z });
        self.vertices.len() - 1
    }

    /// Adds a new ring to the geometry.
    ///
    /// # Errors
    ///
    /// Returns `InvalidRing` if:
    /// - The ring has fewer than 3 vertices
    /// - The vertices don't form a valid ring (first != last)
    /// - Any vertex index is out of bounds
    pub fn add_ring(&mut self, vertices: &[usize]) -> Result<usize> {
        if vertices.len() < 3 {
            return Err(Error::InvalidRing {
                reason: "Ring must have at least 3 vertices".to_string(),
                vertex_count: vertices.len(),
            });
        }
        self.rings.push(vertices.to_vec());
        Ok(self.rings.len() - 1)
    }

    /// Starts a new surface with an optional semantic type.
    ///
    /// Returns the index of the new surface.
    pub fn start_surface(&mut self, semantic: Option<SemanticType>) -> usize {
        let idx = self.surfaces.len();
        self.surfaces.push(SurfaceInProgress::default());
        self.surfaces[idx].semantic = semantic;
        self.current_surface = Some(idx);
        idx
    }

    /// Sets the outer ring for the current surface.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No surface is currently being built (`NoCurrentElement`)
    /// - The ring is invalid (`InvalidRing`)
    /// - An outer ring is already set (`InvalidGeometry`)
    pub fn set_surface_outer_ring(&mut self, vertices: &[usize]) -> Result<()> {
        let surface_idx = self
            .current_surface
            .ok_or_else(|| Error::NoCurrentElement {
                element_type: "surface".to_string(),
            })?;
        let ring_idx = self.add_ring(vertices)?;
        self.surfaces[surface_idx].outer_ring = Some(ring_idx);
        Ok(())
    }

    /// Adds an inner ring (hole) to the current surface.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No surface is currently being built (`NoCurrentElement`)
    /// - The current surface has no outer ring (`MissingOuterElement`)
    /// - The ring is invalid (`InvalidRing`)
    /// - The ring is not contained within the outer ring (`InvalidGeometry`)
    pub fn add_surface_inner_ring(&mut self, vertices: &[usize]) -> Result<()> {
        let surface_idx = self
            .current_surface
            .ok_or_else(|| Error::NoCurrentElement {
                element_type: "surface".to_string(),
            })?;

        if self.surfaces[surface_idx].outer_ring.is_none() {
            return Err(Error::MissingOuterElement {
                context: "Cannot add inner ring before outer ring is set".to_string(),
            });
        }

        let ring_idx = self.add_ring(vertices)?;
        self.surfaces[surface_idx].inner_rings.push(ring_idx);
        Ok(())
    }

    /// Starts a new shell.
    ///
    /// Returns the index of the new shell.
    pub fn start_shell(&mut self) -> usize {
        let idx = self.shells.len();
        self.shells.push(ShellInProgress::default());
        self.current_shell = Some(idx);
        idx
    }

    /// Adds an outer surface to the current shell.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No shell is currently being built (`NoCurrentElement`)
    /// - The surface index is invalid (`InvalidReference`)
    /// - The surface is already part of another shell (`InvalidGeometry`)
    pub fn add_shell_outer_surface(&mut self, surface_idx: usize) -> Result<()> {
        let shell_idx = self.current_shell.ok_or_else(|| Error::NoCurrentElement {
            element_type: "shell".to_string(),
        })?;

        if surface_idx >= self.surfaces.len() {
            return Err(Error::InvalidReference {
                element_type: "surface".to_string(),
                index: surface_idx,
                max_index: self.surfaces.len().saturating_sub(1),
            });
        }

        self.shells[shell_idx].outer_surfaces.push(surface_idx);
        Ok(())
    }

    /// Adds an inner surface to the current shell.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No shell is currently being built (`NoCurrentElement`)
    /// - The surface index is invalid (`InvalidReference`)
    /// - The shell has no outer surfaces (`MissingOuterElement`)
    /// - The surface is already part of another shell (`InvalidGeometry`)
    pub fn add_shell_inner_surface(&mut self, surface_idx: usize) -> Result<()> {
        let shell_idx = self.current_shell.ok_or_else(|| Error::NoCurrentElement {
            element_type: "shell".to_string(),
        })?;

        if surface_idx >= self.surfaces.len() {
            return Err(Error::InvalidReference {
                element_type: "surface".to_string(),
                index: surface_idx,
                max_index: self.surfaces.len().saturating_sub(1),
            });
        }

        if self.shells[shell_idx].outer_surfaces.is_empty() {
            return Err(Error::MissingOuterElement {
                context: "Cannot add inner surface before outer surfaces".to_string(),
            });
        }

        self.shells[shell_idx].inner_surfaces.push(surface_idx);
        Ok(())
    }

    /// Starts a new solid.
    ///
    /// Returns the index of the new solid.
    pub fn start_solid(&mut self) -> usize {
        let idx = self.solids.len();
        self.solids.push(SolidInProgress::default());
        self.current_solid = Some(idx);
        idx
    }

    /// Sets the outer shell for the current solid.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No solid is currently being built (`NoCurrentElement`)
    /// - The shell index is invalid (`InvalidReference`)
    /// - An outer shell is already set (`InvalidGeometry`)
    /// - The shell is already part of another solid (`InvalidGeometry`)
    pub fn set_solid_outer_shell(&mut self, shell_idx: usize) -> Result<()> {
        let solid_idx = self.current_solid.ok_or_else(|| Error::NoCurrentElement {
            element_type: "solid".to_string(),
        })?;

        if shell_idx >= self.shells.len() {
            return Err(Error::InvalidReference {
                element_type: "shell".to_string(),
                index: shell_idx,
                max_index: self.shells.len().saturating_sub(1),
            });
        }

        self.solids[solid_idx].outer_shell = Some(shell_idx);
        Ok(())
    }

    /// Adds an inner shell to the current solid.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - No solid is currently being built (`NoCurrentElement`)
    /// - The shell index is invalid (`InvalidReference`)
    /// - The solid has no outer shell (`MissingOuterElement`)
    /// - The shell is already part of another solid (`InvalidGeometry`)
    pub fn add_solid_inner_shell(&mut self, shell_idx: usize) -> Result<()> {
        let solid_idx = self.current_solid.ok_or_else(|| Error::NoCurrentElement {
            element_type: "solid".to_string(),
        })?;

        if shell_idx >= self.shells.len() {
            return Err(Error::InvalidReference {
                element_type: "shell".to_string(),
                index: shell_idx,
                max_index: self.shells.len().saturating_sub(1),
            });
        }

        if self.solids[solid_idx].outer_shell.is_none() {
            return Err(Error::MissingOuterElement {
                context: "Cannot add inner shell before outer shell is set".to_string(),
            });
        }

        self.solids[solid_idx].inner_shells.push(shell_idx);
        Ok(())
    }

    /// Builds the geometry and adds it to the model.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Any surface is missing an outer ring (`IncompleteGeometry`)
    /// - Any shell has no outer surfaces (`IncompleteGeometry`)
    /// - Any solid is missing an outer shell (`IncompleteGeometry`)
    /// - The geometry type doesn't match the structure (`InvalidGeometryType`)
    /// - Any vertex indices are invalid (`InvalidReference`)
    /// - Any referenced elements are invalid (`InvalidReference`)
    /// - The resulting geometry would be invalid (`InvalidGeometry`)
    pub fn build(self) -> Result<()> {
        // Validate structure before building
        self.validate_structure()?;

        // Add all vertices to the model and get their indices
        let vertex_indices: Vec<VertexIndex<T>> = self
            .vertices
            .into_iter()
            .map(|v| self.model.add_vertex(v))
            .collect::<Result<_>>()?;

        // Create boundary structure
        let mut boundary = Boundary::new();

        // Add vertices
        boundary.vertices = VertexIndices::from_iter(vertex_indices);

        // Create semantic mappings - only for surfaces
        let mut semantic_map = SemanticMaterialMap::default();
        let surface_semantic_indices = self
            .surfaces
            .iter()
            .map(|surface| {
                surface.semantic.as_ref().map(|sem_type| {
                    let semantic = Semantic::new(sem_type.clone());
                    let id = self.model.add_semantic(semantic);
                    VertexIndex::new(T::try_from(id.index() as usize).unwrap())
                })
            })
            .collect::<Vec<_>>(); // Explicitly collect to Vec first
        semantic_map.surfaces = OptionalVertexIndices::from_iter(surface_semantic_indices);

        // Process rings (both outer and inner)
        let mut ring_indices = Vec::new();
        for ring in self.rings {
            // Add indices for this ring's vertices
            for &vertex_idx in &ring {
                ring_indices.push(VertexIndex::new(T::try_from(vertex_idx).unwrap()));
            }
        }
        boundary.rings = VertexIndices::from_iter(ring_indices);

        // Process surfaces with their rings
        let mut surface_start_indices = Vec::new();
        let mut current_ring_idx = T::zero();

        for surface in &self.surfaces {
            // Add index to outer ring
            if let Some(outer_ring) = surface.outer_ring {
                surface_start_indices.push(VertexIndex::new(current_ring_idx));
                current_ring_idx = current_ring_idx.checked_add(&T::one()).unwrap();

                // Add indices to inner rings if any
                for _ in &surface.inner_rings {
                    surface_start_indices.push(VertexIndex::new(current_ring_idx));
                    current_ring_idx = current_ring_idx.checked_add(&T::one()).unwrap();
                }
            }
        }
        boundary.surfaces = VertexIndices::from_iter(surface_start_indices);

        // Process shells with their surfaces
        let mut shell_start_indices = Vec::new();
        let mut current_surface_idx = T::zero();

        for shell in &self.shells {
            // Add indices to outer surfaces
            shell_start_indices.push(VertexIndex::new(current_surface_idx));
            current_surface_idx = current_surface_idx
                .checked_add(&T::try_from(shell.outer_surfaces.len()).unwrap())
                .unwrap();

            // Add indices to inner surfaces
            if !shell.inner_surfaces.is_empty() {
                current_surface_idx = current_surface_idx
                    .checked_add(&T::try_from(shell.inner_surfaces.len()).unwrap())
                    .unwrap();
            }
        }
        if !shell_start_indices.is_empty() {
            boundary.shells = VertexIndices::from_iter(shell_start_indices);
        }

        // Process solids with their shells
        let mut solid_start_indices = Vec::new();
        let mut current_shell_idx = T::zero();

        for solid in &self.solids {
            // Add index to outer shell
            if let Some(_) = solid.outer_shell {
                solid_start_indices.push(VertexIndex::new(current_shell_idx));
                current_shell_idx = current_shell_idx.checked_add(&T::one()).unwrap();

                // Add indices to inner shells if any
                current_shell_idx = current_shell_idx
                    .checked_add(&T::try_from(solid.inner_shells.len()).unwrap())
                    .unwrap();
            }
        }
        if !solid_start_indices.is_empty() {
            boundary.solids = VertexIndices::from_iter(solid_start_indices);
        }

        // Create the geometry
        let geometry = Geometry {
            type_geometry: self.type_geometry,
            lod: self.lod,
            boundaries: Some(boundary),
            semantics: Some(semantic_map),
            template_boundaries: None,
            template_transformation_matrix: None,
        };

        self.model.add_geometry(geometry);
        Ok(())
    }

    fn validate_structure(&self) -> Result<()> {
        // Verify surfaces
        for (i, surface) in self.surfaces.iter().enumerate() {
            if surface.outer_ring.is_none() {
                return Err(Error::IncompleteGeometry(format!(
                    "Surface {} missing outer ring",
                    i
                )));
            }
        }

        // Verify shells
        for (i, shell) in self.shells.iter().enumerate() {
            if shell.outer_surfaces.is_empty() {
                return Err(Error::IncompleteGeometry(format!(
                    "Shell {} has no outer surfaces",
                    i
                )));
            }
        }

        // Verify solids
        for (i, solid) in self.solids.iter().enumerate() {
            if solid.outer_shell.is_none() {
                return Err(Error::IncompleteGeometry(format!(
                    "Solid {} missing outer shell",
                    i
                )));
            }
        }

        // Verify geometry type matches structure
        match self.type_geometry {
            GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                if self.solids.is_empty() {
                    return Err(Error::InvalidGeometryType {
                        expected: "solid geometry".to_string(),
                        found: "empty geometry".to_string(),
                    });
                }
            }
            GeometryType::Solid => {
                if self.solids.len() != 1 {
                    return Err(Error::InvalidGeometryType {
                        expected: "single solid".to_string(),
                        found: format!("{} solids", self.solids.len()),
                    });
                }
            }
            GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                if self.surfaces.is_empty() {
                    return Err(Error::InvalidGeometryType {
                        expected: "surface geometry".to_string(),
                        found: "empty geometry".to_string(),
                    });
                }
                if !self.shells.is_empty() || !self.solids.is_empty() {
                    return Err(Error::InvalidGeometryType {
                        expected: "surface geometry".to_string(),
                        found: "geometry with shells or solids".to_string(),
                    });
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CityModel;

    #[test]
    fn test_build_complex_multisolid() -> Result<()> {
        let mut model = CityModel::<u32>::new();
        let mut builder =
            GeometryBuilder::new(&mut model, GeometryType::MultiSolid).with_lod(LoD::LoD2);

        // Create vertices for a cube with an inner cube
        // Outer cube vertices (0-7)
        let v0 = builder.add_vertex(0.0, 0.0, 0.0);
        let v1 = builder.add_vertex(2.0, 0.0, 0.0);
        let v2 = builder.add_vertex(2.0, 2.0, 0.0);
        let v3 = builder.add_vertex(0.0, 2.0, 0.0);
        let v4 = builder.add_vertex(0.0, 0.0, 2.0);
        let v5 = builder.add_vertex(2.0, 0.0, 2.0);
        let v6 = builder.add_vertex(2.0, 2.0, 2.0);
        let v7 = builder.add_vertex(0.0, 2.0, 2.0);

        // Inner cube vertices (8-15) - smaller cube inside
        let v8 = builder.add_vertex(0.5, 0.5, 0.5);
        let v9 = builder.add_vertex(1.5, 0.5, 0.5);
        let v10 = builder.add_vertex(1.5, 1.5, 0.5);
        let v11 = builder.add_vertex(0.5, 1.5, 0.5);
        let v12 = builder.add_vertex(0.5, 0.5, 1.5);
        let v13 = builder.add_vertex(1.5, 0.5, 1.5);
        let v14 = builder.add_vertex(1.5, 1.5, 1.5);
        let v15 = builder.add_vertex(0.5, 1.5, 1.5);

        // Window vertices for inner rings in front and right faces
        let w0 = builder.add_vertex(0.7, 0.0, 0.7);
        let w1 = builder.add_vertex(1.3, 0.0, 0.7);
        let w2 = builder.add_vertex(1.3, 0.0, 1.3);
        let w3 = builder.add_vertex(0.7, 0.0, 1.3);

        let w4 = builder.add_vertex(2.0, 0.7, 0.7);
        let w5 = builder.add_vertex(2.0, 1.3, 0.7);
        let w6 = builder.add_vertex(2.0, 1.3, 1.3);
        let w7 = builder.add_vertex(2.0, 0.7, 1.3);

        // Start building the outer shell
        let _solid_idx = builder.start_solid();
        let outer_shell_idx = builder.start_shell();

        // Bottom face (GroundSurface)
        let bottom_idx = builder.start_surface(Some(SemanticType::GroundSurface));
        builder.set_surface_outer_ring(&[v0, v3, v2, v1])?;
        builder.add_shell_outer_surface(bottom_idx)?;

        // Top face (RoofSurface)
        let top_idx = builder.start_surface(Some(SemanticType::RoofSurface));
        builder.set_surface_outer_ring(&[v4, v5, v6, v7])?;
        builder.add_shell_outer_surface(top_idx)?;

        // Front face with window (WallSurface)
        let front_idx = builder.start_surface(Some(SemanticType::WallSurface));
        builder.set_surface_outer_ring(&[v0, v1, v5, v4])?;
        builder.add_surface_inner_ring(&[w0, w1, w2, w3])?; // Window hole
        builder.add_shell_outer_surface(front_idx)?;

        // Right face with window (WallSurface)
        let right_idx = builder.start_surface(Some(SemanticType::WallSurface));
        builder.set_surface_outer_ring(&[v1, v2, v6, v5])?;
        builder.add_surface_inner_ring(&[w4, w5, w6, w7])?; // Window hole
        builder.add_shell_outer_surface(right_idx)?;

        // Back face (WallSurface)
        let back_idx = builder.start_surface(Some(SemanticType::WallSurface));
        builder.set_surface_outer_ring(&[v2, v3, v7, v6])?;
        builder.add_shell_outer_surface(back_idx)?;

        // Left face (no semantics)
        let left_idx = builder.start_surface(None);
        builder.set_surface_outer_ring(&[v3, v0, v4, v7])?;
        builder.add_shell_outer_surface(left_idx)?;

        // Set the outer shell
        builder.set_solid_outer_shell(outer_shell_idx)?;

        // Start building the inner shell (void)
        let inner_shell_idx = builder.start_shell();

        // Create the faces of the inner cube (all surfaces of inner shell)
        // Bottom
        let inner_bottom_idx = builder.start_surface(Some(SemanticType::InteriorWallSurface));
        builder.set_surface_outer_ring(&[v8, v11, v10, v9])?;
        builder.add_shell_outer_surface(inner_bottom_idx)?;

        // Top
        let inner_top_idx = builder.start_surface(Some(SemanticType::InteriorWallSurface));
        builder.set_surface_outer_ring(&[v12, v13, v14, v15])?;
        builder.add_shell_outer_surface(inner_top_idx)?;

        // Four sides
        let inner_sides = [
            &[v8, v9, v13, v12],   // Front
            &[v9, v10, v14, v13],  // Right
            &[v10, v11, v15, v14], // Back
            &[v11, v8, v12, v15],  // Left
        ];

        for vertices in inner_sides {
            let inner_side_idx = builder.start_surface(Some(SemanticType::InteriorWallSurface));
            builder.set_surface_outer_ring(vertices)?;
            builder.add_shell_outer_surface(inner_side_idx)?;
        }

        // Add the inner shell to the solid
        builder.add_solid_inner_shell(inner_shell_idx)?;

        // Build the geometry
        builder.build()?;

        // Verify the result
        assert_eq!(model.geometry_count(), 1);
        assert_eq!(model.vertex_count(), 24); // 8 outer + 8 inner + 8 window vertices
        assert_eq!(model.semantic_count(), 10); // 5 outer surfaces + 6 inner surfaces - 1 without semantics

        if let Some(geometry) = model.geometries.first() {
            assert_eq!(geometry.type_geometry, GeometryType::MultiSolid);
            assert_eq!(geometry.lod, Some(LoD::LoD2));

            if let Some(boundary) = &geometry.boundaries {
                // Verify boundary structure
                assert_eq!(boundary.vertices.len(), 24u32);
                // Add more specific boundary checks...
            } else {
                panic!("Expected boundary");
            }

            if let Some(_semantics) = &geometry.semantics {
                // Verify semantic mappings
                // Add specific semantic checks...
            } else {
                panic!("Expected semantics");
            }
        } else {
            panic!("Expected geometry");
        }

        Ok(())
    }
}
