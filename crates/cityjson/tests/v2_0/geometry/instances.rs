//! Test family 11: Validate template geometry and `GeometryInstance` separation.
//!
//! Covers:
//! - positive acceptance of T1 and I1
//! - `GeometryInstance` stores no boundary or mapping payload of its own
//! - instance references an existing template
//! - instance has a valid reference point in the regular vertex pool
//! - instance has a valid 4x4 affine transform

use super::fixtures::*;
use cityjson::v2_0::*;

// ---------------------------------------------------------------------------
// Positive: I1 acceptance
// ---------------------------------------------------------------------------

#[test]
fn i1_instance_has_no_boundary_payload() {
    let I1Result {
        model,
        instance_handle,
        ..
    } = build_i1();
    let geom = model.get_geometry(instance_handle).unwrap();

    assert!(
        geom.boundaries().is_none(),
        "GeometryInstance has no boundary"
    );
    assert!(
        geom.semantics().is_none(),
        "GeometryInstance has no semantics"
    );
    assert!(
        geom.materials().is_none(),
        "GeometryInstance has no materials"
    );
    assert!(
        geom.textures().is_none(),
        "GeometryInstance has no textures"
    );
}

#[test]
fn i1_references_existing_template() {
    let I1Result {
        model,
        template_handle,
        instance_handle,
    } = build_i1();
    let geom = model.get_geometry(instance_handle).unwrap();
    let instance_view = geom.instance().unwrap();

    assert_eq!(
        instance_view.template(),
        template_handle,
        "instance must reference the correct template handle"
    );
    assert!(
        model
            .get_geometry_template(instance_view.template())
            .is_some(),
        "referenced template must exist in the model"
    );
}

#[test]
fn i1_reference_point_in_regular_vertex_pool() {
    let I1Result {
        model,
        instance_handle,
        ..
    } = build_i1();
    let geom = model.get_geometry(instance_handle).unwrap();
    let instance_view = geom.instance().unwrap();

    let ref_point_idx = instance_view.reference_point();
    assert!(
        model.get_vertex(ref_point_idx).is_some(),
        "reference point must be in the regular vertex pool"
    );
}

#[test]
fn i1_has_valid_transformation() {
    let I1Result {
        model,
        instance_handle,
        ..
    } = build_i1();
    let geom = model.get_geometry(instance_handle).unwrap();
    let instance_view = geom.instance().unwrap();

    let transform = instance_view.transformation();
    // Identity matrix: diagonal 1s, rest 0s
    let expected = AffineTransform3D::identity();
    assert_eq!(
        transform, expected,
        "transformation must match what was set"
    );
}

#[test]
fn t1_passes_same_boundary_checks_as_regular() {
    let T1Result {
        model,
        template_handle,
    } = build_t1();
    let geom = model.get_geometry_template(template_handle).unwrap();

    // Template has boundaries just like regular geometry
    assert!(geom.boundaries().is_some(), "template has boundary data");
    assert_eq!(geom.type_geometry(), &GeometryType::MultiSurface);

    let boundary = geom.boundaries().unwrap();
    assert!(boundary.is_consistent());
    assert_eq!(boundary.surfaces().len(), 2);
}

#[test]
fn t1_uses_template_vertex_pool() {
    let T1Result { model, .. } = build_t1();
    // Template builder adds vertices to the template pool, not the regular pool
    assert_eq!(
        model.vertices().len(),
        0,
        "regular vertex pool must be empty (T1 uses template pool)"
    );
    assert!(
        !model.template_vertices().is_empty(),
        "template vertex pool must have vertices"
    );
}

// ---------------------------------------------------------------------------
// Resolve: instance resolves to template geometry
// ---------------------------------------------------------------------------

#[test]
fn i1_resolves_to_template_geometry_type() {
    let I1Result {
        model,
        instance_handle,
        ..
    } = build_i1();
    let resolved = model.resolve_geometry(instance_handle).unwrap();
    assert_eq!(
        resolved.type_geometry(),
        &GeometryType::MultiSurface,
        "resolved instance must have the template's geometry type"
    );
}

// ---------------------------------------------------------------------------
// Verify template and regular geometry stay in separate pools
// ---------------------------------------------------------------------------

#[test]
fn template_geometry_not_in_regular_pool() {
    let T1Result {
        model,
        template_handle,
    } = build_t1();

    // Template geometry is not accessible as a regular geometry
    // (GeometryTemplateHandle ≠ GeometryHandle)
    assert_eq!(
        model.geometry_count(),
        0,
        "regular geometry pool must be empty when only a template is built"
    );
    assert_eq!(
        model.geometry_template_count(),
        1,
        "template geometry pool must have 1 entry"
    );
    let _ = template_handle;
}

#[test]
fn regular_geometry_not_in_template_pool() {
    let P1Result { model, .. } = build_p1();
    assert_eq!(
        model.geometry_template_count(),
        0,
        "template pool must be empty when only regular geometry is built"
    );
    assert_eq!(model.geometry_count(), 1);
}
