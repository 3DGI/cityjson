//! # Geometry
//!
//! Represents a [Geometry object](https://www.cityjson.org/specs/1.1.3/#geometry-objects).
use crate::cityjson::geometry::boundary::Boundary;
use crate::cityjson::geometry::semantic::SemanticType;
use crate::cityjson::geometry::{GeometryTrait, GeometryType, LoD};
use crate::cityjson::index::VertexRef;
use crate::errors::Result;
use crate::resources::mapping::{MaterialMap, SemanticMap, TextureMap};
use crate::resources::pool::{ResourcePool, ResourceRef};
use crate::resources::storage::StringStorage;

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct Geometry<VR: VertexRef, RR: ResourceRef> {
    type_geometry: GeometryType,
    lod: Option<LoD>,
    boundaries: Option<Boundary<VR>>,
    semantics: Option<SemanticMap<VR, RR>>,
    material: Option<MaterialMap<VR, RR>>,
    texture: Option<TextureMap<VR, RR>>,
    template_boundaries: Option<usize>,
    template_transformation_matrix: Option<[f64; 16]>,
}

impl<VR, RR, SS> GeometryTrait<VR, RR, SS> for Geometry<VR, RR>
where
    VR: VertexRef,
    RR: ResourceRef,
    SS: StringStorage,
{
    fn new(
        type_geometry: GeometryType,
        lod: Option<LoD>,
        boundaries: Option<Boundary<VR>>,
        semantics: Option<SemanticMap<VR, RR>>,
        material: Option<MaterialMap<VR, RR>>,
        texture: Option<TextureMap<VR, RR>>,
        template_boundaries: Option<usize>,
        template_transformation_matrix: Option<[f64; 16]>,
    ) -> Self {
        Self {
            type_geometry,
            lod,
            boundaries,
            semantics,
            material,
            texture,
            template_boundaries,
            template_transformation_matrix,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cityjson::attributes::AttributeValue;
    use crate::cityjson::boundary::nested::BoundaryNestedMultiOrCompositeSolid32;
    use crate::cityjson::geometry::GeometryBuilder;
    use crate::cityjson::semantic::Semantic;
    use crate::cityjson::storage::OwnedStringStorage;
    use crate::resources::pool::ResourceId32;
    use crate::v1_1::citymodel::CityModel;

    #[test]
    fn test_multipoint_with_semantics() -> Result<()> {
        // Create a city model
        let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new();
        let mut builder =
            GeometryBuilder::new(&mut model, GeometryType::MultiPoint).with_lod(LoD::LoD2);

        // Create vertices with semantics
        // First point - TransportationMarking
        let v0 = builder.add_point_with_semantic(
            0.0,
            0.0,
            0.0,
            Some(Semantic::new(SemanticType::TransportationMarking)),
        );

        // Second point - no semantic
        let _v1 = builder.add_vertex(1.0, 0.0, 0.0);

        // Third point - TransportationHole with diameter attribute
        let mut hole_semantic = Semantic::new(SemanticType::TransportationHole);
        let mut attrs = hole_semantic.attributes_mut();
        attrs.insert("diameter".to_string(), AttributeValue::Float(1.5));
        let v2 = builder.add_point_with_semantic(2.0, 0.0, 0.0, Some(hole_semantic));

        // Fourth point - no semantic
        let _v3 = builder.add_vertex(3.0, 0.0, 0.0);

        // Fifth point - TransportationMarking
        let v4 = builder.add_point_with_semantic(
            4.0,
            0.0,
            0.0,
            Some(Semantic::new(SemanticType::TransportationMarking)),
        );

        // Build the geometry
        builder.build()?;

        // Get the built geometry and test filtering
        let geometry = &model.geometries[0];

        Ok(())
    }

    #[test]
    fn test_build_complex_multisolid() -> Result<()> {
        let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new();
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

        // Front face with a window (WallSurface)
        let front_idx = builder.start_surface(Some(SemanticType::WallSurface));
        builder.set_surface_outer_ring(&[v0, v1, v5, v4])?;
        builder.add_surface_inner_ring(&[w0, w1, w2, w3])?; // Window hole
        builder.add_shell_outer_surface(front_idx)?;

        // Right face with a window (WallSurface)
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

        // Add vertices for a tetrahedron (second solid)
        let t0 = builder.add_vertex(4.0, 0.0, 0.0); // base point 1
        let t1 = builder.add_vertex(5.0, 0.0, 0.0); // base point 2
        let t2 = builder.add_vertex(4.5, 1.0, 0.0); // base point 3
        let t3 = builder.add_vertex(4.5, 0.5, 1.0); // apex

        // Start building the second solid (tetrahedron)
        let _second_solid_idx = builder.start_solid();
        let tetra_shell_idx = builder.start_shell();

        // Create the four triangular faces of the tetrahedron
        // Base face
        let tetra_base_idx = builder.start_surface(Some(SemanticType::GroundSurface));
        builder.set_surface_outer_ring(&[t0, t1, t2])?;
        builder.add_shell_outer_surface(tetra_base_idx)?;

        // Three side faces
        let side_vertices = [
            &[t0, t2, t3], // First side
            &[t1, t3, t2], // Second side
            &[t0, t3, t1], // Third side
        ];

        for vertices in side_vertices {
            let side_idx = builder.start_surface(Some(SemanticType::WallSurface));
            builder.set_surface_outer_ring(vertices)?;
            builder.add_shell_outer_surface(side_idx)?;
        }

        // Set the outer shell for the tetrahedron
        builder.set_solid_outer_shell(tetra_shell_idx)?;

        // Build the geometry
        builder.build()?;

        // Get the boundary from the built geometry
        let geometry = model.geometries.first().unwrap();
        let boundary = geometry.boundaries.as_ref().unwrap();

        // Convert to nested representation and compare
        let nested = boundary.to_nested_multi_or_composite_solid()?;

        // Expected nested structure:
        let expected: BoundaryNestedMultiOrCompositeSolid32 = vec![
            // First solid (outer cube with windows and inner cube)
            vec![
                // Outer shell
                vec![
                    // Bottom face (ground)
                    vec![vec![0, 3, 2, 1]],
                    // Top face (roof)
                    vec![vec![4, 5, 6, 7]],
                    // Front face with a window
                    vec![
                        vec![0, 1, 5, 4],     // Outer ring
                        vec![16, 17, 18, 19], // Window hole
                    ],
                    // Right face with a window
                    vec![
                        vec![1, 2, 6, 5],     // Outer ring
                        vec![20, 21, 22, 23], // Window hole
                    ],
                    // Back face
                    vec![vec![2, 3, 7, 6]],
                    // Left face
                    vec![vec![3, 0, 4, 7]],
                ],
                // Inner shell (void cube)
                vec![
                    // Bottom face
                    vec![vec![8, 11, 10, 9]],
                    // Top face
                    vec![vec![12, 13, 14, 15]],
                    // Front face
                    vec![vec![8, 9, 13, 12]],
                    // Right face
                    vec![vec![9, 10, 14, 13]],
                    // Back face
                    vec![vec![10, 11, 15, 14]],
                    // Left face
                    vec![vec![11, 8, 12, 15]],
                ],
            ],
            // Second solid (tetrahedron)
            vec![
                // Outer shell
                vec![
                    // Base face
                    vec![vec![24, 25, 26]],
                    // Three side faces
                    vec![vec![24, 26, 27]],
                    vec![vec![25, 27, 26]],
                    vec![vec![24, 27, 25]],
                ],
            ],
        ];

        assert_eq!(nested, expected);
        Ok(())
    }
}
