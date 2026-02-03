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
    use std::collections::HashMap;

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

        let mut material = Material::new("benchmark_material".to_string());
        material.set_ambient_intensity(Some(0.5));
        material.set_diffuse_color(Some([0.8, 0.8, 0.8]));
        let texture = Texture::new("benchmark_texture.png".to_string(), ImageType::Png);

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
            let attrs = cityobject.attributes_mut();
            let height = 10.0 + (i as f64) * 0.5 + (seed as f64) * 0.001;
            attrs.insert("attr_null".to_string(), AttributeValue::Null);
            attrs.insert("attr_bool".to_string(), AttributeValue::Bool(i % 2 == 0));
            attrs.insert("attr_unsigned".to_string(), AttributeValue::Unsigned(i as u64));
            attrs.insert("attr_integer".to_string(), AttributeValue::Integer(i as i64));
            attrs.insert("attr_float".to_string(), AttributeValue::Float(height));
            attrs.insert(
                "attr_string".to_string(),
                AttributeValue::String(format!("name-{}", i)),
            );
            attrs.insert(
                "attr_vec".to_string(),
                AttributeValue::Vec(vec![
                    Box::new(AttributeValue::Integer(i as i64)),
                    Box::new(AttributeValue::Float(height)),
                ]),
            );
            let mut attr_map = HashMap::new();
            attr_map.insert(
                "key".to_string(),
                Box::new(AttributeValue::String("value".to_string())),
            );
            attrs.insert("attr_map".to_string(), AttributeValue::Map(attr_map));
            attrs.insert(
                "attr_geometry".to_string(),
                AttributeValue::Geometry(ResourceId32::new(0, 0)),
            );

            {
                let mut geometry_builder =
                    GeometryBuilder::new(&mut model, GeometryType::Solid, BuilderMode::Regular)
                        .with_lod(LoD::LoD2);

                let mut ground_semantic = Semantic::new(SemanticType::GroundSurface);
                let ground_attrs = ground_semantic.attributes_mut();
                ground_attrs.insert(
                    "area".to_string(),
                    AttributeValue::Float(100.0 + (i as f64) * 0.5),
                );

                let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
                let roof_attrs = roof_semantic.attributes_mut();
                roof_attrs.insert(
                    "azimuth".to_string(),
                    AttributeValue::Float((i % 360) as f64),
                );
                roof_attrs.insert(
                    "slope".to_string(),
                    AttributeValue::Float(15.0 + ((i % 30) as f64)),
                );

                let mut wall_semantic = Semantic::new(SemanticType::WallSurface);
                let wall_attrs = wall_semantic.attributes_mut();
                wall_attrs.insert(
                    "orientation".to_string(),
                    AttributeValue::String("north".to_string()),
                );

                let bv0 = geometry_builder.add_vertex(vertices[0]);
                let bv1 = geometry_builder.add_vertex(vertices[1]);
                let bv2 = geometry_builder.add_vertex(vertices[2]);
                let bv3 = geometry_builder.add_vertex(vertices[3]);
                let bv4 = geometry_builder.add_vertex(vertices[4]);
                let bv5 = geometry_builder.add_vertex(vertices[5]);
                let bv6 = geometry_builder.add_vertex(vertices[6]);
                let bv7 = geometry_builder.add_vertex(vertices[7]);

                let ring_bottom = geometry_builder.add_ring(&[bv0, bv1, bv2, bv3]).unwrap();
                let surface_bottom = geometry_builder.start_surface();
                geometry_builder
                    .add_surface_outer_ring(ring_bottom)
                    .unwrap();
                geometry_builder
                    .set_semantic_surface(None, ground_semantic, false)
                    .unwrap();

                let ring_top = geometry_builder.add_ring(&[bv4, bv7, bv6, bv5]).unwrap();
                let surface_top = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_top).unwrap();
                geometry_builder
                    .set_semantic_surface(None, roof_semantic, false)
                    .unwrap();
                geometry_builder
                    .set_material_surface(None, material.clone(), "default".to_string(), true)
                    .unwrap();

                let ring_front = geometry_builder.add_ring(&[bv0, bv1, bv5, bv4]).unwrap();
                let surface_front = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_front).unwrap();
                geometry_builder
                    .set_semantic_surface(None, wall_semantic, false)
                    .unwrap();

                let uv0 = geometry_builder.add_uv_coordinate(0.0, 0.0);
                let uv1 = geometry_builder.add_uv_coordinate(1.0, 0.0);
                let uv2 = geometry_builder.add_uv_coordinate(1.0, 1.0);
                let uv3 = geometry_builder.add_uv_coordinate(0.0, 1.0);
                geometry_builder.map_vertex_to_uv(bv0, uv0);
                geometry_builder.map_vertex_to_uv(bv1, uv1);
                geometry_builder.map_vertex_to_uv(bv5, uv2);
                geometry_builder.map_vertex_to_uv(bv4, uv3);
                geometry_builder
                    .set_texture_ring(None, texture.clone(), "default".to_string(), true)
                    .unwrap();

                let ring_right = geometry_builder.add_ring(&[bv1, bv2, bv6, bv5]).unwrap();
                let surface_right = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_right).unwrap();

                let ring_back = geometry_builder.add_ring(&[bv2, bv3, bv7, bv6]).unwrap();
                let surface_back = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_back).unwrap();

                let ring_left = geometry_builder.add_ring(&[bv3, bv0, bv4, bv7]).unwrap();
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
    use cityjson::backend::nested::appearance::{ImageType, Material, Texture};
    use cityjson::backend::nested::attributes::AttributeValue;
    use cityjson::backend::nested::nested_boundary::Boundary;
    use cityjson::prelude::*;
    use std::collections::HashMap;

    /// Generate a citymodel with n cityobjects, each with a solid geometry type.
    fn generate_citymodel(
        n: usize,
        seed: u64,
    ) -> nested::CityModel<OwnedStringStorage, ResourceId32> {
        let mut model = nested::CityModel::<OwnedStringStorage, ResourceId32>::new(CityModelType::CityJSON);
        let mut rng = rng_from_seed(seed);

        let metadata = model.metadata_mut();
        metadata.set_identifier(nested::metadata::CityModelIdentifier::new(
            "benchmark-model".to_string(),
        ));
        metadata.set_reference_system(nested::metadata::CRS::new(
            "https://www.opengis.net/def/crs/EPSG/0/2355".to_string(),
        ));

        let mut material = Material::new("benchmark_material".to_string());
        material.set_ambient_intensity(Some(0.5));
        material.set_diffuse_color(Some([0.8, 0.8, 0.8]));
        let material_idx = model.add_material(material);
        let texture = Texture::new("benchmark_texture.png".to_string(), ImageType::Png);
        let texture_idx = model.add_texture(texture);
        model.set_default_theme_material(Some("default".to_string()));
        model.set_default_theme_texture(Some("default".to_string()));

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
            let mut cityobject =
                nested::CityObject::new(nested::cityobject::CityObjectType::Building);
            let attrs = cityobject.attributes_mut();
            let height = 10.0 + (i as f64) * 0.5 + (seed as f64) * 0.001;
            attrs.insert("attr_null".to_string(), AttributeValue::Null);
            attrs.insert("attr_bool".to_string(), AttributeValue::Bool(i % 2 == 0));
            attrs.insert("attr_unsigned".to_string(), AttributeValue::Unsigned(i as u64));
            attrs.insert("attr_integer".to_string(), AttributeValue::Integer(i as i64));
            attrs.insert("attr_float".to_string(), AttributeValue::Float(height));
            attrs.insert(
                "attr_string".to_string(),
                AttributeValue::String(format!("name-{}", i)),
            );
            attrs.insert(
                "attr_vec".to_string(),
                AttributeValue::Vec(vec![
                    Box::new(AttributeValue::Integer(i as i64)),
                    Box::new(AttributeValue::Float(height)),
                ]),
            );
            let mut attr_map = HashMap::new();
            attr_map.insert(
                "key".to_string(),
                Box::new(AttributeValue::String("value".to_string())),
            );
            attrs.insert("attr_map".to_string(), AttributeValue::Map(attr_map));
            attrs.insert(
                "attr_geometry".to_string(),
                AttributeValue::Geometry(Box::new(nested::Geometry::new(
                    GeometryType::Solid,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ))),
            );
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
                let mut ground_semantic =
                    nested::semantics::Semantic::new(nested::semantics::SemanticType::GroundSurface);
                let ground_attrs = ground_semantic.attributes_mut();
                ground_attrs.insert(
                    "area".to_string(),
                    AttributeValue::Float(100.0 + (i as f64) * 0.5),
                );
                geometry_builder
                    .set_semantic_surface(0, ground_semantic, false)
                    .unwrap();
                let surface_bottom = geometry_builder.end_surface().unwrap();

                let ring_top = geometry_builder.add_ring(&[4, 7, 6, 5]).unwrap();
                geometry_builder.start_surface().unwrap();
                geometry_builder.add_surface_outer_ring(ring_top).unwrap();
                let mut roof_semantic =
                    nested::semantics::Semantic::new(nested::semantics::SemanticType::RoofSurface);
                let roof_attrs = roof_semantic.attributes_mut();
                roof_attrs.insert(
                    "azimuth".to_string(),
                    AttributeValue::Float((i % 360) as f64),
                );
                roof_attrs.insert(
                    "slope".to_string(),
                    AttributeValue::Float(15.0 + ((i % 30) as f64)),
                );
                geometry_builder
                    .set_semantic_surface(1, roof_semantic, true)
                    .unwrap();
                let surface_top = geometry_builder.end_surface().unwrap();
                geometry_builder
                    .set_material_surface("default".to_string(), surface_top, material_idx)
                    .unwrap();

                let ring_front = geometry_builder.add_ring(&[0, 1, 5, 4]).unwrap();
                geometry_builder.start_surface().unwrap();
                geometry_builder.add_surface_outer_ring(ring_front).unwrap();
                let mut wall_semantic =
                    nested::semantics::Semantic::new(nested::semantics::SemanticType::WallSurface);
                let wall_attrs = wall_semantic.attributes_mut();
                wall_attrs.insert(
                    "orientation".to_string(),
                    AttributeValue::String("north".to_string()),
                );
                geometry_builder
                    .set_semantic_surface(2, wall_semantic, false)
                    .unwrap();
                let surface_front = geometry_builder.end_surface().unwrap();
                geometry_builder
                    .add_uv_to_vertex(0, UVCoordinate::new(0.0, 0.0))
                    .unwrap();
                geometry_builder
                    .add_uv_to_vertex(1, UVCoordinate::new(1.0, 0.0))
                    .unwrap();
                geometry_builder
                    .add_uv_to_vertex(5, UVCoordinate::new(1.0, 1.0))
                    .unwrap();
                geometry_builder
                    .add_uv_to_vertex(4, UVCoordinate::new(0.0, 1.0))
                    .unwrap();
                geometry_builder
                    .set_texture_ring("default".to_string(), ring_front, texture_idx)
                    .unwrap();

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
