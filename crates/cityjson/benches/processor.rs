//! Benchmarks for processing and querying CityModels.

#[allow(dead_code)]
mod support;

use criterion::{Criterion, criterion_group, criterion_main};
use rand::Rng;
use std::hint::black_box;
use support::{DEFAULT_SIZE_PROCESSOR, FAST_SIZE_PROCESSOR, params_from_env, rng_from_seed};

// ==================== DEFAULT BACKEND BENCHMARKS ====================

#[cfg(feature = "backend-default")]
mod default_benches {
    use super::*;

    use cityjson::prelude::*;
    use cityjson::v2_0::*;

    /// Generate a citymodel with n cityobjects, each with a solid geometry type.
    fn generate_citymodel(n: usize, seed: u64) -> CityModel<u32, ResourceId32, OwnedStringStorage> {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        let mut rng = rng_from_seed(seed);

        let metadata = model.metadata_mut();
        metadata.set_identifier(CityModelIdentifier::new("benchmark-model".to_string()));
        metadata.set_reference_system(CRS::new(
            "https://www.opengis.net/def/crs/EPSG/0/2355".to_string(),
        ));

        for i in 0..n {
            let vertices: Vec<_> = (0..8)
                .map(|_| {
                    let x = rng.random_range(0..100000);
                    let y = rng.random_range(0..100000);
                    let z = rng.random_range(0..1000);
                    model.add_vertex(QuantizedCoordinate::new(x, y, z)).unwrap()
                })
                .collect();

            let mut cityobject =
                CityObject::new(format!("building-{:06}", i), CityObjectType::Building);

            {
                let mut geometry_builder =
                    GeometryBuilder::new(&mut model, GeometryType::Solid, BuilderMode::Regular)
                        .with_lod(LoD::LoD2);

                let bv: Vec<_> = vertices
                    .iter()
                    .map(|&v| geometry_builder.add_vertex(v))
                    .collect();

                // Create 6 faces of the cube
                let ring_bottom = geometry_builder
                    .add_ring(&[bv[0], bv[1], bv[2], bv[3]])
                    .unwrap();
                let surface_bottom = geometry_builder.start_surface();
                geometry_builder
                    .add_surface_outer_ring(ring_bottom)
                    .unwrap();

                let ring_top = geometry_builder
                    .add_ring(&[bv[4], bv[7], bv[6], bv[5]])
                    .unwrap();
                let surface_top = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_top).unwrap();

                let ring_front = geometry_builder
                    .add_ring(&[bv[0], bv[1], bv[5], bv[4]])
                    .unwrap();
                let surface_front = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_front).unwrap();

                let ring_right = geometry_builder
                    .add_ring(&[bv[1], bv[2], bv[6], bv[5]])
                    .unwrap();
                let surface_right = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_right).unwrap();

                let ring_back = geometry_builder
                    .add_ring(&[bv[2], bv[3], bv[7], bv[6]])
                    .unwrap();
                let surface_back = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_back).unwrap();

                let ring_left = geometry_builder
                    .add_ring(&[bv[3], bv[0], bv[4], bv[7]])
                    .unwrap();
                let surface_left = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_left).unwrap();

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
            }

            model.cityobjects_mut().add(cityobject);
        }

        model
    }

    /// Compute the mean x,y,z coordinate for each geometry of each cityobject
    fn compute_mean_coordinates(
        model: &CityModel<u32, ResourceId32, OwnedStringStorage>,
    ) -> Vec<(f64, f64, f64)> {
        let mut means = Vec::new();

        for (_id, cityobject) in model.cityobjects().iter() {
            if let Some(geometries) = cityobject.geometry() {
                for geometry_ref in geometries {
                    if let Some(geometry) = model.get_geometry(*geometry_ref)
                        && let Some(boundary) = geometry.boundaries()
                    {
                        let vertex_indices = boundary.vertices();

                        if vertex_indices.is_empty() {
                            continue;
                        }

                        let mut sum_x = 0i64;
                        let mut sum_y = 0i64;
                        let mut sum_z = 0i64;
                        let mut count = 0usize;

                        for vertex_idx in vertex_indices.iter() {
                            if let Some(vertex) = model.get_vertex(*vertex_idx) {
                                sum_x += vertex.x();
                                sum_y += vertex.y();
                                sum_z += vertex.z();
                                count += 1;
                            }
                        }

                        if count > 0 {
                            let mean_x = sum_x as f64 / count as f64;
                            let mean_y = sum_y as f64 / count as f64;
                            let mean_z = sum_z as f64 / count as f64;
                            means.push((mean_x, mean_y, mean_z));
                        }
                    }
                }
            }
        }

        means
    }

    pub fn benchmark_mean_coordinates(c: &mut Criterion) {
        let params = params_from_env(DEFAULT_SIZE_PROCESSOR, FAST_SIZE_PROCESSOR);
        let model = generate_citymodel(params.size, params.seed);

        c.bench_function("compute_mean_coordinates", |b| {
            b.iter(|| {
                let means = compute_mean_coordinates(black_box(&model));
                black_box(means);
            })
        });
    }
}

// ==================== NESTED BACKEND BENCHMARKS ====================

#[cfg(feature = "backend-nested")]
mod nested_benches {
    use super::*;
    use cityjson::backend::nested;
    use cityjson::backend::nested::boundary::Boundary;
    use cityjson::prelude::*;

    /// Generate a citymodel with n cityobjects, each with a solid geometry type.
    fn generate_citymodel(
        n: usize,
        seed: u64,
    ) -> nested::CityModel<OwnedStringStorage, ResourceId32> {
        let mut model = nested::CityModel::<OwnedStringStorage, ResourceId32>::new(CityModelType::CityJSON);
        let mut rng = rng_from_seed(seed);

        let metadata = model.metadata_mut();
        metadata.set_identifier(CityModelIdentifier::new("benchmark-model".to_string()));
        metadata.set_reference_system(CRS::new(
            "https://www.opengis.net/def/crs/EPSG/0/2355".to_string(),
        ));

        for i in 0..n {
            let vertices: Vec<_> = (0..8)
                .map(|_| {
                    let x = rng.random_range(0..100000);
                    let y = rng.random_range(0..100000);
                    let z = rng.random_range(0..1000);
                    model.add_vertex(QuantizedCoordinate::new(x, y, z)).unwrap()
                })
                .collect();

            let co_id = format!("building-{:06}", i);
            let cityobject = nested::CityObject::new(nested::cityobject::CityObjectType::Building);
            model.add_cityobject(co_id.clone(), cityobject);

            {
                let mut geometry_builder = nested::GeometryBuilder::new(
                    &mut model,
                    GeometryType::Solid,
                    nested::BuilderMode::Regular,
                )
                .with_lod(LoD::LoD2);

                for &v in &vertices {
                    geometry_builder.add_vertex(v).unwrap();
                }

                // Create 6 faces of the cube
                let ring_bottom = geometry_builder.add_ring(&[0, 1, 2, 3]).unwrap();
                geometry_builder.start_surface().unwrap();
                geometry_builder
                    .add_surface_outer_ring(ring_bottom)
                    .unwrap();
                let surface_bottom = geometry_builder.end_surface().unwrap();

                let ring_top = geometry_builder.add_ring(&[4, 7, 6, 5]).unwrap();
                geometry_builder.start_surface().unwrap();
                geometry_builder.add_surface_outer_ring(ring_top).unwrap();
                let surface_top = geometry_builder.end_surface().unwrap();

                let ring_front = geometry_builder.add_ring(&[0, 1, 5, 4]).unwrap();
                geometry_builder.start_surface().unwrap();
                geometry_builder.add_surface_outer_ring(ring_front).unwrap();
                let surface_front = geometry_builder.end_surface().unwrap();

                let ring_right = geometry_builder.add_ring(&[1, 2, 6, 5]).unwrap();
                geometry_builder.start_surface().unwrap();
                geometry_builder.add_surface_outer_ring(ring_right).unwrap();
                let surface_right = geometry_builder.end_surface().unwrap();

                let ring_back = geometry_builder.add_ring(&[2, 3, 7, 6]).unwrap();
                geometry_builder.start_surface().unwrap();
                geometry_builder.add_surface_outer_ring(ring_back).unwrap();
                let surface_back = geometry_builder.end_surface().unwrap();

                let ring_left = geometry_builder.add_ring(&[3, 0, 4, 7]).unwrap();
                geometry_builder.start_surface().unwrap();
                geometry_builder.add_surface_outer_ring(ring_left).unwrap();
                let surface_left = geometry_builder.end_surface().unwrap();

                geometry_builder.start_shell().unwrap();
                geometry_builder.add_shell_surface(surface_bottom).unwrap();
                geometry_builder.add_shell_surface(surface_top).unwrap();
                geometry_builder.add_shell_surface(surface_front).unwrap();
                geometry_builder.add_shell_surface(surface_right).unwrap();
                geometry_builder.add_shell_surface(surface_back).unwrap();
                geometry_builder.add_shell_surface(surface_left).unwrap();
                geometry_builder.end_shell().unwrap();

                let geometry = geometry_builder.build().unwrap();
                model.add_geometry_to_cityobject(&co_id, geometry).unwrap();
            }
        }

        model
    }

    /// Compute the mean x,y,z coordinate for each geometry of each cityobject
    fn compute_mean_coordinates(
        model: &nested::CityModel<OwnedStringStorage, ResourceId32>,
    ) -> Vec<(f64, f64, f64)> {
        let mut means = Vec::new();

        for (_id, cityobject) in model.cityobjects().iter() {
            if let Some(geometries) = cityobject.geometry() {
                for geometry in geometries {
                    if let Some(boundary) = geometry.boundaries() {
                        let mut sum_x = 0i64;
                        let mut sum_y = 0i64;
                        let mut sum_z = 0i64;
                        let mut count = 0usize;

                        // For nested backend, boundaries are directly accessible
                        if let Boundary::Solid(shells) = boundary {
                            for shell in shells {
                                for surface in shell {
                                    for ring in surface {
                                        for vertex_idx in ring {
                                            if let Some(vertex) = model.get_vertex(*vertex_idx) {
                                                sum_x += vertex.x();
                                                sum_y += vertex.y();
                                                sum_z += vertex.z();
                                                count += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if count > 0 {
                            let mean_x = sum_x as f64 / count as f64;
                            let mean_y = sum_y as f64 / count as f64;
                            let mean_z = sum_z as f64 / count as f64;
                            means.push((mean_x, mean_y, mean_z));
                        }
                    }
                }
            }
        }

        means
    }

    pub fn benchmark_mean_coordinates(c: &mut Criterion) {
        let params = params_from_env(DEFAULT_SIZE_PROCESSOR, FAST_SIZE_PROCESSOR);
        let model = generate_citymodel(params.size, params.seed);

        c.bench_function("compute_mean_coordinates", |b| {
            b.iter(|| {
                let means = compute_mean_coordinates(black_box(&model));
                black_box(means);
            })
        });
    }
}

// ==================== CRITERION GROUPS ====================

#[cfg(all(feature = "backend-default", not(feature = "backend-nested")))]
criterion_group!(benches, default_benches::benchmark_mean_coordinates);

#[cfg(all(feature = "backend-nested", not(feature = "backend-default")))]
criterion_group!(benches, nested_benches::benchmark_mean_coordinates);

#[cfg(all(feature = "backend-default", feature = "backend-nested"))]
criterion_group!(
    benches,
    default_benches::benchmark_mean_coordinates,
    nested_benches::benchmark_mean_coordinates
);

criterion_main!(benches);
