use cityjson::prelude::*;
use cityjson::v1_1::*;

/// Integration test for building a MultiPoint geometry and extracting its boundary
#[test]
fn test_multipoint_geometry_boundary() {
    // Create a new city model
    let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new();

    // Create a GeometryBuilder for a MultiPoint geometry
    let mut builder = GeometryBuilder::new(&mut model, GeometryType::MultiPoint);
    builder = builder.with_lod(LoD::LoD1);

    // Add points
    let _p1 = builder.add_vertex(0.0, 0.0, 0.0);
    let _p2 = builder.add_vertex(1.0, 0.0, 0.0);
    let _p3 = builder.add_vertex(1.0, 1.0, 0.0);
    let _p4 = builder.add_vertex(0.0, 1.0, 0.0);

    // Build the geometry
    builder.build().unwrap();

    // Verify the model has one geometry
    assert_eq!(model.geometries().len(), 1);

    // Get the geometry and verify its type
    let (_resource_id, geometry) = model.geometries().first().unwrap();
    assert_eq!(geometry.type_geometry(), &GeometryType::MultiPoint);

    // Get the boundary
    let boundary = geometry.boundaries().unwrap();

    // Verify boundary type
    assert_eq!(boundary.check_type(), BoundaryType::MultiPoint);

    // Convert to nested representation and verify content
    let nested = boundary.to_nested_multi_point().unwrap();
    assert_eq!(nested.len(), 4); // Four points were added

    // Verify the boundary is consistent
    assert!(boundary.is_consistent());
}

/// Integration test for building a MultiLineString geometry and extracting its boundary
#[test]
fn test_multilinestring_geometry_boundary() {
    // Create a new city model
    let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new();

    // Create a GeometryBuilder for a MultiLineString geometry
    let mut builder = GeometryBuilder::new(&mut model, GeometryType::MultiLineString);
    builder = builder.with_lod(LoD::LoD1);

    // Add vertices
    let p1 = builder.add_vertex(0.0, 0.0, 0.0);
    let p2 = builder.add_vertex(1.0, 0.0, 0.0);
    let p3 = builder.add_vertex(1.0, 1.0, 0.0);
    let p4 = builder.add_vertex(0.0, 1.0, 0.0);

    // Add linestrings (rings)
    builder.add_ring(&[p1, p2, p3]).unwrap();
    builder.add_ring(&[p3, p4, p1]).unwrap();

    // Build the geometry
    builder.build().unwrap();

    // Verify the model has one geometry
    assert_eq!(model.geometries().len(), 1);

    // Get the geometry and verify its type
    let (_resource_id, geometry) = model.geometries().first().unwrap();
    assert_eq!(geometry.type_geometry(), &GeometryType::MultiLineString);

    // Get the boundary
    let boundary = geometry.boundaries().unwrap();

    // Verify boundary type
    assert_eq!(boundary.check_type(), BoundaryType::MultiLineString);

    // Convert to nested representation and verify content
    let nested = boundary.to_nested_multi_linestring().unwrap();
    assert_eq!(nested.len(), 2); // Two linestrings were added
    assert_eq!(nested[0].len(), 3); // First linestring has 3 vertices
    assert_eq!(nested[1].len(), 3); // Second linestring has 3 vertices

    // Verify the boundary is consistent
    assert!(boundary.is_consistent());
}

/// Integration test for building a MultiSurface geometry with semantics and extracting its boundary
#[test]
fn test_multisurface_geometry_with_semantics() {
    // Create a new city model
    let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new();

    // Create a GeometryBuilder for a MultiSurface geometry
    let mut builder = GeometryBuilder::new(&mut model, GeometryType::MultiSurface);
    builder = builder.with_lod(LoD::LoD2);

    // Add vertices for a square
    let p1 = builder.add_vertex(0.0, 0.0, 0.0);
    let p2 = builder.add_vertex(1.0, 0.0, 0.0);
    let p3 = builder.add_vertex(1.0, 1.0, 0.0);
    let p4 = builder.add_vertex(0.0, 1.0, 0.0);

    // Start the first surface (wall)
    let s1 = builder.start_surface(Some(SemanticType::WallSurface));

    // Set the outer ring for the surface
    builder
        .set_surface_outer_ring(&[p1, p2, p3, p4, p1])
        .unwrap();

    // Add semantic information
    let wall_semantic = Semantic::new(SemanticType::WallSurface);
    builder.set_surface_semantic(wall_semantic).unwrap();

    // Start the second surface (roof)
    let s2 = builder.start_surface(Some(SemanticType::RoofSurface));

    // Set the outer ring for the surface
    builder
        .set_surface_outer_ring(&[p1, p4, p3, p2, p1])
        .unwrap();

    // Add semantic information
    let roof_semantic = Semantic::new(SemanticType::RoofSurface);
    builder.set_surface_semantic(roof_semantic).unwrap();

    // Build the geometry
    builder.build().unwrap();

    // Verify the model has one geometry
    assert_eq!(model.geometries().len(), 1);

    // Get the geometry and verify its type
    let (_resource_id, geometry) = model.geometries().first().unwrap();
    assert_eq!(geometry.type_geometry(), &GeometryType::MultiSurface);

    // Get the boundary
    let boundary = geometry.boundaries().unwrap();

    // Verify boundary type
    assert_eq!(boundary.check_type(), BoundaryType::MultiOrCompositeSurface);

    // Convert to nested representation and verify content
    let nested = boundary.to_nested_multi_or_composite_surface().unwrap();
    assert_eq!(nested.len(), 2); // Two surfaces were added

    // Get semantics
    let semantics = geometry.semantics().unwrap();
    assert!(!semantics.surfaces().is_empty());

    // Verify the boundary is consistent
    assert!(boundary.is_consistent());
}

/// Integration test for building a Solid geometry and extracting its boundary
#[test]
fn test_solid_geometry_boundary() {
    // Create a new city model
    let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new();

    // Create a GeometryBuilder for a Solid geometry
    let mut builder = GeometryBuilder::new(&mut model, GeometryType::Solid);
    builder = builder.with_lod(LoD::LoD2);

    // Add vertices for a cube
    let p1 = builder.add_vertex(0.0, 0.0, 0.0); // Bottom face
    let p2 = builder.add_vertex(1.0, 0.0, 0.0);
    let p3 = builder.add_vertex(1.0, 1.0, 0.0);
    let p4 = builder.add_vertex(0.0, 1.0, 0.0);
    let p5 = builder.add_vertex(0.0, 0.0, 1.0); // Top face
    let p6 = builder.add_vertex(1.0, 0.0, 1.0);
    let p7 = builder.add_vertex(1.0, 1.0, 1.0);
    let p8 = builder.add_vertex(0.0, 1.0, 1.0);

    // Start a shell
    let _shell_idx = builder.start_shell();

    // Add surfaces to the shell (6 faces of the cube)

    // Bottom face
    let s1 = builder.start_surface(Some(SemanticType::GroundSurface));
    builder
        .set_surface_outer_ring(&[p1, p4, p3, p2, p1])
        .unwrap();
    builder.add_shell_outer_surface(s1).unwrap();

    // Top face
    let s2 = builder.start_surface(Some(SemanticType::RoofSurface));
    builder
        .set_surface_outer_ring(&[p5, p6, p7, p8, p5])
        .unwrap();
    builder.add_shell_outer_surface(s2).unwrap();

    // Side faces
    let s3 = builder.start_surface(Some(SemanticType::WallSurface));
    builder
        .set_surface_outer_ring(&[p1, p2, p6, p5, p1])
        .unwrap();
    builder.add_shell_outer_surface(s3).unwrap();

    let s4 = builder.start_surface(Some(SemanticType::WallSurface));
    builder
        .set_surface_outer_ring(&[p2, p3, p7, p6, p2])
        .unwrap();
    builder.add_shell_outer_surface(s4).unwrap();

    let s5 = builder.start_surface(Some(SemanticType::WallSurface));
    builder
        .set_surface_outer_ring(&[p3, p4, p8, p7, p3])
        .unwrap();
    builder.add_shell_outer_surface(s5).unwrap();

    let s6 = builder.start_surface(Some(SemanticType::WallSurface));
    builder
        .set_surface_outer_ring(&[p4, p1, p5, p8, p4])
        .unwrap();
    builder.add_shell_outer_surface(s6).unwrap();

    // Build the geometry
    builder.build().unwrap();

    // Verify the model has one geometry
    assert_eq!(model.geometries().len(), 1);

    // Get the geometry and verify its type
    let (_resource_id, geometry) = model.geometries().first().unwrap();
    assert_eq!(geometry.type_geometry(), &GeometryType::Solid);

    // Get the boundary
    let boundary = geometry.boundaries().unwrap();

    // Verify boundary type
    assert_eq!(boundary.check_type(), BoundaryType::Solid);

    // Convert to nested representation and verify content
    let nested = boundary.to_nested_solid().unwrap();
    assert_eq!(nested.len(), 1); // One solid with...
    assert_eq!(nested[0].len(), 6); // 6 surfaces

    // Verify the boundary is consistent
    assert!(boundary.is_consistent());
}

/// Integration test for building a MultiSolid geometry and extracting its boundary
#[test]
fn test_multisolid_geometry_boundary() {
    // Create a new city model
    let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new();

    // Create a GeometryBuilder for a MultiSolid geometry
    let mut builder = GeometryBuilder::new(&mut model, GeometryType::MultiSolid);
    builder = builder.with_lod(LoD::LoD2);

    // Create two simple cubes

    // === First cube ===
    // Add vertices for the first cube
    let p1 = builder.add_vertex(0.0, 0.0, 0.0);
    let p2 = builder.add_vertex(1.0, 0.0, 0.0);
    let p3 = builder.add_vertex(1.0, 1.0, 0.0);
    let p4 = builder.add_vertex(0.0, 1.0, 0.0);
    let p5 = builder.add_vertex(0.0, 0.0, 1.0);
    let p6 = builder.add_vertex(1.0, 0.0, 1.0);
    let p7 = builder.add_vertex(1.0, 1.0, 1.0);
    let p8 = builder.add_vertex(0.0, 1.0, 1.0);

    // Start a shell
    let shell1_idx = builder.start_shell();

    // Add surfaces to the shell (just using 2 faces for simplicity)
    let s1 = builder.start_surface(None);
    builder
        .set_surface_outer_ring(&[p1, p4, p3, p2, p1])
        .unwrap();
    builder.add_shell_outer_surface(s1).unwrap();

    let s2 = builder.start_surface(None);
    builder
        .set_surface_outer_ring(&[p5, p6, p7, p8, p5])
        .unwrap();
    builder.add_shell_outer_surface(s2).unwrap();

    // Start a solid
    let solid1_idx = builder.start_solid();

    // Set the outer shell of the solid
    builder.set_solid_outer_shell(shell1_idx).unwrap();

    // === Second cube ===
    // Add vertices for the second cube (shifted by 2.0 in x direction)
    let p9 = builder.add_vertex(2.0, 0.0, 0.0);
    let p10 = builder.add_vertex(3.0, 0.0, 0.0);
    let p11 = builder.add_vertex(3.0, 1.0, 0.0);
    let p12 = builder.add_vertex(2.0, 1.0, 0.0);
    let p13 = builder.add_vertex(2.0, 0.0, 1.0);
    let p14 = builder.add_vertex(3.0, 0.0, 1.0);
    let p15 = builder.add_vertex(3.0, 1.0, 1.0);
    let p16 = builder.add_vertex(2.0, 1.0, 1.0);

    // Start a shell
    let shell2_idx = builder.start_shell();

    // Add surfaces to the shell (just using 2 faces for simplicity)
    let s3 = builder.start_surface(None);
    builder
        .set_surface_outer_ring(&[p9, p12, p11, p10, p9])
        .unwrap();
    builder.add_shell_outer_surface(s3).unwrap();

    let s4 = builder.start_surface(None);
    builder
        .set_surface_outer_ring(&[p13, p14, p15, p16, p13])
        .unwrap();
    builder.add_shell_outer_surface(s4).unwrap();

    // Start a solid
    let solid2_idx = builder.start_solid();

    // Set the outer shell of the solid
    builder.set_solid_outer_shell(shell2_idx).unwrap();

    // Build the geometry
    builder.build().unwrap();

    // Verify the model has one geometry
    assert_eq!(model.geometries().len(), 1);

    // Get the geometry and verify its type
    let (_resource_id, geometry) = model.geometries().first().unwrap();
    assert_eq!(geometry.type_geometry(), &GeometryType::MultiSolid);

    // Get the boundary
    let boundary = geometry.boundaries().unwrap();

    // Verify boundary type
    assert_eq!(boundary.check_type(), BoundaryType::MultiOrCompositeSolid);

    // Convert to nested representation and verify content
    let nested = boundary.to_nested_multi_or_composite_solid().unwrap();
    assert_eq!(nested.len(), 2); // Two solids
    assert_eq!(nested[0].len(), 1); // First solid has 1 shell
    assert_eq!(nested[1].len(), 1); // Second solid has 1 shell
    assert_eq!(nested[0][0].len(), 2); // First shell has 2 surfaces
    assert_eq!(nested[1][0].len(), 2); // Second shell has 2 surfaces

    // Verify the boundary is consistent
    assert!(boundary.is_consistent());
}

/// Integration test for a geometry with materials and verifying boundary interaction
#[test]
fn test_geometry_with_materials() {
    // Create a new city model
    let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new();

    // Create a GeometryBuilder for a MultiSurface geometry
    let mut builder = GeometryBuilder::new(&mut model, GeometryType::MultiSurface);
    builder = builder.with_lod(LoD::LoD2);

    // Add vertices for a square
    let p1 = builder.add_vertex(0.0, 0.0, 0.0);
    let p2 = builder.add_vertex(1.0, 0.0, 0.0);
    let p3 = builder.add_vertex(1.0, 1.0, 0.0);
    let p4 = builder.add_vertex(0.0, 1.0, 0.0);

    // Start the first surface
    let s1 = builder.start_surface(Some(SemanticType::WallSurface));

    // Set the outer ring for the surface
    builder
        .set_surface_outer_ring(&[p1, p2, p3, p4, p1])
        .unwrap();

    // Create a material and add it to the surface
    let mut material = Material::default();
    material.set_name("Red Wall".to_string());
    material.set_diffuse_color(Some([1.0, 0.0, 0.0]));
    builder.set_surface_material(material).unwrap();

    // Start the second surface
    let s2 = builder.start_surface(Some(SemanticType::RoofSurface));

    // Set the outer ring for the surface
    builder
        .set_surface_outer_ring(&[p1, p4, p3, p2, p1])
        .unwrap();

    // Create another material and add it to the surface
    let mut material2 = Material::default();
    material2.set_name("Blue Roof".to_string());
    material2.set_diffuse_color(Some([0.0, 0.0, 1.0]));
    builder.set_surface_material(material2).unwrap();

    // Build the geometry
    builder.build().unwrap();

    // Verify the model has one geometry
    assert_eq!(model.geometries().len(), 1);

    // Get the geometry and verify its type
    let (_resource_id, geometry) = model.geometries().first().unwrap();
    assert_eq!(geometry.type_geometry(), &GeometryType::MultiSurface);

    // Get the boundary
    let boundary = geometry.boundaries().unwrap();

    // Verify boundary type
    assert_eq!(boundary.check_type(), BoundaryType::MultiOrCompositeSurface);

    // Get materials
    let materials = geometry.materials().unwrap();

    // Verify we have materials for surfaces
    assert!(!materials.surfaces().is_empty());
    assert_eq!(materials.surfaces().len(), 2); // Two surfaces have materials

    // Verify the boundary is consistent
    assert!(boundary.is_consistent());
}

/// Integration test for converting between nested and flattened representations
/// in the context of a complete CityJSON workflow
#[test]
fn test_nested_and_flattened_conversion_workflow() {
    // 1. Start with a nested representation (as if coming from a JSON parser)
    let nested_surface: BoundaryNestedMultiOrCompositeSurface<u32> = vec![
        // First surface with one ring
        vec![vec![0, 1, 2, 0]],
        // Second surface with two rings (outer and inner)
        vec![vec![3, 4, 5, 3], vec![6, 7, 8, 6]],
    ];

    // 2. Convert to flattened representation
    let flattened: Boundary<u32> = nested_surface.clone().into();
    assert_eq!(
        flattened.check_type(),
        BoundaryType::MultiOrCompositeSurface
    );

    // 3. If we were to add this to a CityModel, we would use GeometryBuilder
    let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new();
    let mut builder = GeometryBuilder::new(&mut model, GeometryType::MultiSurface);

    // Add vertices
    let v0 = builder.add_vertex(0.0, 0.0, 0.0);
    let v1 = builder.add_vertex(1.0, 0.0, 0.0);
    let v2 = builder.add_vertex(0.0, 1.0, 0.0);
    let v3 = builder.add_vertex(2.0, 0.0, 0.0);
    let v4 = builder.add_vertex(3.0, 0.0, 0.0);
    let v5 = builder.add_vertex(2.0, 1.0, 0.0);
    let v6 = builder.add_vertex(2.5, 0.5, 0.0);
    let v7 = builder.add_vertex(2.7, 0.5, 0.0);
    let v8 = builder.add_vertex(2.5, 0.7, 0.0);

    // Add first surface
    let _s1 = builder.start_surface(None);
    builder.set_surface_outer_ring(&[v0, v1, v2, v0]).unwrap();

    // Add second surface with inner ring
    let _s2 = builder.start_surface(None);
    builder.set_surface_outer_ring(&[v3, v4, v5, v3]).unwrap();
    builder.add_surface_inner_ring(&[v6, v7, v8, v6]).unwrap();

    // Build the geometry
    builder.build().unwrap();

    // 4. Get the boundary from the built geometry
    let (_resource_id, geometry) = model.geometries().first().unwrap();
    let boundary = geometry.boundaries().unwrap();

    // 5. Convert back to nested for serialization
    let nested_again = boundary.to_nested_multi_or_composite_surface().unwrap();

    // 6. Verify that the structure matches our original expectation
    assert_eq!(nested_again.len(), 2); // Two surfaces
    assert_eq!(nested_again[0].len(), 1); // First surface has 1 ring
    assert_eq!(nested_again[1].len(), 2); // Second surface has 2 rings

    // 7. Verify consistency
    assert!(boundary.is_consistent());
}
