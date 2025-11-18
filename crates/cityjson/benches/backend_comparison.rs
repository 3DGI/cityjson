//! Benchmark comparing default and nested backends.
//!
//! This benchmark provides head-to-head comparisons between the two backend implementations.
//!
//! Run with:
//! - `cargo bench --bench backend_comparison --features backend-default`
//! - `cargo bench --bench backend_comparison --features backend-nested`
//! - `cargo bench --bench backend_comparison --features backend-both`

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

// ==================== DEFAULT BACKEND BENCHMARKS ====================

#[cfg(feature = "backend-default")]
mod default_benches {
    use super::*;

    use cityjson::prelude::*;
    use cityjson::v2_0::*;

    fn build_simple_solid(n_buildings: usize) {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // Pre-create vertices for a cube
        let v0 = model.add_vertex(QuantizedCoordinate::new(0, 0, 0)).unwrap();
        let v1 = model
            .add_vertex(QuantizedCoordinate::new(1000, 0, 0))
            .unwrap();
        let v2 = model
            .add_vertex(QuantizedCoordinate::new(1000, 1000, 0))
            .unwrap();
        let v3 = model
            .add_vertex(QuantizedCoordinate::new(0, 1000, 0))
            .unwrap();
        let v4 = model
            .add_vertex(QuantizedCoordinate::new(0, 0, 500))
            .unwrap();
        let v5 = model
            .add_vertex(QuantizedCoordinate::new(1000, 0, 500))
            .unwrap();
        let v6 = model
            .add_vertex(QuantizedCoordinate::new(1000, 1000, 500))
            .unwrap();
        let v7 = model
            .add_vertex(QuantizedCoordinate::new(0, 1000, 500))
            .unwrap();

        for i in 0..n_buildings {
            let mut cityobject =
                CityObject::new(format!("building-{}", i), CityObjectType::Building);

            // Build a simple cube geometry
            let mut geometry_builder =
                GeometryBuilder::new(&mut model, GeometryType::Solid, BuilderMode::Regular)
                    .with_lod(LoD::LoD2);

            let bv0 = geometry_builder.add_vertex(v0);
            let bv1 = geometry_builder.add_vertex(v1);
            let bv2 = geometry_builder.add_vertex(v2);
            let bv3 = geometry_builder.add_vertex(v3);
            let bv4 = geometry_builder.add_vertex(v4);
            let bv5 = geometry_builder.add_vertex(v5);
            let bv6 = geometry_builder.add_vertex(v6);
            let bv7 = geometry_builder.add_vertex(v7);

            // Bottom face
            let ring_bottom = geometry_builder.add_ring(&[bv0, bv1, bv2, bv3]).unwrap();
            let surface_bottom = geometry_builder.start_surface();
            geometry_builder
                .add_surface_outer_ring(ring_bottom)
                .unwrap();

            // Top face
            let ring_top = geometry_builder.add_ring(&[bv4, bv7, bv6, bv5]).unwrap();
            let surface_top = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_top).unwrap();

            // Front face
            let ring_front = geometry_builder.add_ring(&[bv0, bv1, bv5, bv4]).unwrap();
            let surface_front = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_front).unwrap();

            // Right face
            let ring_right = geometry_builder.add_ring(&[bv1, bv2, bv6, bv5]).unwrap();
            let surface_right = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_right).unwrap();

            // Back face
            let ring_back = geometry_builder.add_ring(&[bv2, bv3, bv7, bv6]).unwrap();
            let surface_back = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_back).unwrap();

            // Left face
            let ring_left = geometry_builder.add_ring(&[bv3, bv0, bv4, bv7]).unwrap();
            let surface_left = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_left).unwrap();

            // Create the shell
            geometry_builder
                .add_shell(&[
                    surface_bottom,
                    surface_top,
                    surface_front,
                    surface_right,
                    surface_back,
                    surface_left,
                ])
                .unwrap();

            let geometry_ref = geometry_builder.build().unwrap();
            cityobject.geometry_mut().push(geometry_ref);

            model.cityobjects_mut().add(cityobject);
        }
    }

    pub fn bench_build_solids(c: &mut Criterion) {
        let mut group = c.benchmark_group("backend_comparison");

        for n in [100, 1000, 5000].iter() {
            group.throughput(Throughput::Elements(*n as u64));
            group.bench_with_input(BenchmarkId::new("default/build_solids", n), n, |b, &n| {
                b.iter(|| {
                    build_simple_solid(black_box(n));
                });
            });
        }

        group.finish();
    }
}

// ==================== NESTED BACKEND BENCHMARKS ====================

#[cfg(feature = "backend-nested")]
mod nested_benches {
    use super::*;
    use cityjson::backend::nested;
    use cityjson::prelude::*;

    fn build_simple_solid(n_buildings: usize) {
        let mut model = nested::CityModel::<OwnedStringStorage>::new(CityModelType::CityJSON);

        // Pre-create vertices for a cube
        let v0 = model.add_vertex(QuantizedCoordinate::new(0, 0, 0)).unwrap();
        let v1 = model
            .add_vertex(QuantizedCoordinate::new(1000, 0, 0))
            .unwrap();
        let v2 = model
            .add_vertex(QuantizedCoordinate::new(1000, 1000, 0))
            .unwrap();
        let v3 = model
            .add_vertex(QuantizedCoordinate::new(0, 1000, 0))
            .unwrap();
        let v4 = model
            .add_vertex(QuantizedCoordinate::new(0, 0, 500))
            .unwrap();
        let v5 = model
            .add_vertex(QuantizedCoordinate::new(1000, 0, 500))
            .unwrap();
        let v6 = model
            .add_vertex(QuantizedCoordinate::new(1000, 1000, 500))
            .unwrap();
        let v7 = model
            .add_vertex(QuantizedCoordinate::new(0, 1000, 500))
            .unwrap();

        for i in 0..n_buildings {
            let co_id = format!("building-{}", i);
            let cityobject = nested::CityObject::new(nested::cityobject::CityObjectType::Building);

            // Add the cityobject first
            model.add_cityobject(co_id.clone(), cityobject);

            // Build a simple cube geometry
            let mut geometry_builder = nested::GeometryBuilder::new(
                &mut model,
                GeometryType::Solid,
                nested::BuilderMode::Regular,
            )
            .with_lod(LoD::LoD2);

            geometry_builder.add_vertex(v0).unwrap();
            geometry_builder.add_vertex(v1).unwrap();
            geometry_builder.add_vertex(v2).unwrap();
            geometry_builder.add_vertex(v3).unwrap();
            geometry_builder.add_vertex(v4).unwrap();
            geometry_builder.add_vertex(v5).unwrap();
            geometry_builder.add_vertex(v6).unwrap();
            geometry_builder.add_vertex(v7).unwrap();

            // Bottom face (indices 0-3)
            let ring_bottom = geometry_builder.add_ring(&[0, 1, 2, 3]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder
                .add_surface_outer_ring(ring_bottom)
                .unwrap();
            let surface_bottom = geometry_builder.end_surface().unwrap();

            // Top face (indices 4-7)
            let ring_top = geometry_builder.add_ring(&[4, 7, 6, 5]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder.add_surface_outer_ring(ring_top).unwrap();
            let surface_top = geometry_builder.end_surface().unwrap();

            // Front face
            let ring_front = geometry_builder.add_ring(&[0, 1, 5, 4]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder.add_surface_outer_ring(ring_front).unwrap();
            let surface_front = geometry_builder.end_surface().unwrap();

            // Right face
            let ring_right = geometry_builder.add_ring(&[1, 2, 6, 5]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder.add_surface_outer_ring(ring_right).unwrap();
            let surface_right = geometry_builder.end_surface().unwrap();

            // Back face
            let ring_back = geometry_builder.add_ring(&[2, 3, 7, 6]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder.add_surface_outer_ring(ring_back).unwrap();
            let surface_back = geometry_builder.end_surface().unwrap();

            // Left face
            let ring_left = geometry_builder.add_ring(&[3, 0, 4, 7]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder.add_surface_outer_ring(ring_left).unwrap();
            let surface_left = geometry_builder.end_surface().unwrap();

            // Create the shell
            geometry_builder.start_shell().unwrap();
            geometry_builder.add_shell_surface(surface_bottom).unwrap();
            geometry_builder.add_shell_surface(surface_top).unwrap();
            geometry_builder.add_shell_surface(surface_front).unwrap();
            geometry_builder.add_shell_surface(surface_right).unwrap();
            geometry_builder.add_shell_surface(surface_back).unwrap();
            geometry_builder.add_shell_surface(surface_left).unwrap();
            geometry_builder.end_shell().unwrap();

            // Build the geometry and add it to the cityobject
            let geometry = geometry_builder.build().unwrap();
            model.add_geometry_to_cityobject(&co_id, geometry).unwrap();
        }
    }

    pub fn bench_build_solids(c: &mut Criterion) {
        let mut group = c.benchmark_group("backend_comparison");

        for n in [100, 1000, 5000].iter() {
            group.throughput(Throughput::Elements(*n as u64));
            group.bench_with_input(BenchmarkId::new("nested/build_solids", n), n, |b, &n| {
                b.iter(|| {
                    build_simple_solid(black_box(n));
                });
            });
        }

        group.finish();
    }
}

// ==================== CRITERION GROUPS ====================

#[cfg(all(feature = "backend-default", not(feature = "backend-nested")))]
criterion_group!(benches, default_benches::bench_build_solids);

#[cfg(all(feature = "backend-nested", not(feature = "backend-default")))]
criterion_group!(benches, nested_benches::bench_build_solids);

#[cfg(all(feature = "backend-default", feature = "backend-nested"))]
criterion_group!(
    benches,
    default_benches::bench_build_solids,
    nested_benches::bench_build_solids
);

criterion_main!(benches);
