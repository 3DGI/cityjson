//! Test family 1: Accept canonical fixtures.
//! Test family 5: Accept dense semantic and material maps.
//! Test family 7: Validate resource references.
//! Test family 8: Accept dense texture maps.
//!
//! All tests in this file should pass with the current production code.

use super::fixtures::*;
use cityjson::v2_0::*;

// ---------------------------------------------------------------------------
// Family 1: Accept canonical fixtures
// ---------------------------------------------------------------------------

#[test]
fn accept_p1() {
    let P1Result { model, handle } = build_p1();
    let geom = model.get_geometry(handle).unwrap();
    assert_eq!(geom.type_geometry(), &GeometryType::MultiPoint);

    // 3 vertices in the boundary
    let boundary = geom.boundaries().unwrap();
    assert_eq!(boundary.vertices().len(), 3);
    assert!(boundary.rings().is_empty());
    assert!(boundary.surfaces().is_empty());
    assert!(boundary.shells().is_empty());
    assert!(boundary.solids().is_empty());
    assert_eq!(boundary.check_type(), BoundaryType::MultiPoint);
}

#[test]
fn accept_l1() {
    let L1Result { model, handle } = build_l1();
    let geom = model.get_geometry(handle).unwrap();
    assert_eq!(geom.type_geometry(), &GeometryType::MultiLineString);

    let boundary = geom.boundaries().unwrap();
    // 4 vertices total (v0,v1 in ls0; v1,v2,v3 in ls1 = 5 total with shared v1)
    assert_eq!(boundary.vertices().len(), 5);
    // 2 rings (linestrings)
    assert_eq!(boundary.rings().len(), 2);
    assert!(boundary.surfaces().is_empty());
    assert_eq!(boundary.check_type(), BoundaryType::MultiLineString);
}

#[test]
fn accept_s1_multisurface() {
    let S1Result { model, handle, .. } = build_s1(GeometryType::MultiSurface);
    let geom = model.get_geometry(handle).unwrap();
    assert_eq!(geom.type_geometry(), &GeometryType::MultiSurface);

    let boundary = geom.boundaries().unwrap();
    // 2 surfaces
    assert_eq!(boundary.surfaces().len(), 2);
    // 3 rings: r0 (outer s0), r1 (inner s0), r2 (outer s1)
    assert_eq!(boundary.rings().len(), 3);
    // No shells or solids for MultiSurface
    assert!(boundary.shells().is_empty());
    assert!(boundary.solids().is_empty());
    assert_eq!(boundary.check_type(), BoundaryType::MultiOrCompositeSurface);
}

#[test]
fn accept_s1_composite_surface() {
    let S1Result { model, handle, .. } = build_s1(GeometryType::CompositeSurface);
    let geom = model.get_geometry(handle).unwrap();
    assert_eq!(geom.type_geometry(), &GeometryType::CompositeSurface);

    let boundary = geom.boundaries().unwrap();
    assert_eq!(boundary.surfaces().len(), 2);
    assert_eq!(boundary.rings().len(), 3);
    assert!(boundary.shells().is_empty());
    assert!(boundary.solids().is_empty());
}

#[test]
fn accept_d1() {
    let D1Result { model, handle } = build_d1();
    let geom = model.get_geometry(handle).unwrap();
    assert_eq!(geom.type_geometry(), &GeometryType::Solid);

    let boundary = geom.boundaries().unwrap();
    // 4 surfaces, 4 rings, 2 shells, no solids
    assert_eq!(boundary.surfaces().len(), 4);
    assert_eq!(boundary.rings().len(), 4);
    assert_eq!(boundary.shells().len(), 2);
    assert!(boundary.solids().is_empty());
    assert_eq!(boundary.check_type(), BoundaryType::Solid);
}

#[test]
fn accept_ms1_multisolid() {
    let MS1Result { model, handle } = build_ms1(GeometryType::MultiSolid);
    let geom = model.get_geometry(handle).unwrap();
    assert_eq!(geom.type_geometry(), &GeometryType::MultiSolid);

    let boundary = geom.boundaries().unwrap();
    // 4 surfaces, 4 rings, 2 shells, 2 solids
    assert_eq!(boundary.surfaces().len(), 4);
    assert_eq!(boundary.rings().len(), 4);
    assert_eq!(boundary.shells().len(), 2);
    assert_eq!(boundary.solids().len(), 2);
    assert_eq!(boundary.check_type(), BoundaryType::MultiOrCompositeSolid);
}

#[test]
fn accept_ms1_composite_solid() {
    let MS1Result { model, handle } = build_ms1(GeometryType::CompositeSolid);
    let geom = model.get_geometry(handle).unwrap();
    assert_eq!(geom.type_geometry(), &GeometryType::CompositeSolid);

    let boundary = geom.boundaries().unwrap();
    assert_eq!(boundary.surfaces().len(), 4);
    assert_eq!(boundary.shells().len(), 2);
    assert_eq!(boundary.solids().len(), 2);
}

#[test]
fn accept_t1() {
    let T1Result {
        model,
        template_handle,
    } = build_t1();
    let geom = model.get_geometry_template(template_handle).unwrap();
    assert_eq!(geom.type_geometry(), &GeometryType::MultiSurface);

    let boundary = geom.boundaries().unwrap();
    assert_eq!(boundary.surfaces().len(), 2);
    assert_eq!(boundary.rings().len(), 2);
    assert!(boundary.shells().is_empty());
}

#[test]
fn accept_i1() {
    let I1Result {
        model,
        template_handle,
        instance_handle,
    } = build_i1();
    let geom = model.get_geometry(instance_handle).unwrap();
    assert_eq!(geom.type_geometry(), &GeometryType::GeometryInstance);

    // Instance has no boundary of its own
    assert!(geom.boundaries().is_none());

    // Instance references the template
    let instance_view = geom.instance().unwrap();
    assert_eq!(instance_view.template(), template_handle);
}

// ---------------------------------------------------------------------------
// Family 5: Accept dense semantic maps
// ---------------------------------------------------------------------------

#[test]
fn p1_semantic_map_uses_points_bucket() {
    let P1Result { model, handle } = build_p1();
    let geom = model.get_geometry(handle).unwrap();
    let sem = geom.semantics().expect("P1 must have semantics");

    // Only the points bucket is populated
    assert_eq!(sem.points().len(), 3, "dense: one slot per point");
    assert!(
        sem.linestrings().is_empty(),
        "linestrings bucket unused for MultiPoint"
    );
    assert!(
        sem.surfaces().is_empty(),
        "surfaces bucket unused for MultiPoint"
    );

    // p0=Roof, p1=null, p2=Wall
    assert!(sem.points()[0].is_some(), "point 0 → Roof");
    assert!(sem.points()[1].is_none(), "point 1 → null placeholder");
    assert!(sem.points()[2].is_some(), "point 2 → Wall");
}

#[test]
fn l1_semantic_map_uses_linestrings_bucket() {
    let L1Result { model, handle } = build_l1();
    let geom = model.get_geometry(handle).unwrap();
    let sem = geom.semantics().expect("L1 must have semantics");

    assert_eq!(sem.linestrings().len(), 2, "dense: one slot per linestring");
    assert!(sem.points().is_empty());
    assert!(sem.surfaces().is_empty());

    assert!(sem.linestrings()[0].is_none(), "ls0 → null");
    assert!(sem.linestrings()[1].is_some(), "ls1 → Roof");
}

#[test]
fn s1_semantic_map_uses_surfaces_bucket() {
    let S1Result { model, handle, .. } = build_s1(GeometryType::MultiSurface);
    let geom = model.get_geometry(handle).unwrap();
    let sem = geom.semantics().expect("S1 must have semantics");

    assert_eq!(sem.surfaces().len(), 2, "dense: one slot per surface");
    assert!(sem.points().is_empty());
    assert!(sem.linestrings().is_empty());

    assert!(sem.surfaces()[0].is_some(), "s0 → Roof");
    assert!(sem.surfaces()[1].is_some(), "s1 → Wall");
}

#[test]
fn d1_semantic_map_uses_surfaces_bucket() {
    let D1Result { model, handle } = build_d1();
    let geom = model.get_geometry(handle).unwrap();
    let sem = geom.semantics().expect("D1 must have semantics");

    assert_eq!(sem.surfaces().len(), 4, "dense: one slot per surface");
    assert!(sem.points().is_empty());
    assert!(sem.linestrings().is_empty());

    assert!(sem.surfaces()[0].is_some(), "s0 → Roof");
    assert!(sem.surfaces()[1].is_some(), "s1 → Wall");
    assert!(sem.surfaces()[2].is_some(), "s2 → Ground");
    assert!(sem.surfaces()[3].is_none(), "s3 → null placeholder");
}

#[test]
fn ms1_semantic_map_uses_surfaces_bucket() {
    let MS1Result { model, handle } = build_ms1(GeometryType::MultiSolid);
    let geom = model.get_geometry(handle).unwrap();
    let sem = geom.semantics().expect("MS1 must have semantics");

    assert_eq!(
        sem.surfaces().len(),
        4,
        "dense: one slot per surface across both solids"
    );
    assert!(sem.points().is_empty());
    assert!(sem.linestrings().is_empty());

    assert!(sem.surfaces()[0].is_some(), "s0 → Roof");
    assert!(sem.surfaces()[1].is_some(), "s1 → Wall");
    assert!(sem.surfaces()[2].is_some(), "s2 → Ground");
    assert!(sem.surfaces()[3].is_none(), "s3 → null placeholder");
}

// ---------------------------------------------------------------------------
// Family 7: Resource reference validity
// ---------------------------------------------------------------------------

#[test]
fn s1_semantic_references_resolve() {
    let S1Result { model, handle, .. } = build_s1(GeometryType::MultiSurface);
    let geom = model.get_geometry(handle).unwrap();
    let sem = geom.semantics().unwrap();

    // Every non-null semantic handle must resolve to a pool entry
    for (i, opt) in sem.surfaces().iter().enumerate() {
        if let Some(handle) = opt {
            assert!(
                model.get_semantic(*handle).is_some(),
                "semantic for surface {i} must resolve"
            );
        }
    }
}

#[test]
fn d1_semantic_references_resolve() {
    let D1Result { model, handle } = build_d1();
    let geom = model.get_geometry(handle).unwrap();
    let sem = geom.semantics().unwrap();

    for (i, opt) in sem.surfaces().iter().enumerate() {
        if let Some(handle) = opt {
            assert!(
                model.get_semantic(*handle).is_some(),
                "semantic for surface {i} must resolve"
            );
        }
    }
}

#[test]
fn s1_material_references_resolve() {
    let S1Result {
        model,
        handle,
        theme,
    } = build_s1(GeometryType::MultiSurface);
    let geom = model.get_geometry(handle).unwrap();
    let mats = geom.materials().unwrap();

    let (_theme_name, mat_map) = mats.first().unwrap();
    for (i, opt) in mat_map.surfaces().iter().enumerate() {
        if let Some(h) = opt {
            assert!(
                model.get_material(*h).is_some(),
                "material for surface {i} must resolve"
            );
        }
    }
    let _ = theme;
}

#[test]
fn s1_texture_references_resolve() {
    let S1Result { model, handle, .. } = build_s1(GeometryType::MultiSurface);
    let geom = model.get_geometry(handle).unwrap();
    let textures = geom.textures().unwrap();

    let (_theme_name, tex_map) = textures.first().unwrap();
    for (i, opt) in tex_map.ring_textures().iter().enumerate() {
        if let Some(h) = *opt {
            assert!(
                model.get_texture(h).is_some(),
                "texture for ring {i} must resolve"
            );
        }
    }
}

#[test]
fn s1_uv_references_resolve() {
    let S1Result { model, handle, .. } = build_s1(GeometryType::MultiSurface);
    let geom = model.get_geometry(handle).unwrap();
    let textures = geom.textures().unwrap();

    let (_theme_name, tex_map) = textures.first().unwrap();
    for (i, opt) in tex_map.vertices().iter().enumerate() {
        if let Some(uv_ref) = opt {
            assert!(
                model.get_uv_coordinate(*uv_ref).is_some(),
                "uv for boundary occurrence {i} must resolve"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Family 8: Accept dense texture maps
// ---------------------------------------------------------------------------

#[test]
fn s1_texture_map_is_dense_and_boundary_aligned() {
    let S1Result { model, handle, .. } = build_s1(GeometryType::MultiSurface);
    let geom = model.get_geometry(handle).unwrap();
    let boundary = geom.boundaries().expect("S1 must have boundaries");
    let textures = geom.textures().expect("S1 must have textures");

    let (_theme, tex_map) = textures.first().unwrap();
    assert_eq!(tex_map.rings().len(), boundary.rings().len());
    assert_eq!(tex_map.ring_textures().len(), boundary.rings().len());
    assert_eq!(tex_map.vertices().len(), boundary.vertices().len());
    assert_eq!(tex_map.rings(), boundary.rings());

    assert!(tex_map.ring_textures()[0].is_some(), "ring 0 textured");
    assert!(tex_map.ring_textures()[1].is_none(), "ring 1 untextured");
    assert!(tex_map.ring_textures()[2].is_some(), "ring 2 textured");

    assert!(tex_map.vertices()[0..3].iter().all(Option::is_some));
    assert!(tex_map.vertices()[3..6].iter().all(Option::is_none));
    assert!(tex_map.vertices()[6..10].iter().all(Option::is_some));
}

#[test]
fn s1_reused_geometric_vertex_can_have_different_uvs_per_ring_occurrence() {
    let S1Result { model, handle, .. } = build_s1(GeometryType::MultiSurface);
    let geom = model.get_geometry(handle).unwrap();
    let textures = geom.textures().expect("S1 must have textures");

    let (_theme, tex_map) = textures.first().unwrap();
    let ring0_v4_uv = tex_map.vertices()[2].expect("ring 0 v4 occurrence should have a UV");
    let ring2_v4_uv = tex_map.vertices()[8].expect("ring 2 v4 occurrence should have a UV");

    assert_ne!(ring0_v4_uv, ring2_v4_uv);
}

#[test]
fn s1_material_map_covers_all_surfaces() {
    let S1Result { model, handle, .. } = build_s1(GeometryType::MultiSurface);
    let geom = model.get_geometry(handle).unwrap();
    let mats = geom.materials().expect("S1 must have materials");

    let (_theme, mat_map) = mats.first().unwrap();
    assert_eq!(
        mat_map.surfaces().len(),
        2,
        "material map must have one slot per surface"
    );
    assert!(mat_map.surfaces()[0].is_some(), "s0 has material");
    assert!(mat_map.surfaces()[1].is_none(), "s1 has no material (null)");
}
