//! Test family 4: Boundary round-trip (nested ↔ flat).
//!
//! Verifies that:
//! - nested → flat → nested preserves exact topology
//! - flat → nested → flat preserves exact arrays
//!
//! The round-trip tests cover all canonical fixtures.

use super::fixtures::*;
use cityjson_types::v2_0::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn assert_boundary_consistent(boundary: &Boundary<u32>) {
    assert!(
        boundary.is_consistent(),
        "boundary offsets must be consistent after build"
    );
}

// ---------------------------------------------------------------------------
// P1 round-trip
// ---------------------------------------------------------------------------

#[test]
fn p1_boundary_consistent() {
    let P1Result { model, handle } = build_p1();
    let boundary = model.get_geometry(handle).unwrap().boundaries().unwrap();
    assert_boundary_consistent(boundary);
}

#[test]
fn p1_flat_to_nested_to_flat() {
    let P1Result { model, handle } = build_p1();
    let boundary = model.get_geometry(handle).unwrap().boundaries().unwrap();

    let nested = boundary.to_nested_multi_point().unwrap();
    let back: Boundary<u32> = nested.into();

    assert_boundary_consistent(&back);
    assert_eq!(back.vertices(), boundary.vertices(), "vertices unchanged");
}

// ---------------------------------------------------------------------------
// L1 round-trip
// ---------------------------------------------------------------------------

#[test]
fn l1_boundary_consistent() {
    let L1Result { model, handle } = build_l1();
    let boundary = model.get_geometry(handle).unwrap().boundaries().unwrap();
    assert_boundary_consistent(boundary);
}

#[test]
fn l1_flat_to_nested_to_flat() {
    let L1Result { model, handle } = build_l1();
    let boundary = model.get_geometry(handle).unwrap().boundaries().unwrap();

    let nested: BoundaryNestedMultiLineString32 = boundary.to_nested_multi_linestring().unwrap();
    let back: Boundary<u32> = nested.try_into().unwrap();

    assert_boundary_consistent(&back);
    assert_eq!(back.vertices(), boundary.vertices());
    assert_eq!(back.rings(), boundary.rings());
}

#[test]
fn l1_nested_to_flat_to_nested() {
    let original: BoundaryNestedMultiLineString32 = vec![vec![0u32, 1], vec![1, 2, 3]];
    let flat: Boundary<u32> = original.clone().try_into().unwrap();
    let back = flat.to_nested_multi_linestring().unwrap();
    assert_eq!(back, original, "round-trip must preserve topology");
}

// ---------------------------------------------------------------------------
// S1 round-trip
// ---------------------------------------------------------------------------

#[test]
fn s1_boundary_consistent() {
    let S1Result { model, handle, .. } = build_s1(GeometryType::MultiSurface);
    let boundary = model.get_geometry(handle).unwrap().boundaries().unwrap();
    assert_boundary_consistent(boundary);
}

#[test]
fn s1_flat_to_nested_to_flat() {
    let S1Result { model, handle, .. } = build_s1(GeometryType::MultiSurface);
    let boundary = model.get_geometry(handle).unwrap().boundaries().unwrap();

    let nested: BoundaryNestedMultiOrCompositeSurface32 =
        boundary.to_nested_multi_or_composite_surface().unwrap();
    let back: Boundary<u32> = nested.try_into().unwrap();

    assert_boundary_consistent(&back);
    assert_eq!(back.vertices(), boundary.vertices());
    assert_eq!(back.rings(), boundary.rings());
    assert_eq!(back.surfaces(), boundary.surfaces());
}

#[test]
fn s1_nested_to_flat_to_nested() {
    // surface 0: outer ring [0,1,4] + inner ring [0,2,1]
    // surface 1: outer ring [2,3,4,5]
    let original: BoundaryNestedMultiOrCompositeSurface32 = vec![
        vec![vec![0u32, 1, 4], vec![0, 2, 1]],
        vec![vec![2, 3, 4, 5]],
    ];
    let flat: Boundary<u32> = original.clone().try_into().unwrap();
    assert_boundary_consistent(&flat);
    // 2 surfaces
    assert_eq!(flat.surfaces().len(), 2);
    // 3 rings
    assert_eq!(flat.rings().len(), 3);

    let back = flat.to_nested_multi_or_composite_surface().unwrap();
    assert_eq!(back, original, "round-trip must preserve topology");
}

#[test]
fn s1_inner_ring_attached_to_correct_surface() {
    let original: BoundaryNestedMultiOrCompositeSurface32 = vec![
        vec![vec![0u32, 1, 4], vec![0, 2, 1]], // surface 0: 1 outer + 1 inner
        vec![vec![2, 3, 4, 5]],                // surface 1: 1 outer only
    ];
    let flat: Boundary<u32> = original.clone().try_into().unwrap();
    let back = flat.to_nested_multi_or_composite_surface().unwrap();

    assert_eq!(
        back[0].len(),
        2,
        "surface 0 must have 2 rings (outer + inner)"
    );
    assert_eq!(back[1].len(), 1, "surface 1 must have 1 ring (outer only)");
}

// ---------------------------------------------------------------------------
// D1 round-trip
// ---------------------------------------------------------------------------

#[test]
fn d1_boundary_consistent() {
    let D1Result { model, handle } = build_d1();
    let boundary = model.get_geometry(handle).unwrap().boundaries().unwrap();
    assert_boundary_consistent(boundary);
}

#[test]
fn d1_flat_to_nested_to_flat() {
    let D1Result { model, handle } = build_d1();
    let boundary = model.get_geometry(handle).unwrap().boundaries().unwrap();

    let nested: BoundaryNestedSolid32 = boundary.to_nested_solid().unwrap();
    let back: Boundary<u32> = nested.try_into().unwrap();

    assert_boundary_consistent(&back);
    assert_eq!(back.vertices(), boundary.vertices());
    assert_eq!(back.rings(), boundary.rings());
    assert_eq!(back.surfaces(), boundary.surfaces());
    assert_eq!(back.shells(), boundary.shells());
}

#[test]
fn d1_nested_to_flat_to_nested() {
    // Solid: outer shell with 2 surfaces, inner shell with 2 surfaces
    let original: BoundaryNestedSolid32 = vec![
        vec![vec![vec![0u32, 1, 2]], vec![vec![2, 3, 4]]],
        vec![vec![vec![4u32, 5, 0]], vec![vec![1, 6, 7]]],
    ];
    let flat: Boundary<u32> = original.clone().try_into().unwrap();
    assert_boundary_consistent(&flat);
    assert_eq!(flat.shells().len(), 2);
    assert_eq!(flat.surfaces().len(), 4);

    let back = flat.to_nested_solid().unwrap();
    assert_eq!(back, original);
}

// ---------------------------------------------------------------------------
// MS1 round-trip
// ---------------------------------------------------------------------------

#[test]
fn ms1_boundary_consistent() {
    let MS1Result { model, handle } = build_ms1(GeometryType::MultiSolid);
    let boundary = model.get_geometry(handle).unwrap().boundaries().unwrap();
    assert_boundary_consistent(boundary);
}

#[test]
fn ms1_flat_to_nested_to_flat() {
    let MS1Result { model, handle } = build_ms1(GeometryType::MultiSolid);
    let boundary = model.get_geometry(handle).unwrap().boundaries().unwrap();

    let nested: BoundaryNestedMultiOrCompositeSolid32 =
        boundary.to_nested_multi_or_composite_solid().unwrap();
    let back: Boundary<u32> = nested.try_into().unwrap();

    assert_boundary_consistent(&back);
    assert_eq!(back.vertices(), boundary.vertices());
    assert_eq!(back.rings(), boundary.rings());
    assert_eq!(back.surfaces(), boundary.surfaces());
    assert_eq!(back.shells(), boundary.shells());
    assert_eq!(back.solids(), boundary.solids());
}

#[test]
fn ms1_nested_to_flat_preserves_solid_ordering() {
    let original: BoundaryNestedMultiOrCompositeSolid32 = vec![
        // Solid 0
        vec![vec![vec![vec![0u32, 1, 2]], vec![vec![0, 2, 1]]]],
        // Solid 1
        vec![vec![vec![vec![3u32, 4, 5]], vec![vec![3, 5, 4]]]],
    ];
    let flat: Boundary<u32> = original.clone().try_into().unwrap();
    assert_boundary_consistent(&flat);
    assert_eq!(flat.solids().len(), 2, "2 solids");
    assert_eq!(flat.shells().len(), 2, "2 shells total");

    let back = flat.to_nested_multi_or_composite_solid().unwrap();
    assert_eq!(back, original, "solid ordering preserved");
}

// ---------------------------------------------------------------------------
// T1 round-trip (template geometry)
// ---------------------------------------------------------------------------

#[test]
fn t1_boundary_consistent() {
    let T1Result {
        model,
        template_handle,
    } = build_t1();
    let boundary = model
        .get_geometry_template(template_handle)
        .unwrap()
        .boundaries()
        .unwrap();
    assert_boundary_consistent(boundary);
}

#[test]
fn t1_flat_to_nested_to_flat() {
    let T1Result {
        model,
        template_handle,
    } = build_t1();
    let boundary = model
        .get_geometry_template(template_handle)
        .unwrap()
        .boundaries()
        .unwrap();

    let nested: BoundaryNestedMultiOrCompositeSurface32 =
        boundary.to_nested_multi_or_composite_surface().unwrap();
    let back: Boundary<u32> = nested.try_into().unwrap();

    assert_boundary_consistent(&back);
    assert_eq!(back.vertices(), boundary.vertices());
    assert_eq!(back.rings(), boundary.rings());
    assert_eq!(back.surfaces(), boundary.surfaces());
}
