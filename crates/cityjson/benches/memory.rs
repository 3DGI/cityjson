//! Memory-focused benchmarks that compare heap allocation sizes across different vertex index types
//!
//! This benchmark builds CityModels with three different vertex index sizes (u16, u32, u64)
//! and measures the heap allocated memory for each. This helps understand the memory overhead
//! of different index types.

use cityjson::prelude::*;
use cityjson::v2_0::*;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::Rng;
use std::hint::black_box;

// Enable dhat heap profiling for the entire benchmark
// Run with: cargo bench --bench memory
// Then view the generated dhat-heap.json at https://nnethercote.github.io/dh_view/dh_view.html
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// Build a CityModel with the specified vertex index type and number of cityobjects.
/// Each cityobject will have a solid geometry with 8 vertices (a cube).
fn build_model<VR: VertexRef>(
    n_cityobjects: usize,
) -> CityModel<VR, ResourceId32, OwnedStringStorage> {
    let mut model = CityModel::<VR, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
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

fn bench_memory_u16(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");
    let n_cityobjects = 7_000;

    group.bench_function(BenchmarkId::new("u16", n_cityobjects), |b| {
        b.iter(|| {
            let model = build_model::<u16>(black_box(n_cityobjects));
            black_box(model);
        });
    });

    group.finish();
}

fn bench_memory_u32(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");
    let n_cityobjects = 7_000;

    group.bench_function(BenchmarkId::new("u32", n_cityobjects), |b| {
        b.iter(|| {
            let model = build_model::<u32>(black_box(n_cityobjects));
            black_box(model);
        });
    });

    group.finish();
}

fn bench_memory_u64(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");
    let n_cityobjects = 7_000;

    group.bench_function(BenchmarkId::new("u64", n_cityobjects), |b| {
        b.iter(|| {
            let model = build_model::<u64>(black_box(n_cityobjects));
            black_box(model);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_memory_u16,
    bench_memory_u32,
    bench_memory_u64
);
criterion_main!(benches);
