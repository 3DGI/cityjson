//! Memory-focused benchmarks that compare heap allocation sizes between backends
//!
//! This benchmark builds CityModels with both backend implementations and measures the heap
//! allocated memory for each. This helps understand the memory overhead of different
//! backend architectures.
//!
//! ## Running Benchmarks
//!
//! Run with specific backend:
//! ```bash
//! # Default backend (flattened representation)
//! cargo bench --bench memory --features backend-default
//!
//! # Nested backend (JSON-like representation)
//! cargo bench --bench memory --features backend-nested
//!
//! # Both backends (for comparison)
//! cargo bench --bench memory --features backend-both
//! ```
//!
//! View the generated dhat-heap.json at https://nnethercote.github.io/dh_view/dh_view.html

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::Rng;
use std::hint::black_box;

// Enable dhat heap profiling for the entire benchmark
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

// ==================== DEFAULT BACKEND BENCHMARKS ====================

#[cfg(feature = "backend-default")]
mod default_benches {
    use super::*;

    use cityjson::prelude::*;
    use cityjson::v2_0::*;

    /// Build a CityModel with the specified vertex index type and number of cityobjects.
    /// Each cityobject will have a solid geometry with 8 vertices (a cube).
    fn build_model<VR: VertexRef>(
        n_cityobjects: usize,
    ) -> CityModel<VR, ResourceId32, OwnedStringStorage> {
        let mut model =
            CityModel::<VR, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        let mut rng = rand::rng();

        // Set basic metadata
        let metadata = model.metadata_mut();
        metadata.set_identifier(CityModelIdentifier::new("memory-benchmark".to_string()));
        metadata.set_reference_system(CRS::new(
            "https://www.opengis.net/def/crs/EPSG/0/2355".to_string(),
        ));

        for i in 0..n_cityobjects {
            // Generate 8 random vertices for a cube-like solid
            let vertices: Vec<_> = (0..8)
                .map(|_| {
                    let x = rng.random_range(0..100000);
                    let y = rng.random_range(0..100000);
                    let z = rng.random_range(0..1000);
                    model.add_vertex(QuantizedCoordinate::new(x, y, z)).unwrap()
                })
                .collect();

            // Create a CityObject
            let mut cityobject =
                CityObject::new(format!("building-{:06}", i), CityObjectType::Building);

            // Build a solid geometry using GeometryBuilder
            {
                let mut geometry_builder =
                    GeometryBuilder::new(&mut model, GeometryType::Solid, BuilderMode::Regular)
                        .with_lod(LoD::LoD2);

                // Add vertices to the builder
                let bv: Vec<_> = vertices
                    .iter()
                    .map(|&v| geometry_builder.add_vertex(v))
                    .collect();

                // Create 6 faces of the cube (simplified solid)
                // Bottom face
                let ring_bottom = geometry_builder
                    .add_ring(&[bv[0], bv[1], bv[2], bv[3]])
                    .unwrap();
                let surface_bottom = geometry_builder.start_surface();
                geometry_builder
                    .add_surface_outer_ring(ring_bottom)
                    .unwrap();

                // Top face
                let ring_top = geometry_builder
                    .add_ring(&[bv[4], bv[7], bv[6], bv[5]])
                    .unwrap();
                let surface_top = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_top).unwrap();

                // Front face
                let ring_front = geometry_builder
                    .add_ring(&[bv[0], bv[1], bv[5], bv[4]])
                    .unwrap();
                let surface_front = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_front).unwrap();

                // Right face
                let ring_right = geometry_builder
                    .add_ring(&[bv[1], bv[2], bv[6], bv[5]])
                    .unwrap();
                let surface_right = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_right).unwrap();

                // Back face
                let ring_back = geometry_builder
                    .add_ring(&[bv[2], bv[3], bv[7], bv[6]])
                    .unwrap();
                let surface_back = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_back).unwrap();

                // Left face
                let ring_left = geometry_builder
                    .add_ring(&[bv[3], bv[0], bv[4], bv[7]])
                    .unwrap();
                let surface_left = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_left).unwrap();

                // Create the shell from all surfaces
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

                // Build the geometry
                let geometry_ref = geometry_builder.build().unwrap();

                // Add geometry to the CityObject
                cityobject.geometry_mut().push(geometry_ref);
            }

            // Add the CityObject to the model
            model.cityobjects_mut().add(cityobject);
        }

        model
    }

    pub fn bench_memory_u32(c: &mut Criterion) {
        let mut group = c.benchmark_group("memory");
        let n_cityobjects = 7_000;

        group.bench_function(BenchmarkId::new("default/u32", n_cityobjects), |b| {
            b.iter(|| {
                let model = build_model::<u32>(black_box(n_cityobjects));
                black_box(model);
            });
        });

        group.finish();
    }
}

// ==================== NESTED BACKEND BENCHMARKS ====================

#[cfg(feature = "backend-nested")]
mod nested_benches {
    use super::*;
    use cityjson::backend::nested;
    use cityjson::prelude::*;

    /// Build a CityModel with the nested backend and the specified number of cityobjects.
    /// Each cityobject will have a solid geometry with 8 vertices (a cube).
    fn build_model(n_cityobjects: usize) -> nested::CityModel<OwnedStringStorage, ResourceId32> {
        let mut model = nested::CityModel::<OwnedStringStorage, ResourceId32>::new(CityModelType::CityJSON);
        let mut rng = rand::rng();

        // Set basic metadata
        let metadata = model.metadata_mut();
        metadata.set_identifier(CityModelIdentifier::new("memory-benchmark".to_string()));
        metadata.set_reference_system(CRS::new(
            "https://www.opengis.net/def/crs/EPSG/0/2355".to_string(),
        ));

        for i in 0..n_cityobjects {
            // Generate 8 random vertices for a cube-like solid
            let vertices: Vec<_> = (0..8)
                .map(|_| {
                    let x = rng.random_range(0..100000);
                    let y = rng.random_range(0..100000);
                    let z = rng.random_range(0..1000);
                    model.add_vertex(QuantizedCoordinate::new(x, y, z)).unwrap()
                })
                .collect();

            // Create a CityObject ID
            let co_id = format!("building-{:06}", i);

            // Create and add a CityObject
            let cityobject = nested::CityObject::new(nested::cityobject::CityObjectType::Building);
            model.add_cityobject(co_id.clone(), cityobject);

            // Build a solid geometry using GeometryBuilder
            {
                let mut geometry_builder = nested::GeometryBuilder::new(
                    &mut model,
                    GeometryType::Solid,
                    nested::BuilderMode::Regular,
                )
                .with_lod(LoD::LoD2);

                // Add vertices to the builder
                for &v in &vertices {
                    geometry_builder.add_vertex(v).unwrap();
                }

                // Create 6 faces of the cube (simplified solid)
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

                // Create the shell from all surfaces
                geometry_builder.start_shell().unwrap();
                geometry_builder.add_shell_surface(surface_bottom).unwrap();
                geometry_builder.add_shell_surface(surface_top).unwrap();
                geometry_builder.add_shell_surface(surface_front).unwrap();
                geometry_builder.add_shell_surface(surface_right).unwrap();
                geometry_builder.add_shell_surface(surface_back).unwrap();
                geometry_builder.add_shell_surface(surface_left).unwrap();
                geometry_builder.end_shell().unwrap();

                // Build the geometry
                let geometry = geometry_builder.build().unwrap();

                // Add geometry to the CityObject
                model.add_geometry_to_cityobject(&co_id, geometry).unwrap();
            }
        }

        model
    }

    pub fn bench_memory_nested(c: &mut Criterion) {
        let mut group = c.benchmark_group("memory");
        let n_cityobjects = 7_000;

        group.bench_function(BenchmarkId::new("nested", n_cityobjects), |b| {
            b.iter(|| {
                let model = build_model(black_box(n_cityobjects));
                black_box(model);
            });
        });

        group.finish();
    }
}

// ==================== CRITERION GROUPS ====================

#[cfg(all(feature = "backend-default", not(feature = "backend-nested")))]
criterion_group!(benches, default_benches::bench_memory_u32);

#[cfg(all(feature = "backend-nested", not(feature = "backend-default")))]
criterion_group!(benches, nested_benches::bench_memory_nested);

#[cfg(all(feature = "backend-default", feature = "backend-nested"))]
criterion_group!(
    benches,
    default_benches::bench_memory_u32,
    nested_benches::bench_memory_nested
);

criterion_main!(benches);
